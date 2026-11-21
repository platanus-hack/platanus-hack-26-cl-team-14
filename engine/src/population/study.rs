//! Weekday study roles on the baseline population. Resident IDs and schedules stay inside the engine.
use super::{Point, Population, Sampler, rows};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Purpose {
    Preschool,
    School,
    HigherEd,
}

#[derive(Deserialize)]
struct School {
    rbd: String,
    enrollment: u32,
    enrollment_no_preschool: u32,
}

#[derive(Deserialize, Hash, Eq, PartialEq)]
struct CampusKey {
    institution: String,
    campus: String,
    cut: String,
    year: String,
}

#[derive(Deserialize)]
struct BlockSector {
    block_id: String,
    sector: Option<usize>,
}

#[derive(Deserialize)]
struct HomeWeight {
    block_id: String,
    higher_18_24: Option<usize>,
}

#[derive(Deserialize)]
struct Census {
    cut: String,
    study_indicator: String,
    residents: usize,
    internal_workers: usize,
}

#[derive(Deserialize)]
struct Seed {
    home_sector: usize,
    destination_sector: usize,
    weighted_outbound_study_trips: f64,
}

#[derive(Deserialize)]
pub struct Timing {
    pub out_start_bin: u8,
    pub out_end_bin: u8,
    pub return_start_bin: u8,
    pub return_end_bin: u8,
    pub out_mode: u8,
    pub return_mode: u8,
}

#[derive(Default, Serialize)]
pub struct Coverage {
    // Arrays use preschool, school, higher education, in that order.
    pub roles_by_level: [usize; 3],
    pub distinct_workers_and_learners: usize,
    pub linked_workers_by_level: [usize; 3],
    pub unavailable_worker_at_home_by_level: [usize; 3],
    pub roles_without_home_sector_by_level: [usize; 3],
    pub sampled_tours_by_level: [usize; 3],
    pub unplaced_worker_tours_by_level: [usize; 3],
    pub placed_tours_by_level: [usize; 3],
    pub campus_enrollment_without_sector: usize,
    pub higher_home_attendance_without_sector: usize,
    pub higher_home_weights_suppressed: usize,
    pub higher_home_margin: usize,
    pub located_campus_margin: usize,
    pub higher_roles_in_unobserved_sector_pairs: usize,
}

struct Role {
    resident: u32,
    home: u32,
    destination: u32,
    timing: Option<std::num::NonZeroU16>,
}

pub struct StudyDay {
    pub coverage: Coverage,
    pub destinations: Vec<[f64; 2]>,
    roles: Vec<Role>,
    timings: Vec<Timing>,
}

type SectorMatrix<T> = [[T; 7]; 7];

// A fixed shuffle separates block, campus and survey-row order. No state comes from the host.
fn shuffle<T>(items: &mut [T], state: &mut u64) {
    for n in (1..items.len()).rev() {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        let j = ((*state as u128 * (n as u128 + 1)) >> 64) as usize;
        items.swap(n, j);
    }
}

// Round cumulative quotas to give exactly count slots across positive integer weights.
fn slots(points: &[(usize, usize)], count: usize) -> Vec<usize> {
    let total: usize = points.iter().map(|x| x.1).sum();
    let mut assigned = Vec::with_capacity(count);
    let mut sum = 0;
    for &(point, weight) in points {
        sum += weight;
        assigned.resize((sum * count + total / 2) / total, point);
    }
    assigned
}

