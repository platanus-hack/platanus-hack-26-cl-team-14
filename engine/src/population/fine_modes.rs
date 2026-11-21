//! Restore Census submodes after the three-way public/car/other choice.
use super::{Person, Point, Population};
use std::collections::HashMap;

/// The choice caller marks new "other" workers with this code until apply() assigns a Census code.
pub const NEW_OTHER_MODE: i8 = -2;
const CODES: [i8; 6] = [3, 4, 5, 6, 7, -99];
type Counts = [usize; 6];
type Pair = (u32, u32);

fn cut_ids(points: &[Point]) -> Vec<u32> {
    points.iter().map(|p| p.cut.parse().unwrap()).collect()
}

fn targets(counts: Counts, n: usize) -> Counts {
    let total: usize = counts.iter().sum();
    let mut cumulative = 0;
    let mut used = 0;
    counts.map(|c| {
        cumulative += c;
        let end = (n * cumulative + total / 2) / total;
        let slots = end - used;
        used = end;
        slots
    })
}

// SplitMix64 gives independent orderings for quota retention and distance-weighted selection.
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

fn bike_priority(p: &Person, pop: &Population, seed: u64, slope: f64) -> f64 {
    let h = &pop.homes[p.home as usize];
    let j = &pop.jobs[p.dest as usize];
    let kx = 111.32 * ((h.latitude + j.latitude) * 0.5).to_radians().cos();
    let km = ((h.longitude - j.longitude) * kx).hypot((h.latitude - j.latitude) * 110.54);
    let u = ((mix(seed ^ 0x8178cadc52a55d63) >> 12) as f64 + 0.5) / (1u64 << 52) as f64;
    // Gumbel-top-k samples by relative EOD bike odds. The fitted intercept cancels for a fixed quota.
    slope * km.ln_1p() - (-u.ln()).ln()
}

pub struct FineModes {
    od: HashMap<Pair, Counts>,
    home: HashMap<u32, Counts>,
    bike_slope: f64,
}

impl FineModes {
    /// Count the observed OD submodes before the city changes. Each worker has unit weight.
    pub fn from_baseline(pop: &Population) -> Self {
        let (homes, jobs) = (cut_ids(&pop.homes), cut_ids(&pop.jobs));
        let mut od: HashMap<Pair, Counts> = HashMap::new();
        let mut home: HashMap<u32, Counts> = HashMap::new();
        for p in pop.persons.iter().filter(|p| !p.student) {
            if let Some(m) = CODES.iter().position(|&c| c == p.mode) {
                let o = homes[p.home as usize];
                od.entry((o, jobs[p.dest as usize])).or_default()[m] += 1;
                home.entry(o).or_default()[m] += 1;
            }
        }
        #[derive(serde::Deserialize)]
        struct Coefficient { slope_log1p_km: f64 }
        let source = include_bytes!("../../../data/mvp/active_mode_distance.json");
        let fit: Coefficient = serde_json::from_slice(source).unwrap();
        Self { od, home, bike_slope: fit.slope_log1p_km }
    }

    /// Apply after destinations and coarse modes are set. Returns the number of new or changed fine modes.
    pub fn apply(&self, pop: &mut Population) -> usize {
        let (homes, jobs) = (cut_ids(&pop.homes), cut_ids(&pop.jobs));
        let mut groups: HashMap<Pair, (Counts, Vec<usize>)> = HashMap::new();
        for (i, p) in pop.persons.iter().enumerate().filter(|(_, p)| !p.student && p.mode != 1 && p.mode != 2) {
            let group = groups.entry((homes[p.home as usize], jobs[p.dest as usize])).or_default();
            if let Some(m) = CODES.iter().position(|&c| c == p.mode) { group.0[m] += 1; }
            group.1.push(i);
        }

        let mut changed = 0;
        for (od, (current, members)) in groups {
            // An OD with no observed other workers borrows the home comuna’s submode shares.
            let source = self.od.get(&od).unwrap_or_else(|| &self.home[&od.0]);
            let mut free = targets(*source, members.len());
            if current == free { continue; }
            // Only identical synthetic records use input order to break a tie.
            let mut copies = HashMap::new();
            let mut pool: Vec<_> = members.into_iter().map(|i| {
                let p = pop.persons[i];
                let copy = copies.entry((p.home, p.dest, p.mode)).or_insert(0u64);
                let endpoints = ((p.home as u64) << 32) | p.dest as u64;
                let seed = mix(mix(mix(endpoints) ^ p.mode as u8 as u64) ^ *copy);
                *copy += 1;
                (seed, i)
            }).collect();
            pool.sort_unstable();
            pool.retain(|&(_, i)| {
                if let Some(m) = CODES.iter().position(|&c| c == pop.persons[i].mode) {
                    if free[m] > 0 { free[m] -= 1; return false; }
                }
                true
            });
            changed += pool.len();

            let (active, other) = pool.split_at(free[0] + free[1]);
            let mut race: Vec<_> = active.iter().map(|&(seed, i)| {
                (bike_priority(&pop.persons[i], pop, seed, self.bike_slope), i)
            }).collect();
            race.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
            for (rank, (_, i)) in race.into_iter().enumerate() {
                pop.persons[i].mode = if rank < free[1] { 4 } else { 3 };
            }
            let remaining = (2..CODES.len()).flat_map(|m| std::iter::repeat_n(CODES[m], free[m]));
            for (&(_, i), mode) in other.iter().zip(remaining) { pop.persons[i].mode = mode; }
        }
        changed
    }
}
