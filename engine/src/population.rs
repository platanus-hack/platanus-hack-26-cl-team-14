#![allow(dead_code)] // fields used by other modules (choice, city)
//! Synthetic person-level state of the RM (gentle-fox 11:52). One record per RM-internal commuter and student.
//! Built deterministically from margins only: `mode_od.csv` (commuters by home comuna, work comuna and mode),
//! `school_od.csv` (students by home and school comuna), `blocks.csv` (home weights), `jobs.csv` (work weights),
//! `schools.csv` (`enrollment_no_preschool`, school weights), `higher_ed.csv` (SIES campuses), and age and household size from
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
/// `UNKNOWN` marks a missing trait, including household size for people outside private households.
pub const UNKNOWN: u8 = u8::MAX;

pub struct Point { pub id: String, pub cut: String, pub longitude: f64, pub latitude: f64, pub weight: f64 }

/// A resident who is neither an internal commuter nor a school student (`person_traits_residents.csv`).
/// role: 0 under 15, 1 higher education, 2 employed without an internal commute, 3 unemployed,
/// 4 outside the labour force, 5 unknown. No trip is modeled for them yet. Higher-education residents who
/// travel to a campus become students (see `Population::build`); the rest stay here with role 1.
#[derive(Clone, Copy, Deserialize)]
pub struct Resident { pub home: u32, pub role: u8, pub age_band: u8, pub household_size: u8 }

pub struct Population { pub homes: Vec<Point>, pub jobs: Vec<Point>, pub schools: Vec<Point>, pub persons: Vec<Person>, pub residents: Vec<Resident> }

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
struct ResidentTraits { cut: String, role: u8, age_band: u8, household_size: u8, persons: f64 }
#[derive(Deserialize)]
struct Campus { institution: String, campus: String, cut: String, longitude: Option<f64>, latitude: Option<f64>, enrollment: f64, in_person: f64, hybrid: f64 }
#[derive(Deserialize)]
struct Commune { cut: String, longitude: f64, latitude: f64 }
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
        let trait_order = 5_f64.sqrt() - 2.0;
        for cell in workers.chunk_by(|a, b| (&a.origin_cut, &a.destination_cut, a.mode) == (&b.origin_cut, &b.destination_cut, b.mode)) {
            let Some(home) = hw.get_mut(cell[0].origin_cut.as_str()) else { continue };
            let Some(dest) = js.get_mut(cell[0].destination_cut.as_str()) else { continue };
            // The file lists a cell by age, then household. Interleave them, so person order inside a cell
            // (which rechoose.rs uses to pick who moves) does not follow age or household.
            let mut traits: Vec<(f64, u8, u8)> = cell.iter().flat_map(|r| (0..r.commuters as u32).map(|_| (r.age_band, r.household_size)))
                .enumerate().map(|(i, (a, h))| (((i as f64 + 0.5) * trait_order).fract(), a, h)).collect();
            traits.sort_by(|x, y| x.0.total_cmp(&y.0));
            for (_, age_band, household_size) in traits {
                persons.push(Person { home: home.take(home_step), dest: dest.take(destination_step), student: false, mode: cell[0].mode,
                    age_band, household_size });
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
        // Other residents get their own home sampler, so they also cover all home points of their comuna.
        let mut hr = Sampler::by_cut(&homes);
        let mut residents = Vec::with_capacity(3_700_000);
        let others: Vec<ResidentTraits> = rows(&data.join("person_traits_residents.csv"));
        for cut in others.chunk_by(|a, b| a.cut == b.cut) {
            let Some(home) = hr.get_mut(cut[0].cut.as_str()) else { continue };
            // The file lists a comuna by role, age and household. Interleave them, as for workers.
            let mut traits: Vec<(f64, &ResidentTraits)> = cut.iter().flat_map(|r| (0..r.persons as u32).map(move |_| r))
                .enumerate().map(|(i, r)| (((i as f64 + 0.5) * trait_order).fract(), r)).collect();
            traits.sort_by(|x, y| x.0.total_cmp(&y.0));
            residents.extend(traits.into_iter().map(|(_, r)| Resident { home: home.take(home_step), role: r.role, age_band: r.age_band, household_size: r.household_size }));
        }
        // Higher education. Unverified assumption: the in-person + hybrid share of SIES 2026 registrations (69.6 %) is
        // the share of Census higher-education residents who travel on a weekday. That share goes to a campus of `higher_ed.csv`, drawn region-wide in proportion to its
        // in-person + hybrid registrations. A campus without a point sits at its comuna centroid (`communes.csv`).
        let mut schools = schools;
        let centroid: HashMap<String, (f64, f64)> = rows::<Commune>(&data.join("communes.csv")).into_iter().map(|c| (c.cut, (c.longitude, c.latitude))).collect();
        let campuses: Vec<Campus> = rows(&data.join("higher_ed.csv"));
        let first = schools.len();
        schools.extend(campuses.iter().map(|c| {
            let (longitude, latitude) = c.longitude.zip(c.latitude).unwrap_or(centroid[&c.cut]);
            Point { id: format!("{}|{}", c.institution, c.campus), cut: c.cut.clone(), longitude, latitude, weight: c.in_person + c.hybrid }
        }));
        let pool: Vec<Point> = schools[first..].iter().map(|p| Point { id: String::new(), cut: String::new(), longitude: 0.0, latitude: 0.0, weight: p.weight }).collect();
        let mut campus = Sampler::by_cut(&pool).remove("").unwrap();
        let travel = pool.iter().map(|p| p.weight).sum::<f64>() / campuses.iter().map(|c| c.enrollment).sum::<f64>();
        let campus_step = std::f64::consts::E.fract();
        let mut k = 0.0;
        residents.retain(|r| {
            if r.role != 1 { return true; }
            k += 1.0;
            // Systematic selection keeps exactly the travelling share, spread evenly over the comuna-ordered list.
            if (k * travel).floor() == ((k - 1.0) * travel).floor() { return true; }
            persons.push(Person { home: r.home, dest: (first + campus.take(campus_step) as usize) as u32, student: true, mode: -1,
                age_band: r.age_band, household_size: r.household_size });
            false
        });
        Population { homes, jobs, schools, persons, residents }
    }
}

#[path = "population/trips.rs"]
mod trips;
#[allow(unused_imports)] // Each binary imports only the population outputs it needs.
pub use trips::{TripDelta, TripGroup, TripPlan, person_departure, sample_trips, trip_deltas};

#[path = "population/fine_modes.rs"]
mod fine_modes;
#[allow(unused_imports)]
pub use fine_modes::{FineModes, NEW_OTHER_MODE};
