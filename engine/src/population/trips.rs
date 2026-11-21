//! For the view, cache `TripPlan::new(&baseline, n)` once and call `plan.sample(&state)`.
//! Match rows across states by their numeric `id`. A different baseline or budget starts a new plan.
//! Compare `count` and the two endpoints separately. The point nearest a changed group mean can move.
//! See [the MUT comparison](TRIP_PLAN.md) for the measured change.
use super::{Point, Population};
use serde::Serialize;

const MODES: [&str; 5] = ["car", "public", "walk", "bicycle", "other"];
const KM_PER_LON: f64 = 92.88;
const KM_PER_LAT: f64 = 110.54;

#[derive(Serialize)]
pub struct TripGroup {
    /// Match this ID only within one baseline plan and row budget.
    pub id: usize,
    pub origin: [f64; 2],
    pub destination: [f64; 2],
    pub mode: &'static str,
    pub half_hour: usize,
    pub count: usize,
    pub origin_cut: String,
    pub destination_cut: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_entrance: Option<u64>,
}

/// JSON array `[id, signed_count_delta, flags]`. Bits 1/2 mark the origin/destination,
/// 4 the departure half-hour, 8 an added group and 16 a removed group.
#[derive(Serialize)]
pub struct TripDelta(pub usize, pub i64, pub u8);

/// Compare two outputs of the same plan, both in ascending ID order.
pub fn trip_deltas(mut base: &[TripGroup], mut scenario: &[TripGroup]) -> Vec<TripDelta> {
    let mut out = Vec::new();
    loop {
        match (base.first(), scenario.first()) {
            (Some(b), Some(s)) if b.id == s.id => {
                let mut flags = 0;
                if b.origin != s.origin || b.origin_cut != s.origin_cut { flags |= 1; }
                if b.destination != s.destination || b.destination_cut != s.destination_cut { flags |= 2; }
                if b.half_hour != s.half_hour { flags |= 4; }
                let count = s.count as i64 - b.count as i64;
                if count != 0 || flags != 0 { out.push(TripDelta(s.id, count, flags)); }
                base = &base[1..];
                scenario = &scenario[1..];
            }
            (Some(b), s) if s.is_none_or(|s| b.id < s.id) => {
                out.push(TripDelta(b.id, -(b.count as i64), 16));
                base = &base[1..];
            }
            (_, Some(s)) => {
                out.push(TripDelta(s.id, s.count as i64, 8));
                scenario = &scenario[1..];
            }
            _ => break,
        }
    }
    out
}

// Interleave the four 1-km coordinates of a home/job pair into one spatial sort key.
fn point_key(point: &Point) -> u64 {
    let x = ((point.longitude + 180.0) * KM_PER_LON).floor() as u64;
    let y = ((point.latitude + 90.0) * KM_PER_LAT).floor() as u64;
    let mut key = 0;
    for bit in 0..16 {
        key |= (((x >> bit) & 1) | (((y >> bit) & 1) << 1)) << (4 * bit);
    }
    key
}

fn group_counts(sizes: [usize; 5], budget: usize) -> [usize; 5] {
    let mut modes = [0, 1, 2, 3, 4];
    modes.sort_by_key(|&i| std::cmp::Reverse(sizes[i]));
    let mut counts = [0; 5];
    for &i in modes.iter().filter(|&&i| sizes[i] >= 2).take(budget) {
        counts[i] = 1;
    }
    // Divide the remaining rows by mode size. Every group must have at least two workers.
    while counts.iter().sum::<usize>() < budget {
        let priority = |i: usize| sizes[i] as f64 / (2 * counts[i] + 1) as f64;
        let Some(i) = (0..5).filter(|&i| counts[i] < sizes[i] / 2)
            .max_by(|&a, &b| priority(a).total_cmp(&priority(b)))
        else { break };
        counts[i] += 1;
    }
    counts
}

