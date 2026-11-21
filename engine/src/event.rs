//! Extra hypothetical event sensitivity (contract PR #95, draft until merged). The source is the observed
//! Saturday 2026-04-11 evening excess at Parque O'Higgins over its Line 2 peers, bins 38-47 (19:00-23:30):
//! E_t = max(0, PQ_t - D_PQ * median_k(peer_t / D_peer)), D = sum of that station's observed Saturday bins.
//! A scenario adds multiplier x E_t to the observed Saturday series of the parents within 1,000 m of a venue,
//! split with the accepted L4b rule 1/max(d, 50 m). Nothing is subtracted at the source; unknowns stay unknown.
use serde::{Deserialize, Serialize};
use std::path::Path;

const WINDOW: std::ops::Range<usize> = 38..48;
/// Venue points: OSM stadium-bowl centroids (clever-heron, 10:14 UTC), fixed by the contract.
const VENUES: [(&str, f64, f64); 2] = [("NACIONAL", -70.6106404881, -33.4644707147), ("MONUMENTAL", -70.6059662871, -33.5065367738)];

#[derive(Deserialize)]
struct Bins { #[serde(rename = "SABADO")] sabado: Vec<Option<f64>> }
#[derive(Deserialize)]
struct Point { id: String, name: String, longitude: f64, latitude: f64 }
#[derive(Deserialize)]
struct Obs { station: Point, half_hour_bins: Bins }
#[derive(Deserialize)]
struct Observations { stations: Vec<Obs> }
#[derive(Deserialize)]
struct Case { day_type: String, station_id: String, peers: Vec<String> }
#[derive(Deserialize)]
struct Manifest { cases: Vec<Case> }

pub struct EventModel { obs: Vec<Obs>, template: Vec<Option<f64>> }

#[derive(Deserialize)]
pub struct EventRequest { pub venue: String, pub multiplier: f64 }
#[derive(Serialize)]
pub struct EventStation { id: String, name: String, share: f64, observed: Vec<Option<f64>>, scenario: Vec<Option<f64>> }
#[derive(Serialize)]
pub struct EventResponse { venue: String, multiplier: f64, day: &'static str, template: Vec<Option<f64>>, stations: Vec<EventStation>, assumptions: [&'static str; 3] }

fn day_total(b: &[Option<f64>]) -> f64 { b.iter().flatten().sum() }

impl EventModel {
    pub fn new(data: &Path, manifest: &Path) -> EventModel {
        let o: Observations = serde_json::from_reader(std::fs::File::open(data.join("metro_observations.json")).unwrap()).unwrap();
        let m: Manifest = serde_json::from_reader(std::fs::File::open(manifest).unwrap()).unwrap();
        let case = m.cases.iter().find(|c| c.day_type == "SABADO" && c.station_id == "PQ").unwrap();
        let sat = |id: &str| &o.stations.iter().find(|s| s.station.id == id).unwrap().half_hour_bins.sabado;
        let pq = sat("PQ");
        let mut template = vec![None; 48];
        for t in WINDOW {
            let Some(v) = pq[t] else { continue };
            let mut r: Vec<f64> = case.peers.iter().filter_map(|p| { let b = sat(p); b[t].map(|x| x / day_total(b)) }).collect();
            r.sort_by(f64::total_cmp);
            let med = if r.len() % 2 == 1 { r[r.len() / 2] } else { (r[r.len() / 2 - 1] + r[r.len() / 2]) / 2.0 };
            template[t] = Some((v - day_total(pq) * med).max(0.0));
        }
        EventModel { obs: o.stations, template }
    }

    pub fn scenario(&self, r: EventRequest) -> Result<EventResponse, String> {
        let Some(&(venue, x, y)) = VENUES.iter().find(|v| v.0 == r.venue) else { return Err("venue must be NACIONAL or MONUMENTAL".into()) };
        if !(r.multiplier >= 0.0 && r.multiplier.is_finite()) { return Err("multiplier must be finite and >= 0".into()); }
        // Same flat distance and 1/max(d, 50 m) shares as the accepted L4b station model.
        let kx = 111_320.0 * (-33.45f64).to_radians().cos();
        let near: Vec<(&Obs, f64)> = self.obs.iter().map(|s| (s, ((s.station.longitude - x) * kx).hypot((s.station.latitude - y) * 110_540.0)))
            .filter(|&(_, d)| d <= 1000.0).map(|(s, d)| (s, 1.0 / d.max(50.0))).collect();
        let tot: f64 = near.iter().map(|n| n.1).sum();
        let stations = near.into_iter().map(|(s, w)| {
            let share = w / tot;
            let b = &s.half_hour_bins.sabado;
            let scenario = (0..48).map(|t| if r.multiplier == 0.0 || !WINDOW.contains(&t) { b[t] } else {
                match (b[t], self.template[t]) { (Some(o), Some(e)) => Some(o + r.multiplier * share * e), _ => None }
            }).collect();
            EventStation { id: s.station.id.clone(), name: s.station.name.clone(), share, observed: b.clone(), scenario }
        }).collect();
        Ok(EventResponse { venue: venue.into(), multiplier: r.multiplier, day: "SABADO 2026-04-11", template: self.template.clone(), stations,
            assumptions: ["The Parque O'Higgins Saturday evening excess over its Line 2 peers is taken as an event pattern; attribution to the event is an assumption.",
                "The same pattern is assumed to occur at another venue; transfer between places is an assumption, not a finding.",
                "The venue point is one stadium-bowl centroid; access is the L4b 1/d share within 1,000 m, not routing or capacity."] })
    }
}
