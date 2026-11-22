//! Weekday shopping trips of residents and the shop points that attract them.
//! Trips: EOD 2012 shopping trips per resident of the home comuna (`eod_trip_rates.csv`) × Census 2024 residents (`blocks.csv`).
//! Destination: Huff model by comuna, P(d | o) ∝ (shops_d + 1)^α × exp(−β × minutes_od), fitted on EOD 2012 shopping trips
//! (`tools/pack/shopping_huff.py`, `data/mvp/shopping_huff.md`). shops_d = mapped shop, mall and supermarket points
//! (`activity_points.csv`). Inside a comuna each shop point gets an equal part. A change of shop points moves shoppers
//! by the same rule; there is no rule for one place.
use crate::population::Point;
use serde_json::{Value, json};
use std::path::Path;

const ALPHA: f64 = 0.586;
const BETA: f64 = 0.2047;
/// Huff fit of EOD 2012 health trips (Proposito 5) to hospital and clinic points: `shopping_huff.py 5 hospital,clinic`.
pub const HEALTH: (&str, &[&str], f64, f64) = ("health", &["hospital", "clinic"], 1.170, 0.1354);
/// The shopping layer: shop, mall and supermarket points.
pub const SHOPPING: (&str, &[&str], f64, f64) = ("shopping", &["shop", "mall", "supermarket"], ALPHA, BETA);

/// One shop point: longitude, latitude, comuna index, id (baseline points 0..n, added points after).
pub type Shop = (f64, f64, usize, u32);

#[path = "shopping/visits.rs"]
pub mod visits;