fn coordinates(points: &[Point]) -> Vec<[f64; 2]> {
    points.iter().map(|p| [p.longitude, p.latitude]).collect()
}

fn representatives(members: &[(u32, u32)], homes: &[[f64; 2]], jobs: &[[f64; 2]]) -> (usize, usize) {
    let mut origin = [0.0; 2];
    let mut destination = [0.0; 2];
    for &(h, j) in members {
        let home = homes[h as usize];
        let job = jobs[j as usize];
        for k in 0..2 {
            origin[k] += home[k];
            destination[k] += job[k];
        }
    }
    let mean = |x| x / members.len() as f64;
    let origin = origin.map(mean);
    let destination = destination.map(mean);
    let distance = |p: [f64; 2], center: [f64; 2]| {
        ((p[0] - center[0]) * KM_PER_LON).powi(2) + ((p[1] - center[1]) * KM_PER_LAT).powi(2)
    };
    let mut home = members[0].0 as usize;
    let mut job = members[0].1 as usize;
    let mut home_distance = distance(homes[home], origin);
    let mut job_distance = distance(jobs[job], destination);
    for &(h, j) in &members[1..] {
        let hd = distance(homes[h as usize], origin);
        let jd = distance(jobs[j as usize], destination);
        if hd.total_cmp(&home_distance).is_lt() {
            home = h as usize;
            home_distance = hd;
        }
        if jd.total_cmp(&job_distance).is_lt() {
            job = j as usize;
            job_distance = jd;
        }
    }
    (home, job)
}

fn visit_workers(pop: &Population, mut visit: impl FnMut(usize, u64, &super::Person)) {
    let homes: Vec<_> = pop.homes.iter().map(point_key).collect();
    let jobs: Vec<_> = pop.jobs.iter().map(|p| point_key(p) << 2).collect();
    for p in pop.persons.iter().filter(|p| !p.student) {
        let mode = match p.mode { 1..=4 => (p.mode - 1) as usize, _ => 4 };
        visit(mode, homes[p.home as usize] | jobs[p.dest as usize], p);
    }
}

struct Slot { start: u64, id: usize, half_hour: usize }

/// Build once on the baseline, then call `sample` on each state.
/// An ID identifies a mode and a fixed range of home/job cells in this plan.
pub struct TripPlan { slots: [Vec<Slot>; 5], entrance_group_id: usize }

impl TripPlan {
    pub fn new(baseline: &Population, n: usize) -> Self {
        let mut keys = [const { Vec::new() }; 5];
        visit_workers(baseline, |mode, key, _| keys[mode].push(key));
        let counts = group_counts(keys.each_ref().map(Vec::len), n);
        let mut next_id = 0;
        let slots = std::array::from_fn(|mode| {
            let keys = &mut keys[mode];
            let n = counts[mode];
            if n == 0 { return Vec::new(); }
            keys.sort_unstable();
            // Keep identical cell-pairs together even when a baseline quantile falls inside one.
            let mut starts: Vec<_> = (0..n).map(|i| keys[i * keys.len() / n]).collect();
            starts.dedup();
            starts[0] = 0;
            let mut order: Vec<_> = (0..starts.len()).collect();
            order.sort_unstable_by_key(|i| i.reverse_bits());
            let mut slots: Vec<_> = starts.into_iter().map(|start| {
                let id = next_id;
                next_id += 1;
                Slot { start, id, half_hour: 0 }
            }).collect();
            for (rank, &i) in order.iter().enumerate() {
                slots[i].half_hour = departure_half_hour(mode, (rank as f64 + 0.5) / order.len() as f64);
            }
            slots
        });
        Self { slots, entrance_group_id: n }
    }

