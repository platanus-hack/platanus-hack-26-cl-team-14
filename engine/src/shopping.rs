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

/// One shop point: longitude, latitude, comuna index.
pub type Shop = (f64, f64, usize);

pub struct Shopping {
    /// Weekday shopping trips by home comuna. Zero for a comuna without an EOD 2012 rate.
    trips: Vec<f64>,
    pub shops: Vec<Shop>,
    /// Comunas without an EOD 2012 rate (outside the survey area).
    no_rate: Vec<String>,
    /// Total variation between modeled and EOD 2012 destination totals by comuna, at baseline.
    destination_tv: f64,
}

fn records(p: &Path) -> impl Iterator<Item = csv::StringRecord> {
    csv::Reader::from_path(p).unwrap().into_records().map(|r| r.unwrap())
}

/// Trips by comuna pair (row = home, column = destination).
fn flows(trips: &[f64], shops: &[Shop], minutes: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = trips.len();
    let mut size = vec![0.0_f64; n];
    for s in shops { size[s.2] += 1.0; }
    let attraction: Vec<f64> = size.iter().map(|s| (s + 1.0).powf(ALPHA)).collect(); // +1: comunas outside the OSM extract
    let mut od = vec![0.0; n * n];
    for o in 0..n {
        let row = &mut od[o * n..][..n];
        for d in 0..n { row[d] = attraction[d] * (-BETA * minutes[o * n + d]).exp(); }
        let total: f64 = row.iter().sum();
        row.iter_mut().for_each(|x| *x *= trips[o] / total);
    }
    (od, size)
}

impl Shopping {
    pub fn load(dir: &Path, cuts: &[String], homes: &[Point], minutes: &[f64]) -> Shopping {
        let n = cuts.len();
        let ix = |c: &str| cuts.iter().position(|x| x == c);
        let mut residents = vec![0.0; n];
        for p in homes { residents[ix(&p.cut).unwrap()] += p.weight; }
        let mut rate = vec![None; n];
        for r in records(&dir.join("eod_trip_rates.csv")).filter(|r| &r[1] == "shopping") {
            if let Some(o) = ix(&r[0]) { rate[o] = Some(r[6].parse::<f64>().unwrap()); }
        }
        let no_rate = (0..n).filter(|&o| rate[o].is_none()).map(|o| cuts[o].clone()).collect();
        let trips: Vec<f64> = (0..n).map(|o| residents[o] * rate[o].unwrap_or(0.0)).collect();
        let shops: Vec<Shop> = records(&dir.join("activity_points.csv"))
            .filter(|r| ["shop", "mall", "supermarket"].contains(&&r[0]))
            .filter_map(|r| Some((r[2].parse().unwrap(), r[3].parse().unwrap(), ix(&r[4])?))).collect();
        // EOD 2012 destination totals by comuna, homes pooled. Unknown destinations are left out of both sides.
        let mut observed = vec![0.0; n];
        for r in records(&dir.join("eod_destinations.csv")).filter(|r| &r[1] == "shopping") {
            if let Some(d) = ix(&r[3]) { observed[d] += r[6].parse::<f64>().unwrap(); }
        }
        let (od, _) = flows(&trips, &shops, minutes);
        let modeled: Vec<f64> = (0..n).map(|d| (0..n).map(|o| od[o * n + d]).sum()).collect();
        let (m, s) = (modeled.iter().sum::<f64>(), observed.iter().sum::<f64>());
        let destination_tv = (0..n).map(|d| (modeled[d] / m - observed[d] / s).abs()).sum::<f64>() / 2.0;
        Shopping { trips, shops, no_rate, destination_tv }
    }

    /// Add `delta` shop points at a point (comuna `cut`), or remove the `-delta` nearest shop points.
    /// Returns what it took and the added or removed points.
    pub fn edit(shops: &mut Vec<Shop>, longitude: f64, latitude: f64, cut: usize, delta: i64) -> (Value, Vec<Shop>) {
        if delta >= 0 {
            let added = vec![(longitude, latitude, cut); delta as usize];
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

    /// Base and scenario shopping trips. `sites` are the edited points: (longitude, latitude, shops removed there, shops added there).
    pub fn run(&self, cuts: &[String], names: &[String], base_minutes: &[f64], minutes: &[f64], shops: &[Shop], sites: &[(f64, f64, Vec<Shop>, Vec<Shop>)]) -> Value {
        let n = cuts.len();
        let (b, b_size) = flows(&self.trips, &self.shops, base_minutes);
        let (s, s_size) = flows(&self.trips, shops, minutes);
        let inflow = |od: &[f64], d: usize| (0..n).map(|o| od[o * n + d]).sum::<f64>();
        // Trips to a set of shop points, by home comuna: each point gets 1 / shops of its comuna.
        let visitors = |od: &[f64], size: &[f64], points: &[Shop]| -> Vec<f64> {
            (0..n).map(|o| points.iter().map(|p| od[o * n + p.2] / size[p.2]).sum()).collect()
        };
        let summary = |by_home: Vec<f64>| {
            let total: f64 = by_home.iter().sum();
            let mut order: Vec<usize> = (0..n).collect();
            order.sort_by(|&x, &y| by_home[y].total_cmp(&by_home[x]));
            json!({ "trips": total.round() + 0.0, "by_home_top5": order[..5].iter().filter(|&&o| by_home[o] > 0.0)
                .map(|&o| json!({ "id": cuts[o], "name": names[o], "trips": by_home[o].round() })).collect::<Vec<_>>() })
        };
        // A removed site reports its shoppers in the base, an added site in the scenario.
        // Only baseline shops count as removed and only shops still in the final stock count as added.
        let sites: Vec<Value> = sites.iter().map(|(x, y, removed, added)| {
            let removed: Vec<Shop> = removed.iter().filter(|p| self.shops.contains(p)).copied().collect();
            let added: Vec<Shop> = added.iter().filter(|p| shops.contains(p)).copied().collect();
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
                     "alpha": ALPHA, "beta_per_minute": BETA },
            "comunas_without_eod_rate": self.no_rate,
            "unit": "weekday outbound shopping trips of residents (EOD 2012 rates × Census 2024 residents), not distinct persons; returns are not included",
        })
    }
}
