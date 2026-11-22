#![allow(dead_code)] // fields used by other binaries (choice_fit, choice_scenario)
//! Comuna-level choice rules of a worker: where to work (gravity) and how to travel (3-way logit).
//! Each rule is fitted on the baseline and keeps one residual factor per OD pair. Thus the baseline
//! call returns the observed Census 2024 flows and shares, and a change in jobs or minutes moves them.
//! All matrices are n×n, row = home comuna, column = work comuna, index = row of communes.csv.
use std::path::Path;

// Mode order of every `[f64; 3]`: public transport (census code 2), car (code 1), all other codes.

#[derive(Clone)]
pub struct Choice {
    pub n: usize,
    pub cuts: Vec<String>,
    pub names: Vec<String>,
    /// Baseline jobs by comuna (jobs.csv; equal to the destination totals of work_od.csv).
    pub jobs: Vec<f64>,
    /// Baseline public-transport door-to-door minutes. travel_times.csv, or the distance fit where it has no row.
    pub minutes: Vec<f64>,
    /// Pairs with an available GTFS estimate before distance-fit filling.
    pub gtfs_minutes: Vec<bool>,
    /// Number of pairs that have minutes from the distance fit (the diagonal is included).
    pub filled: usize,
    /// Observed workers (work_od.csv, destinations in the RM only, as main.rs).
    pub flows: Vec<f64>,
    /// Observed commuters by mode (mode_od.csv).
    pub mode_flows: Vec<[f64; 3]>,
    /// Destination utility loses `beta` for each minute.
    pub beta: f64,
    /// Straight-line km between comuna points. Intra-comuna: half the distance to the nearest comuna.
    pub km: Vec<f64>,
    /// Mode utility, car = 0. Public = [0] + [1] × minutes + [2] × ln km. Other = [3] + [4] × ln km.
    /// ln km stands for the car time and the walk distance, which the data pack does not have.
    pub mode_coef: [f64; 5],
    /// Residuals: observed share / share of the pure rule.
    pub k: Vec<f64>,
    pub mode_k: Vec<[f64; 3]>,
}

fn records(p: &Path) -> impl Iterator<Item = csv::StringRecord> {
    csv::Reader::from_path(p).unwrap().into_records().map(|r| r.unwrap())
}

/// P(d | o) ∝ jobs_d × exp(−beta × minutes_od) × k_od. Rows sum to 1.
pub fn gravity(n: usize, jobs: &[f64], minutes: &[f64], beta: f64, k: &[f64]) -> Vec<f64> {
    let mut p: Vec<f64> = (0..n * n).map(|i| jobs[i % n] * (-beta * minutes[i]).exp() * k[i]).collect();
    for row in p.chunks_mut(n) {
        let s: f64 = row.iter().sum();
        row.iter_mut().for_each(|x| *x /= s);
    }
    p
}

/// Mode shares [public, car, other] of one pair: P(j) ∝ exp(utility_j) × k_j.
pub fn shares(c: &[f64; 5], minutes: f64, km: f64, k: &[f64; 3]) -> [f64; 3] {
    let p = [(c[0] + c[1] * minutes + c[2] * km.ln()).exp() * k[0], k[1], (c[3] + c[4] * km.ln()).exp() * k[2]];
    p.map(|x| x / (p[0] + p[1] + p[2]))
}

