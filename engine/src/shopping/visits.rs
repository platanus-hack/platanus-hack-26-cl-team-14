//! Optional weekday shopping visits of persons (V1 needs layer, #388, #402).
//! Each eligible person (worker or other resident) decides once whether to shop and where. The utility of a destination
//! comuna is the fitted Huff utility of the shopping layer, α·ln(shops+1) − β·minutes, plus a fixed personal taste draw
//! per destination (Gumbel, from a hash of person and destination). With these draws the person-level choice reproduces
//! the Huff shares. The same draws are used after a change, so each person keeps the same preferences.
//! Skip is one more option. Its utility is calibrated per home comuna at baseline, so the expected trips equal the
//! shopping layer. When a change lowers the utility of every destination, more people skip (logit assumption).
use super::super::optional_visit::{VisitOption, choose_visit};
use super::{ALPHA, BETA, Shop, flows};
use serde_json::{Value, json};

/// EOD 2012 weekday: 0.246023 shopping trips per resident, 15.30% of residents with a shopping trip (#351).
const TRIPS_PER_SHOPPER: f64 = 0.246023 / 0.1530;
/// Seed of the taste draws, distinct from the departure and retention hashes.
const SEED: u64 = 0x5EED_5140_7A11_0000;

fn gumbel(key: u64) -> f64 {
    let mut z = key.wrapping_add(SEED).wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    let u = ((z >> 11) as f64 + 0.5) / (1u64 << 53) as f64;
    -(-u.ln()).ln()
}

/// Huff utility of each (home, destination) pair.
fn utilities(n: usize, shops: &[Shop], minutes: &[f64]) -> Vec<f64> {
    let mut size = vec![0.0_f64; n];
    for s in shops { size[s.2] += 1.0; }
    (0..n * n).map(|i| ALPHA * (size[i % n] + 1.0).ln() - BETA * minutes[i]).collect()
}

fn uniform(key: u64) -> f64 {
    let g = gumbel(key);
    (-(-g).exp()).exp() // inverse of the Gumbel transform
}

/// The baseline decisions of all eligible persons, made once at load, and what each person needs to decide again.
pub struct Visits {
    n: usize,
    homes: Vec<u8>,
    /// The person shops by public transport (EOD 2012 shopping mode share of the home comuna).
    public: Vec<bool>,
    /// Departure half-hour of the shopping trip (EOD 2012 shopping profile of the home comuna).
    depart: Vec<u8>,
    pub before: Vec<Option<u8>>,
    skip: Vec<f64>,
    /// Home comunas with an EOD 2012 shopping rate.
    active: Vec<bool>,
    base: Vec<f64>,
    /// Total variation between the destination shares of the baseline persons and of the shopping layer (Huff flows).
    pub tv_vs_shopping_layer: f64,
}

/// `f(i, options)` for i in 0..len, in parallel chunks; each chunk has its own option buffer.
fn par<T: Send>(len: usize, n: usize, f: impl Fn(usize, &mut [VisitOption<u8>]) -> T + Sync) -> Vec<T> {
    let threads = std::thread::available_parallelism().map_or(1, |t| t.get());
    let chunk = len.div_ceil(threads).max(1);
    let run = |start: usize| {
        let mut options: Vec<VisitOption<u8>> = (0..n as u8).map(|d| VisitOption { choice: d, total_minutes: 0, cost_clp: 0, available: true, preference: 0 }).collect();
        (start..(start + chunk).min(len)).map(|i| f(i, &mut options)).collect::<Vec<T>>()
    };
    // The browser build has no threads; one chunk runs in place.
    if threads == 1 {
        return run(0);
    }
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..len).step_by(chunk).map(|start| scope.spawn(move || run(start))).collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    })
}

fn decide(options: &mut [VisitOption<u8>], v: &[f64], unreachable: &[bool], i: usize, skip: f64) -> Option<u8> {
    let score = |x: f64| (x * 1e6) as i32;
    for (d, option) in options.iter_mut().enumerate() {
        option.preference = score(v[d] + gumbel(((i as u64) << 7) | d as u64));
        option.available = !unreachable[d];
    }
    // Time and budget limits have no Santiago source yet, so they do not bind.
    choose_visit(options, u32::MAX, u32::MAX, score(skip + gumbel(((i as u64) << 7) | 127))).copied()
}

