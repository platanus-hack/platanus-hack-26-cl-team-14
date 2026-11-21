//! Household car ownership → mode shares of trips to work (link 3 of cut 5).
//! `eod_car_mode.csv` (EOD 2012, `tools/pack/eod_car_mode.py`): by home comuna, expanded households, households with a car, and the
//! expanded weekday trips to work by household car status and mode [public, car, other].
//! The input is the household car share H. The mixture uses the share of trips to work from car households:
//! t(H) = H·a / (H·a + (1 − H)·b), with a, b = trips to work per household with and without a car (measured by comuna).
//! P(H) = (1 − t)·P_nocar + t·P_car. The factor P_m(H_new) / P_m(H_base) is 1 at the base, so the Census calibration by origin stays.
//! This is a survey association (EOD 2012), not a measured effect of a change in ownership.
//! A comuna with fewer than 30 sample trips in either group, or not in the survey, uses the pooled values of the region.
use std::collections::HashMap;
use std::path::Path;

/// One comuna: households, households with a car, [without, with a car] × [public, car, other] (sample trips, expanded trips).
struct Row { households: f64, with_car: f64, trips: [[(u32, f64); 3]; 2] }

pub struct CarOwnership {
    /// Household car share H of each comuna, in the order of `cuts`.
    pub base: Vec<f64>,
    /// Trips to work per household [without, with a car] of each comuna.
    rate: Vec<[f64; 2]>,
    /// [without, with a car] mode shares of each comuna.
    shares: Vec<[[f64; 3]; 2]>,
}

const MIN_SAMPLE: u32 = 30;

impl CarOwnership {
    pub fn load(dir: &Path, cuts: &[String]) -> CarOwnership {
        let m = |s: &str| match s { "public" => 0, "car" => 1, _ => 2 };
        let mut rows: HashMap<String, Row> = HashMap::new();
        for r in csv::Reader::from_path(dir.join("eod_car_mode.csv")).unwrap().into_records().map(|r| r.unwrap()) {
            let e = rows.entry(r[0].to_string()).or_insert(Row { households: r[1].parse().unwrap(), with_car: r[2].parse().unwrap(), trips: [[(0, 0.0); 3]; 2] });
            e.trips[r[3].parse::<usize>().unwrap()][m(&r[4])] = (r[5].parse().unwrap(), r[6].parse().unwrap());
        }
        let norm = |t: [f64; 3]| { let s: f64 = t.iter().sum(); t.map(|x| x / s) };
        let rate = |hh: [f64; 2], t: &[[(u32, f64); 3]; 2]| [0, 1].map(|k| t[k].iter().map(|x| x.1).sum::<f64>() / hh[k]);
        let mut all = Row { households: 0.0, with_car: 0.0, trips: [[(0, 0.0); 3]; 2] };
        for r in rows.values() {
            all.households += r.households;
            all.with_car += r.with_car;
            for k in 0..2 { for j in 0..3 { all.trips[k][j].0 += r.trips[k][j].0; all.trips[k][j].1 += r.trips[k][j].1; } }
        }
        let pooled = (all.trips.map(|g| norm(g.map(|x| x.1))), rate([all.households - all.with_car, all.with_car], &all.trips));
        let mut c = CarOwnership { base: vec![], rate: vec![], shares: vec![] };
        for cut in cuts {
            let (h, (s, r)) = match rows.get(cut) {
                Some(r) if r.trips.iter().all(|g| g.iter().map(|x| x.0).sum::<u32>() >= MIN_SAMPLE) =>
                    (r.with_car / r.households, (r.trips.map(|g| norm(g.map(|x| x.1))), rate([r.households - r.with_car, r.with_car], &r.trips))),
                Some(r) => (r.with_car / r.households, pooled),
                None => (all.with_car / all.households, pooled),
            };
            c.base.push(h);
            c.shares.push(s);
            c.rate.push(r);
        }
        c
    }

    /// Mode factors [public, car, other] of origin comuna `o` when its household car share goes from the base to `h`.
    pub fn factors(&self, o: usize, h: f64) -> [f64; 3] {
        let ([b, a], s) = (self.rate[o], &self.shares[o]);
        let p = |h: f64| { let t = h * a / (h * a + (1.0 - h) * b); [0, 1, 2].map(|m| (1.0 - t) * s[0][m] + t * s[1][m]) };
        let (x, y) = (p(self.base[o]), p(h.clamp(0.0, 1.0)));
        [0, 1, 2].map(|m| y[m] / x[m])
    }
}