fn pair_sectors(home: [usize; 7], campus: [usize; 7], seed: SectorMatrix<f64>) -> SectorMatrix<usize> {
    let n = home.iter().sum::<usize>() as f64;
    let mut matrix: SectorMatrix<f64> = std::array::from_fn(|h| {
        std::array::from_fn(|d| {
            if home[h] == 0 || campus[d] == 0 {
                return 0.0;
            }
            seed[h][d] + home[h] as f64 * campus[d] as f64 / (n * n)
        })
    });
    loop {
        for (h, row) in matrix.iter_mut().enumerate().filter(|(h, _)| home[*h] > 0) {
            let scale = home[h] as f64 / row.iter().sum::<f64>();
            for value in row {
                *value *= scale;
            }
        }
        for d in 0..7 {
            if campus[d] == 0 {
                continue;
            }
            let scale = campus[d] as f64 / matrix.iter().map(|r| r[d]).sum::<f64>();
            for row in &mut matrix {
                row[d] *= scale;
            }
        }
        if (0..7).all(|h| (matrix[h].iter().sum::<f64>() - home[h] as f64).abs() < 1e-7) {
            break;
        }
    }
    let mut out = matrix.map(|row| row.map(|v| v.floor() as usize));
    let mut left = home;
    let mut needed = campus;
    for h in 0..7 {
        for d in 0..7 {
            left[h] -= out[h][d];
            needed[d] -= out[h][d];
        }
    }
    // Place each residual in the cell with the largest remaining fraction and two free margins.
    while let Some((h, d)) = (0..7)
        .filter(|&h| left[h] > 0)
        .flat_map(|h| (0..7).map(move |d| (h, d)))
        .filter(|&(_, d)| needed[d] > 0)
        .max_by(|&(h, d), &(i, j)| {
            (matrix[h][d] - out[h][d] as f64).total_cmp(&(matrix[i][j] - out[i][j] as f64))
        })
    {
        out[h][d] += 1;
        left[h] -= 1;
        needed[d] -= 1;
    }
    out
}

struct Quota {
    rate: f64,
    count: usize,
}

impl Quota {
    fn take(&mut self) -> bool {
        let before = (self.count as f64 * self.rate).round();
        self.count += 1;
        (self.count as f64 * self.rate).round() > before
    }
}

fn table(data: &Path, file: &str) -> Vec<csv::StringRecord> {
    csv::Reader::from_path(data.join(file)).unwrap().records().map(Result::unwrap).collect()
}

fn purpose(s: &str) -> Option<Purpose> {
    match s {
        "preschool" => Some(Purpose::Preschool),
        "school" => Some(Purpose::School),
        "higher_ed" => Some(Purpose::HigherEd),
        _ => None,
    }
}