impl Choice {
    pub fn load(data: &Path) -> Self {
        let com: Vec<_> = records(&data.join("communes.csv")).collect();
        let n = com.len();
        let cuts: Vec<String> = com.iter().map(|r| r[0].to_string()).collect();
        let ix = |c: &str| cuts.iter().position(|x| x == c).unwrap();
        let line = |a: usize, b: usize| {
            let f = |r: &csv::StringRecord, i| r[i].parse::<f64>().unwrap();
            ((f(&com[a], 5) - f(&com[b], 5)) * 92.9).hypot((f(&com[a], 6) - f(&com[b], 6)) * 110.6) // km per degree at 33.45° S
        };
        let km: Vec<f64> = (0..n * n).map(|i| match (i / n, i % n) {
            (o, d) if o == d => (0..n).filter(|&e| e != o).map(|e| line(o, e)).fold(f64::INFINITY, f64::min) / 2.0,
            (o, d) => line(o, d),
        }).collect();

        // Minutes: the GTFS matrix, then a straight-line fit minutes = a + b × km on its own pairs
        // for the pairs it does not have.
        let mut minutes = vec![f64::NAN; n * n];
        let mut gtfs_minutes = vec![false; n * n];
        let (mut m, mut sx, mut sy, mut sxx, mut sxy) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for r in records(&data.join("travel_times.csv")) {
            let (o, d, t) = (ix(&r[0]), ix(&r[1]), r[2].parse::<f64>().unwrap());
            minutes[o * n + d] = t;
            gtfs_minutes[o * n + d] = t.is_finite();
            let x = km[o * n + d];
            (m, sx, sy, sxx, sxy) = (m + 1.0, sx + x, sy + t, sxx + x * x, sxy + x * t);
        }
        let slope = (m * sxy - sx * sy) / (m * sxx - sx * sx);
        let intercept = (sy - slope * sx) / m;
        for i in 0..n * n {
            if minutes[i].is_nan() { minutes[i] = intercept + slope * km[i]; }
        }

        let mut jobs = vec![0.0; n];
        for r in records(&data.join("jobs.csv")) {
            jobs[ix(&r[1])] += r[4].parse::<f64>().unwrap();
        }
        let mut flows = vec![0.0; n * n];
        for r in records(&data.join("work_od.csv")).filter(|r| r[1].starts_with("13")) {
            flows[ix(&r[0]) * n + ix(&r[1])] += r[2].parse::<f64>().unwrap();
        }
        let mut mode_flows = vec![[0.0; 3]; n * n];
        for r in records(&data.join("mode_od.csv")).filter(|r| r[1].starts_with("13")) {
            mode_flows[ix(&r[0]) * n + ix(&r[1])][match &r[2] { "2" => 0, "1" => 1, _ => 2 }] += r[3].parse::<f64>().unwrap();
        }

        // beta: maximum likelihood, ternary search (the log-likelihood is concave in beta).
        let ones = vec![1.0; n * n];
        let ll = |b: f64| gravity(n, &jobs, &minutes, b, &ones).iter().zip(&flows).map(|(p, f)| f * p.ln()).sum::<f64>();
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..50 {
            let (a, b) = (lo + (hi - lo) / 3.0, hi - (hi - lo) / 3.0);
            if ll(a) < ll(b) { lo = a } else { hi = b }
        }
        let beta = (lo + hi) / 2.0;
        let pure = gravity(n, &jobs, &minutes, beta, &ones);
        let k = (0..n * n).map(|i| flows[i] / flows[i / n * n..][..n].iter().sum::<f64>() / pure[i]).collect();

        // Mode logit: maximum likelihood by Newton steps.
        let mut mode_coef = [0.0; 5];
        for _ in 0..25 {
            let (mut g, mut h) = ([0.0; 5], [[0.0; 5]; 5]);
            for i in 0..n * n {
                let (w, l) = (mode_flows[i], km[i].ln());
                let (p, tot) = (shares(&mode_coef, minutes[i], km[i], &[1.0; 3]), w[0] + w[1] + w[2]);
                let x = [[1.0, minutes[i], l, 0.0, 0.0], [0.0; 5], [0.0, 0.0, 0.0, 1.0, l]]; // d utility_j / d coef
                let mean: [f64; 5] = std::array::from_fn(|a| p[0] * x[0][a] + p[2] * x[2][a]);
                for a in 0..5 {
                    g[a] += w[0] * x[0][a] + w[2] * x[2][a] - tot * mean[a];
                    for b in 0..5 {
                        h[a][b] += tot * (p[0] * x[0][a] * x[0][b] + p[2] * x[2][a] * x[2][b] - mean[a] * mean[b]);
                    }
                }
            }
            for i in 0..5 { // solve h × step = g
                for r in (0..5).filter(|&r| r != i) {
                    let f = h[r][i] / h[i][i];
                    for c in 0..5 { h[r][c] -= f * h[i][c]; }
                    g[r] -= f * g[i];
                }
            }
            for i in 0..5 { mode_coef[i] += g[i] / h[i][i]; }
        }
        let mode_k = (0..n * n).map(|i| {
            let (w, p) = (mode_flows[i], shares(&mode_coef, minutes[i], km[i], &[1.0; 3]));
            let tot = w[0] + w[1] + w[2];
            if tot > 0.0 { [0, 1, 2].map(|j| w[j] / tot / p[j]) } else { [1.0; 3] } // a pair with no commuters uses the pure rule
        }).collect();

        let names = com.iter().map(|r| r[1].to_string()).collect();
        Choice { n, cuts, names, jobs, minutes, gtfs_minutes, km, filled: n * n - m as usize, flows, mode_flows, beta, mode_coef, k, mode_k }
    }

    /// n×n probabilities P(work comuna | home comuna). Rows sum to 1.
    pub fn destinations(&self, jobs: &[f64], minutes: &[f64]) -> Vec<f64> {
        gravity(self.n, jobs, minutes, self.beta, &self.k)
    }

    /// Mode shares [public, car, other] of each OD pair.
    pub fn modes(&self, minutes: &[f64]) -> Vec<[f64; 3]> {
        (0..self.n * self.n).map(|i| shares(&self.mode_coef, minutes[i], self.km[i], &self.mode_k[i])).collect()
    }
}
