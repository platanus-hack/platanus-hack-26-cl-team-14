use super::{Point, Population};
use serde::Serialize;

const MODES: [&str; 5] = ["car", "public", "walk", "bicycle", "other"];
const KM_PER_LON: f64 = 92.88;
const KM_PER_LAT: f64 = 110.54;

#[derive(Serialize)]
pub struct TripGroup {
    pub origin: [f64; 2],
    pub destination: [f64; 2],
    pub mode: &'static str,
    pub half_hour: usize,
    pub count: usize,
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

fn representative(points: &[Point], indices: impl Iterator<Item = usize> + Clone, count: usize) -> [f64; 2] {
    let (mut lon, mut lat) = (0.0, 0.0);
    for i in indices.clone() {
        lon += points[i].longitude;
        lat += points[i].latitude;
    }
    let (lon, lat) = (lon / count as f64, lat / count as f64);
    let distance = |p: &Point| ((p.longitude - lon) * KM_PER_LON).powi(2)
        + ((p.latitude - lat) * KM_PER_LAT).powi(2);
    let p = indices.map(|i| &points[i]).min_by(|a, b| distance(a).total_cmp(&distance(b))).unwrap();
    [p.longitude, p.latitude]
}

/// Aggregate synthetic workers into at most `n` display groups. Each endpoint is the point
/// nearest its group's mean on that side. The half-hour represents the mode's survey profile.
pub fn sample_trips(pop: &Population, n: usize) -> Vec<TripGroup> {
    if n == 0 { return Vec::new(); }
    let homes: Vec<_> = pop.homes.iter().map(point_key).collect();
    let jobs: Vec<_> = pop.jobs.iter().map(|p| point_key(p) << 2).collect();
    let mut groups = [const { Vec::new() }; 5];
    for (i, p) in pop.persons.iter().enumerate().filter(|(_, p)| !p.student) {
        let mode = match p.mode { 1..=4 => (p.mode - 1) as usize, _ => 4 };
        let key = homes[p.home as usize] | jobs[p.dest as usize];
        groups[mode].push((key, i as u32));
    }
    let counts = group_counts(groups.each_ref().map(Vec::len), n);
    let mut out = Vec::new();
    for (mode, rows) in groups.iter_mut().enumerate().filter(|(m, _)| counts[*m] > 0) {
        rows.sort_unstable();
        let size = counts[mode];
        let mut time_order: Vec<_> = (0..size).collect();
        time_order.sort_unstable_by_key(|i| i.reverse_bits());
        for (rank, &group) in time_order.iter().enumerate() {
            let members = &rows[group * rows.len() / size..(group + 1) * rows.len() / size];
            let persons = members.iter().map(|&(_, i)| &pop.persons[i as usize]);
            let origin = representative(&pop.homes, persons.clone().map(|p| p.home as usize), members.len());
            let destination = representative(&pop.jobs, persons.map(|p| p.dest as usize), members.len());
            let quantile = (rank as f64 + 0.5) / size as f64;
            out.push(TripGroup { origin, destination, mode: MODES[mode], count: members.len(),
                half_hour: departure_half_hour(mode, quantile) });
        }
    }
    out
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
