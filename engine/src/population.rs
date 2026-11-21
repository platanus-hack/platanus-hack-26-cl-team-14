#![allow(dead_code)] // fields used by other modules (choice, city)
//! Synthetic person-level state of the RM (gentle-fox 11:52). One record per RM-internal commuter and student.
//! Built deterministically from margins only: `mode_od.csv` (commuters by home comuna, work comuna and mode),
//! `school_od.csv` (students by home and school comuna), `blocks.csv` (home weights), `jobs.csv` (work weights),
//! `schools.csv` (`enrollment_no_preschool`, school weights). Inside a comuna, persons are placed on points by
//! systematic sampling of the cumulative weights, so comuna flows are exact and point totals follow the weights.
//! Label: "estado sintético a nivel de persona, consistente con los márgenes del Censo, no microdatos" — the
//! joint of home point, destination point and mode is not observed in any file.
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Copy)]
pub struct Person { pub home: u32, pub dest: u32, pub student: bool, pub mode: i8 }

pub struct Point { pub id: String, pub cut: String, pub longitude: f64, pub latitude: f64, pub weight: f64 }

pub struct Population { pub homes: Vec<Point>, pub jobs: Vec<Point>, pub schools: Vec<Point>, pub persons: Vec<Person> }

#[derive(Deserialize)]
struct Block { block_id: String, cut: String, longitude: f64, latitude: f64, population: f64 }
#[derive(Deserialize)]
struct Job { block_id: String, cut: String, longitude: f64, latitude: f64, jobs: f64 }
#[derive(Deserialize)]
struct School { rbd: String, cut: String, longitude: f64, latitude: f64, enrollment_no_preschool: f64 }
#[derive(Deserialize)]
struct ModeOd { origin_cut: String, destination_cut: String, mode: i8, commuters: f64 }
#[derive(Deserialize)]
struct SchoolOd { origin_cut: String, destination_cut: String, students: f64 }

fn rows<T: for<'de> Deserialize<'de>>(p: &Path) -> Vec<T> {
    csv::Reader::from_path(p).unwrap().deserialize().map(|r| r.unwrap()).collect()
}

/// Cumulative weights of each comuna's points, for systematic sampling.
struct Sampler { by_cut: HashMap<String, (Vec<u32>, Vec<f64>)>, next: HashMap<String, f64> }

impl Sampler {
    fn new(points: &[Point]) -> Sampler {
        let mut by_cut: HashMap<String, (Vec<u32>, Vec<f64>)> = HashMap::new();
        for (i, p) in points.iter().enumerate().filter(|(_, p)| p.weight > 0.0) {
            let e = by_cut.entry(p.cut.clone()).or_default();
            let c = e.1.last().copied().unwrap_or(0.0) + p.weight;
            e.0.push(i as u32);
            e.1.push(c);
        }
        Sampler { by_cut, next: HashMap::new() }
    }
    /// The next point of a comuna. The k-th person of a comuna sits at the golden-ratio (Weyl) position
    /// frac((k + 0.5) * 0.618...) of its cumulative weight: the points are filled in proportion to their weights,
    /// and persons of one home (or destination) group spread over the whole comuna instead of a contiguous run,
    /// so the order of the input rows does not tie a destination point to one origin.
    fn take(&mut self, cut: &str) -> u32 {
        let (ids, cum) = &self.by_cut[cut];
        let total = *cum.last().unwrap();
        let k = self.next.entry(cut.to_string()).or_insert(0.0);
        let at = ((*k + 0.5) * 0.618_033_988_749_894_9).fract() * total;
        *k += 1.0;
        ids[cum.partition_point(|&c| c <= at).min(ids.len() - 1)]
    }
}

impl Population {
    pub fn build(data: &Path) -> Population {
        let homes: Vec<Point> = rows::<Block>(&data.join("blocks.csv")).into_iter()
            .map(|b| Point { id: b.block_id, cut: b.cut, longitude: b.longitude, latitude: b.latitude, weight: b.population }).collect();
        let jobs: Vec<Point> = rows::<Job>(&data.join("jobs.csv")).into_iter()
            .map(|j| Point { id: j.block_id, cut: j.cut, longitude: j.longitude, latitude: j.latitude, weight: j.jobs }).collect();
        let schools: Vec<Point> = rows::<School>(&data.join("schools.csv")).into_iter()
            .map(|s| Point { id: s.rbd, cut: s.cut, longitude: s.longitude, latitude: s.latitude, weight: s.enrollment_no_preschool }).collect();
        // Workers and students get their own home sampler, so each group alone covers all home points of its comuna.
        let (mut hw, mut hst, mut js, mut ss) = (Sampler::new(&homes), Sampler::new(&homes), Sampler::new(&jobs), Sampler::new(&schools));
        let internal = |d: &str| d.starts_with("13");
        let workers: Vec<ModeOd> = rows::<ModeOd>(&data.join("mode_od.csv")).into_iter().filter(|r| internal(&r.destination_cut)).collect();
        let students: Vec<SchoolOd> = rows::<SchoolOd>(&data.join("school_od.csv")).into_iter().filter(|r| internal(&r.destination_cut)).collect();
        let mut persons = Vec::with_capacity(4_100_000);
        for r in &workers {
            if !hw.by_cut.contains_key(&r.origin_cut) || !js.by_cut.contains_key(&r.destination_cut) { continue; }
            for _ in 0..r.commuters as u32 {
                persons.push(Person { home: hw.take(&r.origin_cut), dest: js.take(&r.destination_cut), student: false, mode: r.mode });
            }
        }
        for r in &students {
            if !hst.by_cut.contains_key(&r.origin_cut) || !ss.by_cut.contains_key(&r.destination_cut) { continue; }
            for _ in 0..r.students as u32 {
                persons.push(Person { home: hst.take(&r.origin_cut), dest: ss.take(&r.destination_cut), student: true, mode: -1 });
            }
        }
        Population { homes, jobs, schools, persons }
    }
}