    /// Each group uses current workers and current representative points. A mode outside
    /// the baseline budget or with fewer than two current workers contributes no rows.
    pub fn sample(&self, pop: &Population) -> Vec<TripGroup> {
        let mut bins = self.slots.each_ref().map(|slots| vec![Vec::new(); slots.len()]);
        visit_workers(pop, |mode, key, person| {
            let slots = &self.slots[mode];
            if !slots.is_empty() {
                let bin = slots.partition_point(|s| s.start <= key) - 1;
                bins[mode][bin].push((person.home, person.dest));
            }
        });
        let homes = coordinates(&pop.homes);
        let jobs = coordinates(&pop.jobs);
        let ahumada_origin = pop.homes.iter().position(|p| p.id == "13101021008002");
        let ahumada_destination = pop.jobs.iter().position(|p| p.id == "13101021005003");
        let door_trip = ahumada_origin.zip(ahumada_destination)
            .map(|(home, job)| (home as u32, job as u32));
        let mut entrance_group = None;
        let mut out = Vec::new();
        for (mode, bins) in bins.iter_mut().enumerate() {
            // Fold a singleton into the adjacent occupied interval of the same mode.
            let mut previous: Option<usize> = None;
            for i in 0..bins.len() {
                if bins[i].is_empty() { continue; }
                if let Some(j) = previous {
                    if bins[j].len() == 1 {
                        let person = bins[j].pop().unwrap();
                        bins[i].push(person);
                    } else if bins[i].len() == 1 {
                        let person = bins[i].pop().unwrap();
                        bins[j].push(person);
                        continue;
                    }
                }
                previous = Some(i);
            }
            for (members, slot) in bins.iter().zip(&self.slots[mode]).filter(|(m, _)| m.len() >= 2) {
                // Allocate one modeled walker to the Portal Fernández Concha door
                // in the assigned job block. Keep the sampled departure bin.
                let entrance_walker = mode == 2 && door_trip.is_some_and(|pair| members.contains(&pair));
                let (home, job) = representatives(members, &homes, &jobs);
                let (origin, destination) = (&pop.homes[home], &pop.jobs[job]);
                if entrance_walker {
                    let (home, job) = door_trip.unwrap();
                    entrance_group = Some(TripGroup {
                        id: self.entrance_group_id,
                        origin: homes[home as usize],
                        destination: [-70.6510204, -33.4385882],
                        origin_cut: pop.homes[home as usize].cut.clone(),
                        destination_cut: pop.jobs[job as usize].cut.clone(),
                        mode: "walk",
                        count: 1,
                        half_hour: slot.half_hour,
                        building_entrance: Some(11064204030),
                    });
                }
                out.push(TripGroup {
                    id: slot.id, origin: [origin.longitude, origin.latitude],
                    destination: [destination.longitude, destination.latitude],
                    origin_cut: origin.cut.clone(), destination_cut: destination.cut.clone(),
                    mode: MODES[mode], count: members.len() - usize::from(entrance_walker),
                    half_hour: slot.half_hour, building_entrance: None,
                });
            }
        }
        out.extend(entrance_group);
        out
    }
}

/// Use a shared TripPlan when comparing states.
pub fn sample_trips(pop: &Population, n: usize) -> Vec<TripGroup> {
    if n == 0 { return Vec::new(); }
    TripPlan::new(pop, n).sample(pop)
}

fn departure_half_hour(mode: usize, quantile: f64) -> usize {
    use std::sync::OnceLock;
    static PROFILE: OnceLock<[[f64; 48]; 5]> = OnceLock::new();
    let cumulative = PROFILE.get_or_init(|| {
        let mut shares = [[0.0; 48]; 5];
        let table = include_bytes!("../../../data/mvp/work_departures.csv");
        for row in csv::Reader::from_reader(table.as_slice()).records() {
            let r = row.unwrap();
            let m = MODES.iter().position(|&name| name == &r[0]).unwrap();
            shares[m][r[1].parse::<usize>().unwrap()] = r[3].parse().unwrap();
        }
        for bins in &mut shares {
            for t in 1..48 { bins[t] += bins[t - 1]; }
        }
        shares
    });
    cumulative[mode].partition_point(|&p| p < quantile).min(47)
}