impl StudyDay {
    pub fn build(data: &Path, pop: &Population) -> Self {
        let mut coverage = Coverage::default();
        let by_block: HashMap<_, _> = rows::<BlockSector>(&data.join("eod_sectors/blocks.csv"))
            .into_iter()
            .map(|x| (x.block_id, x.sector))
            .collect();
        let sectors: Vec<_> = pop.homes.iter().map(|p| by_block[&p.id]).collect();

        let mut seed = 0x9e37_79b9_7f4a_7c15;
        let mut workers = vec![Vec::new(); pop.homes.len()];
        for (id, p) in pop.persons.iter().enumerate().filter(|(_, p)| !p.student) {
            workers[p.home as usize].push(id as u32);
        }
        for ids in &mut workers {
            shuffle(ids, &mut seed);
        }

        let mut margins = HashMap::new();
        for row in rows::<Census>(&data.join("census_study_by_comuna.csv")) {
            let level = match row.study_indicator.as_str() {
                "preschool_0_5" => Purpose::Preschool,
                "basic_6_13" | "secondary_14_17" => Purpose::School,
                "higher_18_24" => Purpose::HigherEd,
                _ => continue,
            };
            let (people, workers) = margins.entry((row.cut, level)).or_insert((0, 0));
            *people += row.residents;
            *workers += row.internal_workers;
        }
        let mut overlap: HashMap<_, _> = margins
            .into_iter()
            .map(|(key, (people, workers))| (key, Quota { rate: workers as f64 / people as f64, count: 0 }))
            .collect();
        let mut rates = HashMap::new();
        for r in table(data, "eod_tours/rates.csv") {
            if let Some(p) = purpose(&r[0]) {
                rates.insert(
                    (p, r[1].parse::<usize>().unwrap()),
                    Quota { rate: r[5].parse().unwrap(), count: 0 },
                );
            }
        }
        let mut profiles = [const { Vec::<(f64, usize)>::new() }; 3];
        let mut timings = Vec::new();
        for r in table(data, "eod_tours/time_mode.csv") {
            let Some(p) = purpose(&r[0]) else { continue };
            let bins = |i: usize| r[i].parse().unwrap();
            let cumulative = profiles[p as usize].last().map_or(0.0, |x| x.0) + r[8].parse::<f64>().unwrap();
            profiles[p as usize].push((cumulative, timings.len()));
            timings.push(Timing {
                out_start_bin: bins(1),
                out_end_bin: bins(2),
                return_start_bin: bins(3),
                return_end_bin: bins(4),
                out_mode: bins(5),
                return_mode: bins(6),
            });
        }
        let mut roles = Vec::new();
        let mut next_time = [0usize; 3];
        let mut add = |home: usize, destination: usize, lone_id: usize, p: Purpose, cov: &mut Coverage| {
            let k = p as usize;
            cov.roles_by_level[k] += 1;
            let mut dual = false;
            let mut id = lone_id;
            if overlap.get_mut(&(pop.homes[home].cut.clone(), p)).unwrap().take() {
                if let Some(worker) = workers[home].pop() {
                    dual = true;
                    id = worker as usize;
                    cov.linked_workers_by_level[k] += 1;
                } else {
                    cov.unavailable_worker_at_home_by_level[k] += 1;
                }
            }
            let mut timing = None;
            if let Some(sector) = sectors[home] {
                if rates.get_mut(&(p, sector)).unwrap().take() {
                    cov.sampled_tours_by_level[k] += 1;
                    let shape = &profiles[k];
                    let at = (((next_time[k] as f64 + 0.5) * 0.7548776662466927).fract())
                        * shape.last().unwrap().0;
                    next_time[k] += 1;
                    if dual {
                        // The work layer publishes departures only. A free return interval cannot be established.
                        cov.unplaced_worker_tours_by_level[k] += 1;
                    } else {
                        cov.placed_tours_by_level[k] += 1;
                        let i = shape[shape.partition_point(|s| s.0 <= at)].1;
                        timing = std::num::NonZeroU16::new(u16::try_from(i + 1).unwrap());
                    }
                }
            } else {
                cov.roles_without_home_sector_by_level[k] += 1;
            }
            roles.push(Role {
                resident: id as u32,
                home: home as u32,
                destination: destination as u32,
                timing,
            });
        };

        let schools: HashMap<_, _> =
            rows::<School>(&data.join("schools.csv")).into_iter().map(|s| (s.rbd.clone(), s)).collect();
        let mut preschool = Vec::new();
        let points: Vec<_> = pop
            .schools
            .iter()
            .map(|s| {
                let seats = &schools[&s.id];
                preschool.push(Quota {
                    rate: (seats.enrollment - seats.enrollment_no_preschool) as f64 / seats.enrollment as f64,
                    count: 0,
                });
                Point {
                    id: s.id.clone(),
                    cut: s.cut.clone(),
                    latitude: s.latitude,
                    longitude: s.longitude,
                    weight: seats.enrollment as f64,
                }
            })
            .collect();
        let mut destinations: Vec<_> = points.iter().map(|p| [p.longitude, p.latitude]).collect();
        let mut school_points = Sampler::new(&points, std::f64::consts::SQRT_2 - 1.0);
        for (id, person) in pop.persons.iter().enumerate().filter(|(_, p)| p.student) {
            let destination = school_points.take(&pop.schools[person.dest as usize].cut) as usize;
            let p = if preschool[destination].take() { Purpose::Preschool } else { Purpose::School };
            add(person.home as usize, destination, id, p, &mut coverage);
        }

        let weights: HashMap<_, _> = rows::<HomeWeight>(&data.join("higher_ed_home_blocks.csv"))
            .into_iter()
            .map(|r| (r.block_id, r.higher_18_24))
            .collect();
        let mut homes = Vec::new();
        for (i, block) in pop.homes.iter().enumerate() {
            match (weights[&block.id], sectors[i]) {
                (Some(n), Some(_)) if n > 0 => homes.push((i, n)),
                (Some(n), None) => coverage.higher_home_attendance_without_sector += n,
                (None, _) => coverage.higher_home_weights_suppressed += 1,
                _ => {}
            }
        }
        let mut campus_sectors = HashMap::new();
        for r in table(data, "eod_sectors/higher_ed.csv") {
            let key = CampusKey {
                institution: r[0].into(),
                campus: r[1].into(),
                cut: r[2].into(),
                year: r[3].into(),
            };
            campus_sectors.insert(key, r[5].parse::<usize>().ok());
        }
        let mut campuses = Vec::new();
        let mut destinations_by_sector = HashMap::new();
        for r in table(data, "higher_ed.csv") {
            let key = CampusKey {
                institution: r[0].into(),
                campus: r[1].into(),
                cut: r[2].into(),
                year: r[6].into(),
            };
            let n: usize = r[7].parse().unwrap();
            if let Some(sector) = campus_sectors[&key] {
                let id = destinations.len();
                if n > 0 {
                    campuses.push((id, n));
                }
                destinations_by_sector.insert(id, sector);
                destinations.push([r[3].parse().unwrap(), r[4].parse().unwrap()]);
            } else {
                coverage.campus_enrollment_without_sector += n;
            }
        }
        coverage.higher_home_margin = homes.iter().map(|p| p.1).sum();
        coverage.located_campus_margin = campuses.iter().map(|p| p.1).sum();
        let count = coverage.higher_home_margin.min(coverage.located_campus_margin);
        let mut home_slots = [const { Vec::new() }; 7];
        let mut campus_slots = [const { Vec::new() }; 7];
        for h in slots(&homes, count) {
            home_slots[sectors[h].unwrap() - 1].push(h);
        }
        for d in slots(&campuses, count) {
            campus_slots[destinations_by_sector[&d] - 1].push(d);
        }
        for group in home_slots.iter_mut().chain(&mut campus_slots) {
            shuffle(group, &mut seed);
        }

        let mut evidence = [[0.0; 7]; 7];
        for r in rows::<Seed>(&data.join("eod_tours/higher_ed_seed.csv")) {
            evidence[r.home_sector - 1][r.destination_sector - 1] = r.weighted_outbound_study_trips;
        }
        let pairs = pair_sectors(
            home_slots.each_ref().map(Vec::len),
            campus_slots.each_ref().map(Vec::len),
            evidence,
        );
        let mut id = pop.persons.len();
        for (h, row) in pairs.into_iter().enumerate() {
            for (d, n) in row.into_iter().enumerate() {
                if evidence[h][d] == 0.0 {
                    coverage.higher_roles_in_unobserved_sector_pairs += n;
                }
                for _ in 0..n {
                    add(
                        home_slots[h].pop().unwrap(),
                        campus_slots[d].pop().unwrap(),
                        id,
                        Purpose::HigherEd,
                        &mut coverage,
                    );
                    id += 1;
                }
            }
        }
        let workers = pop.persons.iter().take_while(|p| !p.student).count();
        coverage.distinct_workers_and_learners =
            workers + roles.iter().filter(|r| r.resident as usize >= workers).count();
        Self { coverage, destinations, roles, timings }
    }

    pub fn visit_tours(&self, mut visit: impl FnMut(usize, usize, &Timing)) {
        for role in &self.roles {
            if let Some(i) = role.timing {
                visit(role.home as usize, role.destination as usize, &self.timings[i.get() as usize - 1]);
            }
        }
    }
}