pub struct Shopping {
    /// Weekday shopping trips by home comuna. Zero for a comuna without an EOD 2012 rate.
    pub trips: Vec<f64>,
    pub shops: Vec<Shop>,
    /// Comunas without an EOD 2012 rate (outside the survey area).
    no_rate: Vec<String>,
    /// Departure half-hour shares (48) and mode shares (car, public, walk, bicycle, other) of shopping trips, by home comuna.
    /// EOD 2012 `eod_trip_profiles.csv`. A trip keeps the profile of its home comuna at any destination.
    profile: Vec<([f64; 48], [f64; 5])>,
    /// Total variation between modeled and EOD 2012 destination totals by comuna, at baseline.
    destination_tv: f64,
    /// EOD purpose, destination point kinds and Huff fit (SHOPPING or HEALTH).
    layer: (&'static str, &'static [&'static str], f64, f64),
}

fn records(p: &Path) -> impl Iterator<Item = csv::StringRecord> {
    csv::Reader::from_path(p).unwrap().into_records().map(|r| r.unwrap())
}

/// Trips by comuna pair (row = home, column = destination).
fn flows(trips: &[f64], shops: &[Shop], minutes: &[f64], alpha: f64, beta: f64) -> (Vec<f64>, Vec<f64>) {
    let n = trips.len();
    let mut size = vec![0.0_f64; n];
    for s in shops { size[s.2] += 1.0; }
    let attraction: Vec<f64> = size.iter().map(|s| (s + 1.0).powf(alpha)).collect(); // +1: comunas outside the OSM extract
    let mut od = vec![0.0; n * n];
    for o in 0..n {
        let row = &mut od[o * n..][..n];
        for d in 0..n { row[d] = attraction[d] * (-beta * minutes[o * n + d]).exp(); }
        let total: f64 = row.iter().sum();
        row.iter_mut().for_each(|x| *x *= trips[o] / total);
    }
    (od, size)
}

impl Shopping {
    /// Baseline optional visits of the eligible persons (shopping/visits.rs).
    pub fn visits(&self, n: usize, homes: &[usize], minutes: &[f64]) -> visits::Visits {
        visits::Visits::new(n, homes, &self.trips, &self.profile, &self.shops, minutes)
    }

    pub fn load(dir: &Path, cuts: &[String], homes: &[Point], minutes: &[f64], layer: (&'static str, &'static [&'static str], f64, f64)) -> Shopping {
        let n = cuts.len();
        let ix = |c: &str| cuts.iter().position(|x| x == c);
        let mut residents = vec![0.0; n];
        for p in homes { residents[ix(&p.cut).unwrap()] += p.weight; }
        let mut rate = vec![None; n];
        for r in records(&dir.join("eod_trip_rates.csv")).filter(|r| &r[1] == layer.0) {
            if let Some(o) = ix(&r[0]) { rate[o] = Some(r[6].parse::<f64>().unwrap()); }
        }
        let mut profile = vec![([0.0; 48], [0.0; 5]); n];
        for r in records(&dir.join("eod_trip_profiles.csv")).filter(|r| &r[1] == layer.0) {
            let (Some(o), Ok(bin)) = (ix(&r[0]), r[3].parse::<usize>()) else { continue }; // bin "unknown" has no time
            let w: f64 = r[6].parse().unwrap();
            match &r[2] {
                "start_half_hour" => profile[o].0[bin] += w,
                // ModoAgregado: 1 car; 2–7 and 9–12 public transport; 17 walk; 18 bicycle (tools/pack/eod_leisure.py).
                "mode_eod" => profile[o].1[match bin { 1 => 0, 2..=7 | 9..=12 => 1, 17 => 2, 18 => 3, _ => 4 }] += w,
                _ => {}
            }
        }
        for (time, mode) in &mut profile {
            let (t, m) = (time.iter().sum::<f64>(), mode.iter().sum::<f64>());
            time.iter_mut().for_each(|x| *x /= t.max(1e-9));
            mode.iter_mut().for_each(|x| *x /= m.max(1e-9));
        }
        let no_rate = (0..n).filter(|&o| rate[o].is_none()).map(|o| cuts[o].clone()).collect();
        let trips: Vec<f64> = (0..n).map(|o| residents[o] * rate[o].unwrap_or(0.0)).collect();
        let shops: Vec<Shop> = records(&dir.join("activity_points.csv"))
            .filter(|r| layer.1.contains(&&r[0]))
            .filter_map(|r| Some((r[2].parse().unwrap(), r[3].parse().unwrap(), ix(&r[4])?)))
            .enumerate().map(|(i, (x, y, c))| (x, y, c, i as u32)).collect();
        // EOD 2012 destination totals by comuna, homes pooled. Unknown destinations are left out of both sides.
        let mut observed = vec![0.0; n];
        for r in records(&dir.join("eod_destinations.csv")).filter(|r| &r[1] == layer.0) {
            if let Some(d) = ix(&r[3]) { observed[d] += r[6].parse::<f64>().unwrap(); }
        }
        let (od, _) = flows(&trips, &shops, minutes, layer.2, layer.3);
        let modeled: Vec<f64> = (0..n).map(|d| (0..n).map(|o| od[o * n + d]).sum()).collect();
        let (m, s) = (modeled.iter().sum::<f64>(), observed.iter().sum::<f64>());
        let destination_tv = (0..n).map(|d| (modeled[d] / m - observed[d] / s).abs()).sum::<f64>() / 2.0;
        Shopping { trips, shops, no_rate, profile, destination_tv, layer }
    }

    /// Add `delta` shop points at a point (comuna `cut`), with ids from `next_id`, or remove the `-delta` nearest shop points.
    /// Returns what it took and the added or removed points.
    pub fn edit(shops: &mut Vec<Shop>, next_id: &mut u32, longitude: f64, latitude: f64, cut: usize, delta: i64) -> (Value, Vec<Shop>) {
        if delta >= 0 {
            let added: Vec<Shop> = (0..delta).map(|_| { *next_id += 1; (longitude, latitude, cut, *next_id - 1) }).collect();
            shops.extend(&added);
            return (json!({ "shops_added": delta }), added);
        }
        let d = |s: &Shop| ((s.0 - longitude) * 0.835).hypot(s.1 - latitude) * 111_320.0;
        let mut order: Vec<usize> = (0..shops.len()).collect();
        order.sort_by(|&a, &b| d(&shops[a]).total_cmp(&d(&shops[b])));
        order.truncate(delta.unsigned_abs() as usize);
        let farthest = order.last().map_or(0.0, |&i| d(&shops[i]));
        order.sort_unstable_by(|a, b| b.cmp(a));
        let removed: Vec<Shop> = order.iter().map(|&i| shops.swap_remove(i)).collect();
        (json!({ "shops_removed": removed.len(), "farthest_removed_m": farthest.round() }), removed)
    }

    /// Base and scenario trips. `sites` are the edited points: (longitude, latitude, shops removed there, shops added there).
    /// Only the public-transport trips that leave inside `window` (all day when None) see the scenario minutes: share of the
    /// home comuna's EOD 2012 profile (public mode share × departure share in the window). The rest keep the base minutes.
    pub fn run(&self, cuts: &[String], names: &[String], base_minutes: &[f64], minutes: &[f64], shops: &[Shop], sites: &[(f64, f64, Vec<Shop>, Vec<Shop>)], window: Option<(usize, usize)>) -> Value {
        let n = cuts.len();
        let (b, b_size) = flows(&self.trips, &self.shops, base_minutes, self.layer.2, self.layer.3);
        let (moved, s_size) = flows(&self.trips, shops, minutes, self.layer.2, self.layer.3);
        let (kept, _) = flows(&self.trips, shops, base_minutes, self.layer.2, self.layer.3);
        let exposed: Vec<f64> = self.profile.iter().map(|(time, mode)| mode[1] * window.map_or(1.0, |(a, z)| time[a..z].iter().sum())).collect();
        let s: Vec<f64> = (0..n * n).map(|i| exposed[i / n] * moved[i] + (1.0 - exposed[i / n]) * kept[i]).collect();
        let inflow = |od: &[f64], d: usize| (0..n).map(|o| od[o * n + d]).sum::<f64>();
        // Trips to a set of shop points, by home comuna: each point gets 1 / shops of its comuna.
        let visitors = |od: &[f64], size: &[f64], points: &[Shop]| -> Vec<f64> {
            (0..n).map(|o| points.iter().map(|p| od[o * n + p.2] / size[p.2]).sum()).collect()
        };
        let summary = |by_home: Vec<f64>| {
            let total: f64 = by_home.iter().sum();
            let mut order: Vec<usize> = (0..n).collect();
            order.sort_by(|&x, &y| by_home[y].total_cmp(&by_home[x]));
            let (mut time, mut mode) = ([0.0; 48], [0.0; 5]);
            for o in 0..n {
                for (t, p) in time.iter_mut().zip(self.profile[o].0) { *t += by_home[o] * p; }
                for (m, p) in mode.iter_mut().zip(self.profile[o].1) { *m += by_home[o] * p; }
            }
            let [car, public, walk, bicycle, other] = mode.map(|x| x.round());
            json!({ "trips": total.round() + 0.0, "by_home_top5": order[..5].iter().filter(|&&o| by_home[o] > 0.0)
                .map(|&o| json!({ "id": cuts[o], "name": names[o], "trips": by_home[o].round() })).collect::<Vec<_>>(),
                "by_mode": { "car": car, "public": public, "walk": walk, "bicycle": bicycle, "other": other },
                "by_departure_half_hour": time.iter().map(|x| x.round()).collect::<Vec<_>>() })
        };
        // A removed site reports its shoppers in the base, an added site in the scenario.
        // Only baseline shops count as removed and only shops still in the final stock count as added.
        let sites: Vec<Value> = sites.iter().map(|(x, y, removed, added)| {
            let removed: Vec<Shop> = removed.iter().filter(|p| (p.3 as usize) < self.shops.len()).copied().collect();
            let added: Vec<Shop> = added.iter().filter(|p| shops.iter().any(|s| s.3 == p.3)).copied().collect();
            let (shops, visitors) = if added.is_empty() { (-(removed.len() as i64), visitors(&b, &b_size, &removed)) } else { (added.len() as i64, visitors(&s, &s_size, &added)) };
            json!({ "longitude": x, "latitude": y, "shops": shops, "shoppers": summary(visitors) })
        }).collect();
        let mut changed: Vec<usize> = (0..n).filter(|&d| (inflow(&s, d) - inflow(&b, d)).abs() >= 1.0).collect();
        changed.sort_by(|&x, &y| (inflow(&s, y) - inflow(&b, y)).abs().total_cmp(&(inflow(&s, x) - inflow(&b, x)).abs()));
        let destinations: Vec<Value> = changed.into_iter().map(|d| json!({
            "id": cuts[d], "name": names[d], "shops_base": b_size[d], "shops_scenario": s_size[d],
            "trips_base": inflow(&b, d).round(), "trips_scenario": inflow(&s, d).round(),
        })).collect();
        json!({
            "trips_total": self.trips.iter().sum::<f64>().round(),
            "sites": sites, "destinations_changed": destinations,
            "fit": { "destination_total_variation_vs_eod_2012": (self.destination_tv * 1000.0).round() / 1000.0,
                     "alpha": self.layer.2, "beta_per_minute": self.layer.3 },
            "comunas_without_eod_rate": self.no_rate,
            "exposed_trips": (0..n).map(|o| self.trips[o] * self.profile[o].1[1] * window.map_or(1.0, |(a, z)| self.profile[o].0[a..z].iter().sum())).sum::<f64>().round(),
            "unit": format!("weekday outbound {} trips of residents (EOD 2012 rates × Census 2024 residents), not distinct persons; returns are not included. shops_* count the destination points of kinds {}", self.layer.0, self.layer.1.join(", ")),
        })
    }
}
