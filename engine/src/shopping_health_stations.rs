//! Fit shopping and health Metro legs to the April OTROS arrivals at training stations.
use crate::stations::Station;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

pub type Day = BTreeMap<String, Vec<f64>>;

pub struct StationLegs {
    entries: Vec<[f64; 48]>,
    outward_exits: Vec<[f64; 48]>,
    metro_legs: usize,
    assigned_legs: usize,
}

impl StationLegs {
    pub fn new(stations: usize) -> Self {
        Self {
            entries: vec![[0.0; 48]; stations],
            outward_exits: vec![[0.0; 48]; stations],
            metro_legs: 0,
            assigned_legs: 0,
        }
    }

    /// All distinct entry-exit pairs share one leg. A same-parent pair receives zero.
    pub fn leg(
        &mut self,
        from: &[(usize, f64)],
        to: &[(usize, f64)],
        start: u8,
        end: u8,
        outward: bool,
    ) {
        self.metro_legs += 1;
        let mut total = 0.0;
        for &(entry, a) in from {
            for &(exit, b) in to {
                if entry != exit {
                    total += a * b;
                }
            }
        }
        if total == 0.0 {
            return;
        }
        self.assigned_legs += 1;
        for &(entry, a) in from {
            for &(exit, b) in to {
                if entry == exit {
                    continue;
                }
                let share = a * b / total;
                self.entries[entry][usize::from(start) % 48] += share;
                if outward {
                    self.outward_exits[exit][usize::from(end) % 48] += share;
                }
            }
        }
    }
}

#[derive(Serialize)]
pub struct StationFit {
    training_stations: usize,
    coverage_share: f64,
    pub multiplier: f64,
    metro_legs: usize,
    assigned_legs: usize,
    train_arrivals_scaled_target: f64,
    train_arrivals_fitted: f64,
    train_arrival_absolute_error_by_half_hour: Vec<f64>,
    entry_scale: Vec<f64>,
    replaced_share: Vec<f64>,
}

impl StationFit {
    pub fn fit(
        stations: &[Station],
        legs: &StationLegs,
        baseline: &Day,
        coverage_share: f64,
        arrivals: &Path,
    ) -> Self {
        let train: HashMap<_, _> = stations
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.holdout)
            .map(|(i, s)| (s.id.as_str(), i))
            .collect();
        let mut target = vec![[0.0; 48]; stations.len()];
        for row in csv::Reader::from_path(arrivals).unwrap().records() {
            let row = row.unwrap();
            let Some(&i) = train.get(&row[0]) else {
                continue;
            };
            if &row[2] != "OTROS" {
                continue;
            }
            let bin = row[3].parse::<usize>().unwrap() % 48;
            target[i][bin] += row[4].parse::<f64>().unwrap() * coverage_share;
        }

        let mut xy = 0.0;
        let mut xx = 0.0;
        let mut predicted_total = 0.0;
        let mut target_total = 0.0;
        let mut base_entries = [0.0; 48];
        let mut tour_entries = [0.0; 48];
        for (i, station) in stations.iter().enumerate().filter(|(_, s)| !s.holdout) {
            let x: f64 = legs.outward_exits[i].iter().sum();
            let y: f64 = target[i].iter().sum();
            xy += x * y;
            xx += x * x;
            predicted_total += x;
            target_total += y;
            for t in 0..48 {
                base_entries[t] += baseline[&station.id][t];
                tour_entries[t] += legs.entries[i][t];
            }
        }
        let multiplier = if xx > 0.0 { (xy / xx).max(0.0) } else { 0.0 };
        let mut error = vec![0.0; 48];
        for (i, _) in stations.iter().enumerate().filter(|(_, s)| !s.holdout) {
            for (t, bin) in error.iter_mut().enumerate() {
                *bin += (multiplier * legs.outward_exits[i][t] - target[i][t]).abs();
            }
        }
        let mut entry_scale = vec![0.0; 48];
        let mut replaced_share = vec![0.0; 48];
        for t in 0..48 {
            let b = base_entries[t];
            let c = multiplier * tour_entries[t];
            if b > 0.0 && c > 0.0 {
                entry_scale[t] = (b / c).min(1.0);
                replaced_share[t] = (entry_scale[t] * c / b).min(1.0);
            }
        }
        Self {
            training_stations: train.len(),
            coverage_share,
            multiplier,
            metro_legs: legs.metro_legs,
            assigned_legs: legs.assigned_legs,
            train_arrivals_scaled_target: target_total,
            train_arrivals_fitted: multiplier * predicted_total,
            train_arrival_absolute_error_by_half_hour: error,
            entry_scale,
            replaced_share,
        }
    }

    /// Reuse the coefficients fitted on the baseline when applying a scenario.
    pub fn apply(&self, stations: &[Station], legs: &StationLegs, baseline: &Day) -> Day {
        stations
            .iter()
            .enumerate()
            .map(|(i, station)| {
                let bins = (0..48)
                    .map(|t| {
                        (1.0 - self.replaced_share[t]) * baseline[&station.id][t]
                            + self.entry_scale[t] * self.multiplier * legs.entries[i][t]
                    })
                    .collect();
                (station.id.clone(), bins)
            })
            .collect()
    }
}