impl Visits {
    /// `homes`: home comuna of each eligible person. `trips`: shopping-layer trips by home comuna.
    /// `profile`: departure (48) and mode (car, public, walk, bicycle, other) shares by home comuna.
    pub fn new(n: usize, homes: &[usize], trips: &[f64], profile: &[([f64; 48], [f64; 5])], shops: &[Shop], minutes: &[f64]) -> Visits {
        let base = utilities(n, shops, minutes);
        let mut eligible = vec![0.0_f64; n];
        for &o in homes { eligible[o] += 1.0; }
        // Skip utility: logsum of the destinations plus ln((1 − p) / p), p = share of eligible people who shop.
        let skip: Vec<f64> = (0..n).map(|o| {
            let p = (trips[o] / TRIPS_PER_SHOPPER / eligible[o].max(1.0)).clamp(1e-9, 0.99);
            base[o * n..][..n].iter().map(|v| v.exp()).sum::<f64>().ln() + ((1.0 - p) / p).ln()
        }).collect();
        let public = homes.iter().enumerate().map(|(i, &o)| uniform(((i as u64) << 7) | 125) < profile[o].1[1]).collect();
        let depart = homes.iter().enumerate().map(|(i, &o)| {
            let (u, mut cumulative) = (uniform(((i as u64) << 7) | 126), 0.0);
            profile[o].0.iter().position(|s| { cumulative += s; u < cumulative }).unwrap_or(47) as u8
        }).collect();
        let none = vec![false; n];
        let before = par(homes.len(), n, |i, options| {
            let o = homes[i];
            if trips[o] == 0.0 { None } else { decide(options, &base[o * n..][..n], &none, i, skip[o]) }
        });
        let (od, _) = flows(trips, shops, minutes, ALPHA, BETA);
        let mut persons = vec![0.0; n];
        for d in before.iter().flatten() { persons[*d as usize] += 1.0; }
        let huff: Vec<f64> = (0..n).map(|d| (0..n).map(|o| od[o * n + d]).sum()).collect();
        let (p, h) = (persons.iter().sum::<f64>(), huff.iter().sum::<f64>());
        let tv_vs_shopping_layer = (0..n).map(|d| (persons[d] / p - huff[d] / h).abs()).sum::<f64>() / 2.0;
        Visits { n, homes: homes.iter().map(|&o| o as u8).collect(), public, depart, before, skip, active: trips.iter().map(|&t| t > 0.0).collect(), base, tv_vs_shopping_layer }
    }

    /// Decisions after the change. Only public-transport shoppers who leave inside `window` (all day when None) see
    /// the scenario minutes; everyone sees the scenario shops. A person whose options did not change keeps the decision.
    /// `unreachable`: comuna pairs that lost every transit path; exposed shoppers cannot choose them.
    pub fn after(&self, shops: &[Shop], base_minutes: &[f64], minutes: &[f64], unreachable: &[bool], window: Option<(usize, usize)>) -> Vec<Option<u8>> {
        let n = self.n;
        let (moved, kept) = (utilities(n, shops, minutes), utilities(n, shops, base_minutes));
        let same = |v: &[f64], o: usize| v[o * n..][..n] == self.base[o * n..][..n];
        let open = vec![false; n];
        let same_moved: Vec<bool> = (0..n).map(|o| same(&moved, o) && !unreachable[o * n..][..n].contains(&true)).collect();
        let same_kept: Vec<bool> = (0..n).map(|o| same(&kept, o)).collect();
        par(self.homes.len(), n, |i, options| {
            let o = self.homes[i] as usize;
            let exposed = self.public[i] && window.is_none_or(|(a, z)| (a..z).contains(&(self.depart[i] as usize)));
            let (v, lost, unchanged) = if exposed { (&moved, &unreachable[o * n..][..n], same_moved[o]) } else { (&kept, &open[..], same_kept[o]) };
            if unchanged || !self.active[o] { self.before[i] } else { decide(options, &v[o * n..][..n], lost, i, self.skip[o]) }
        })
    }
}

/// Counts of the paired decisions, in persons and in trips (persons × trips per shopper).
pub fn summary(cuts: &[String], before: &[Option<u8>], after: &[Option<u8>]) -> Value {
    let (mut same, mut other, mut stopped, mut started, mut by_dest) = (0u64, 0u64, 0u64, 0u64, vec![[0u64; 2]; cuts.len()]);
    for (b, a) in before.iter().zip(after) {
        match (b, a) {
            (Some(x), Some(y)) if x == y => same += 1,
            (Some(_), Some(_)) => other += 1,
            (Some(_), None) => stopped += 1,
            (None, Some(_)) => started += 1,
            _ => {}
        }
        if let Some(x) = b { by_dest[*x as usize][0] += 1; }
        if let Some(y) = a { by_dest[*y as usize][1] += 1; }
    }
    let shoppers = |k: usize| by_dest.iter().map(|d| d[k]).sum::<u64>();
    let trips = |x: u64| (x as f64 * TRIPS_PER_SHOPPER).round();
    json!({
        "shoppers_base": shoppers(0), "shoppers_scenario": shoppers(1),
        "same_destination": same, "other_destination_comuna": other, "stopped_shopping": stopped, "started_shopping": started,
        "trips_base": trips(shoppers(0)), "trips_scenario": trips(shoppers(1)),
        "destinations_changed": by_dest.iter().enumerate().filter(|(_, d)| d[0] != d[1])
            .map(|(d, x)| json!({ "id": cuts[d], "shoppers_base": x[0], "shoppers_scenario": x[1] })).collect::<Vec<_>>(),
        "unit": "persons who make an optional weekday shopping visit (workers and other residents); trips = persons × 1.608 (EOD 2012 trips per shopper)",
    })
}
