#![allow(dead_code)] // fields used by other modules (choice, city)
//! Synthetic person-level state of the RM (gentle-fox 11:52). One record per RM-internal commuter and student.
//! Built deterministically from margins only: `mode_od.csv` (commuters by home comuna, work comuna and mode),
//! `school_od.csv` (students by home and school comuna), `blocks.csv` (home weights), `jobs.csv` (work weights),
//! `schools.csv` (`enrollment_no_preschool`, school weights), and age and household size from
//! `person_traits_workers.csv` / `person_traits_students.csv` (Census 2024 person records, `tools/pack/person_traits.py`). Inside a comuna, persons are placed on points by
//! systematic sampling of the cumulative weights, so comuna flows are exact and point totals follow the weights.
//! Label: "estado sintético a nivel de persona, consistente con los márgenes del Censo, no microdatos" — the
//! joint of home point, destination point and mode is not observed in any file.
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Copy)]
pub struct Person { pub home: u32, pub dest: u32, pub student: bool, pub mode: i8, pub age_band: u8, pub household_size: u8 }

/// Age band: 0 under 15, 1 15-24, 2 25-44, 3 45-64, 4 65+. Household size: 1-5, 5 = 5 or more.
/// Workers carry the observed Census 2024 joint of home comuna, work comuna, mode, age and household
/// (`person_traits_workers.csv`). Students draw both from Census students of their home comuna.
/// `UNKNOWN` marks persons a scenario adds, whose traits no source gives.
pub const UNKNOWN: u8 = u8::MAX;

pub struct Point { pub id: String, pub cut: String, pub longitude: f64, pub latitude: f64, pub weight: f64 }

pub struct Population { pub homes: Vec<Point>, pub jobs: Vec<Point>, pub schools: Vec<Point>, pub persons: Vec<Person> }

#[derive(Deserialize)]
struct Block { block_id: String, cut: String, longitude: f64, latitude: f64, population: f64 }
#[derive(Deserialize)]
struct Job { block_id: String, cut: String, longitude: f64, latitude: f64, jobs: f64 }
#[derive(Deserialize)]
struct School { rbd: String, cut: String, longitude: f64, latitude: f64, enrollment_no_preschool: f64 }
#[derive(Deserialize)]
struct WorkerTraits { origin_cut: String, destination_cut: String, mode: i8, age_band: u8, household_size: u8, commuters: f64 }
#[derive(Deserialize)]
struct StudentTraits { cut: String, age_band: u8, household_size: u8, students: f64 }
#[derive(Deserialize)]
struct SchoolOd { origin_cut: String, destination_cut: String, students: f64 }

fn rows<T: for<'de> Deserialize<'de>>(p: &Path) -> Vec<T> {
    csv::Reader::from_path(p).unwrap().deserialize().map(|r| r.unwrap()).collect()
}

/// Cumulative point weights and the sampling position in one comuna.
#[derive(Default)]
struct Sampler { ids: Vec<u32>, cumulative: Vec<f64>, next: f64 }

impl Sampler {
    fn by_cut(points: &[Point]) -> HashMap<&str, Sampler> {
        let mut by_cut: HashMap<&str, Sampler> = HashMap::new();
        for (i, p) in points.iter().enumerate().filter(|(_, p)| p.weight > 0.0) {
            let sampler = by_cut.entry(p.cut.as_str()).or_default();
            let cumulative = sampler.cumulative.last().copied().unwrap_or(0.0) + p.weight;
            sampler.ids.push(i as u32);
            sampler.cumulative.push(cumulative);
        }
        by_cut
    }

    fn take(&mut self, step: f64) -> u32 {
        let total = *self.cumulative.last().unwrap();
        let at = ((self.next + 0.5) * step).fract() * total;
        self.next += 1.0;
        self.ids[self.cumulative.partition_point(|&c| c <= at).min(self.ids.len() - 1)]
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
        // Different irrational steps keep home and destination ranks from moving together inside each OD row.
        let home_step = 0.618_033_988_749_894_9;
        let destination_step = std::f64::consts::SQRT_2 - 1.0;
        let (mut hw, mut hst) = (Sampler::by_cut(&homes), Sampler::by_cut(&homes));
        let (mut js, mut ss) = (Sampler::by_cut(&jobs), Sampler::by_cut(&schools));
        let internal = |d: &str| d.starts_with("13");
        // Same cells and order as mode_od.csv, split by age and household, so the samplers take the same points.
        let workers: Vec<WorkerTraits> = rows::<WorkerTraits>(&data.join("person_traits_workers.csv")).into_iter().filter(|r| internal(&r.destination_cut)).collect();
        let students: Vec<SchoolOd> = rows::<SchoolOd>(&data.join("school_od.csv")).into_iter().filter(|r| internal(&r.destination_cut)).collect();
        let mut persons = Vec::with_capacity(4_100_000);
        for r in &workers {
            let Some(home) = hw.get_mut(r.origin_cut.as_str()) else { continue };
            let Some(dest) = js.get_mut(r.destination_cut.as_str()) else { continue };
            for _ in 0..r.commuters as u32 {
                persons.push(Person { home: home.take(home_step), dest: dest.take(destination_step), student: false, mode: r.mode,
                    age_band: r.age_band, household_size: r.household_size });
            }
        }
        let traits: Vec<StudentTraits> = rows(&data.join("person_traits_students.csv"));
        let trait_points: Vec<Point> = traits.iter()
            .map(|t| Point { id: String::new(), cut: t.cut.clone(), longitude: 0.0, latitude: 0.0, weight: t.students }).collect();
        let mut ts = Sampler::by_cut(&trait_points);
        let trait_step = 3_f64.sqrt() - 1.0;
        for r in &students {
            let Some(home) = hst.get_mut(r.origin_cut.as_str()) else { continue };
            let Some(dest) = ss.get_mut(r.destination_cut.as_str()) else { continue };
            let t = ts.get_mut(r.origin_cut.as_str()).unwrap();
            for _ in 0..r.students as u32 {
                let s = &traits[t.take(trait_step) as usize];
                persons.push(Person { home: home.take(home_step), dest: dest.take(destination_step), student: true, mode: -1,
                    age_band: s.age_band, household_size: s.household_size });
            }
        }
        Population { homes, jobs, schools, persons }
    }
}

#[path = "population/trips.rs"]
mod trips;
#[allow(unused_imports)] // Each binary imports only the population outputs it needs.
pub use trips::{TripDelta, TripGroup, TripPlan, sample_trips, trip_deltas};

#[path = "population/fine_modes.rs"]
mod fine_modes;
#[allow(unused_imports)]
pub use fine_modes::{FineModes, NEW_OTHER_MODE};
