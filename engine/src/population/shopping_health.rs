//! A retained normal-weekday survey tour is a home → activity → home pair.
//! Census resident counts include workers and learners. The work and study schedules
//! are not jointly observed with these tours.
use super::{Point, rows};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const SHOPPING: usize = 0;
const HEALTH: usize = 1;

fn purpose(name: &str) -> Option<usize> {
    match name {
        "shopping" => Some(SHOPPING),
        "health" => Some(HEALTH),
        _ => None,
    }
}

#[derive(Deserialize)]
struct BlockSector {
    block_id: String,
    sector: Option<usize>,
}

#[derive(Deserialize)]
struct RateRow {
    purpose: String,
    home_sector: usize,
    tours_per_eligible_person: f64,
}

#[derive(Deserialize)]
struct DestinationRow {
    purpose: String,
    home_cut: String,
    destination_cut: Option<String>,
    destination_share: f64,
}

#[derive(Deserialize)]
struct Place {
    kind: String,
    longitude: f64,
    latitude: f64,
    cut: String,
}

#[derive(Deserialize)]
pub struct Timing {
    purpose: String,
    weighted_tours: f64,
    pub out_start_bin: u8,
    pub out_end_bin: u8,
    pub return_start_bin: u8,
    pub return_end_bin: u8,
    pub out_mode: u8,
    pub return_mode: u8,
}

#[derive(Default, Serialize)]
pub struct PurposeCoverage {
    residents_without_rate: usize,
    sampled_tours: usize,
    tours_without_resident_slot: usize,
    tours_without_destination_distribution: usize,
    tours_with_unknown_destination_cut: usize,
    tours_without_mapped_object: usize,
    placed_tours: usize,
}

#[derive(Default, Serialize)]
pub struct Coverage {
    residents: usize,
    residents_without_home_sector: usize,
    shopping: PurposeCoverage,
    health: PurposeCoverage,
}

// Use cumulative rounding per sector. A small block can contribute a tour even if
// its own expected count is below one.
struct Rate {
    per_resident: f64,
    residents: usize,
}

impl Rate {
    fn take(&mut self, residents: usize) -> usize {
        let before = (self.residents as f64 * self.per_resident).round() as usize;
        self.residents += residents;
        (self.residents as f64 * self.per_resident).round() as usize - before
    }
}

#[derive(Default)]
struct Distribution<T>(Vec<(f64, T)>);

impl<T> Distribution<T> {
    fn add(&mut self, weight: f64, value: T) {
        let sum = self.0.last().map_or(0.0, |row| row.0) + weight;
        self.0.push((sum, value));
    }

    fn pick(&self, fraction: f64) -> &T {
        let at = fraction * self.0.last().unwrap().0;
        &self.0[self.0.partition_point(|row| row.0 <= at)].1
    }
}

// SplitMix64 gives independent draws from a fixed block, purpose, and slot.
fn draw(state: &mut u64) -> f64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut x = *state;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((x ^ (x >> 31)) >> 11) as f64 / (1_u64 << 53) as f64
}

struct Tour {
    home: u32,
    destination: u32,
    timing: u32,
}

pub struct ShoppingHealthDay {
    pub destinations: Vec<[f64; 2]>,
    pub coverage: Coverage,
    timings: Vec<Timing>,
    tours: Vec<Tour>,
}

impl ShoppingHealthDay {
    pub fn build(data: &Path, homes: &[Point]) -> Self {
        let sectors: HashMap<_, _> = rows::<BlockSector>(&data.join("eod_sectors/blocks.csv"))
            .into_iter()
            .map(|row| (row.block_id, row.sector))
            .collect();

        let mut rates = HashMap::new();
        for row in rows::<RateRow>(&data.join("eod_tours/rates.csv")) {
            if let Some(p) = purpose(&row.purpose) {
                rates.insert(
                    (p, row.home_sector),
                    Rate {
                        per_resident: row.tours_per_eligible_person,
                        residents: 0,
                    },
                );
            }
        }

        let mut by_home: [HashMap<String, Distribution<Option<String>>>; 2] = Default::default();
        for row in rows::<DestinationRow>(&data.join("eod_destinations.csv")) {
            if let Some(p) = purpose(&row.purpose) {
                by_home[p]
                    .entry(row.home_cut)
                    .or_default()
                    .add(row.destination_share, row.destination_cut);
            }
        }

        let mut destinations = Vec::new();
        let mut places: [HashMap<String, Vec<u32>>; 2] = Default::default();
        for row in rows::<Place>(&data.join("activity_points.csv")) {
            let p = match row.kind.as_str() {
                "shop" | "supermarket" | "mall" => SHOPPING,
                "hospital" | "clinic" => HEALTH,
                _ => continue,
            };
            places[p]
                .entry(row.cut)
                .or_default()
                .push(destinations.len() as u32);
            destinations.push([row.longitude, row.latitude]);
        }

        let mut timings = Vec::new();
        let mut profiles: [Distribution<u32>; 2] = Default::default();
        for row in rows::<Timing>(&data.join("eod_tours/time_mode.csv")) {
            if let Some(p) = purpose(&row.purpose) {
                profiles[p].add(row.weighted_tours, timings.len() as u32);
                timings.push(row);
            }
        }

        let mut coverage = Coverage::default();
        let mut tours = Vec::new();
        for (home, block) in homes.iter().enumerate() {
            let residents = block.weight as usize;
            coverage.residents += residents;
            let Some(sector) = sectors.get(&block.id).copied().flatten() else {
                coverage.residents_without_home_sector += residents;
                continue;
            };
            // Distinct slots within the block prevent one resident from receiving both tours.
            let mut free = residents;
            for p in [SHOPPING, HEALTH] {
                let counter = if p == SHOPPING {
                    &mut coverage.shopping
                } else {
                    &mut coverage.health
                };
                let Some(rate) = rates.get_mut(&(p, sector)) else {
                    counter.residents_without_rate += residents;
                    continue;
                };
                let count = rate.take(residents);
                let assigned = count.min(free);
                free -= assigned;
                counter.sampled_tours += count;
                counter.tours_without_resident_slot += count - assigned;
                let Some(distribution) = by_home[p].get(&block.cut) else {
                    counter.tours_without_destination_distribution += assigned;
                    continue;
                };
                for slot in 0..assigned {
                    let mut state = (home as u64) << 32 | (p as u64) << 31 | slot as u64;
                    let Some(cut) = distribution.pick(draw(&mut state)) else {
                        counter.tours_with_unknown_destination_cut += 1;
                        continue;
                    };
                    let Some(pool) = places[p].get(cut) else {
                        counter.tours_without_mapped_object += 1;
                        continue;
                    };
                    let destination = pool[(draw(&mut state) * pool.len() as f64) as usize];
                    let timing = *profiles[p].pick(draw(&mut state));
                    tours.push(Tour {
                        home: home as u32,
                        destination,
                        timing,
                    });
                    counter.placed_tours += 1;
                }
            }
        }
        Self {
            destinations,
            coverage,
            timings,
            tours,
        }
    }

    pub fn visit_tours(&self, mut visit: impl FnMut(usize, usize, &Timing)) {
        for tour in &self.tours {
            visit(
                tour.home as usize,
                tour.destination as usize,
                &self.timings[tour.timing as usize],
            );
        }
    }
}
