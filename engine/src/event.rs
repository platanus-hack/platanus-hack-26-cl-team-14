//! Additional hypothetical event (accepted contract `context/research/extra-event-contract.md`, PR #95).
//! Template: the observed Saturday 2026-04-11 evening contrast at Parque O'Higgins over its 25 Line 2 peers,
//! E_t = max(0, PQ_t - D_PQ * median(peer_t / D_peer)) for t = 38..47, null when either term is missing, 0 outside.
//! A scenario adds multiplier x weight x E_t to the observed Saturday series of the parents within 1,000 m of a
//! reviewed venue, weights 1/max(d, 50 m) (the L4b rule). The source event is not removed.
use serde::{Deserialize, Serialize};
use std::path::Path;

const WINDOW: std::ops::Range<usize> = 38..48;
const LABEL: &str = "additional hypothetical event; contrast attribution and transfer are assumptions";

#[derive(Deserialize)]
struct Bins { #[serde(rename = "SABADO")] sabado: Vec<Option<f64>> }
#[derive(Deserialize)]
struct Id { id: String }
#[derive(Deserialize)]
struct Obs { station: Id, half_hour_bins: Bins }
#[derive(Deserialize)]
struct Observations { stations: Vec<Obs> }
#[derive(Deserialize)]
struct Case { day_type: String, station_id: String, peers: Vec<String> }
#[derive(Deserialize)]
struct Manifest { cases: Vec<Case> }
#[derive(Deserialize)]
struct Parent { id: String, name: String, longitude: f64, latitude: f64 }
#[derive(Deserialize, Clone)]
struct Venue { site_id: String, name: String, longitude: f64, latitude: f64 }
#[derive(Deserialize)]
struct Venues { venues: Vec<Venue> }

pub struct EventModel { parents: Vec<Parent>, sat: Vec<Vec<Option<f64>>>, venues: Vec<Venue>, template: Vec<Option<f64>>, source_sha256: String }

#[derive(Deserialize)]
pub struct EventRequest { pub site_id: String, pub multiplier: f64 }

#[derive(Serialize)]
pub struct EventStation {
    id: String, name: String, longitude: f64, latitude: f64, distance_m: f64, weight: f64,
    observed: Vec<Option<f64>>, event_delta: Vec<Option<f64>>, scenario: Vec<Option<f64>>,
    target_observed_mask: Vec<bool>, common_known_bins: usize,
    baseline_common_sum: f64, scenario_common_sum: f64, delta_common_sum: f64, observed_day_total: f64,
}

#[derive(Serialize)]
pub struct EventResponse {
    site_id: String, site_name: String, longitude: f64, latitude: f64, multiplier: f64,
    source_day: &'static str, source_sha256: String, template: Vec<Option<f64>>, source_template_mask: Vec<bool>,
    known_template_mass_times_multiplier: f64, assigned_mass: f64, unassigned_mass: f64,
    stations: Vec<EventStation>, label: &'static str,
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn day_total(b: &[Option<f64>]) -> f64 { b.iter().flatten().sum() }

impl EventModel {
    /// `data` is data/mvp; the frozen peer IDs come from data/validation/event_shape_2026_04/manifest.json.
    pub fn new(data: &Path) -> EventModel {
        let raw = std::fs::read(data.join("metro_observations.json")).unwrap();
        let o: Observations = serde_json::from_slice(&raw).unwrap();
        let m: Manifest = serde_json::from_reader(std::fs::File::open(data.join("../validation/event_shape_2026_04/manifest.json")).unwrap()).unwrap();
        let parents: Vec<Parent> = serde_json::from_reader(std::fs::File::open(data.join("metro_training.json")).unwrap()).unwrap();
        let venues: Venues = serde_json::from_reader(std::fs::File::open(data.join("event_venues.json")).unwrap()).unwrap();
        let sat_of = |id: &str| o.stations.iter().find(|s| s.station.id == id).unwrap().half_hour_bins.sabado.clone();
        let case = m.cases.iter().find(|c| c.day_type == "SABADO" && c.station_id == "PQ").unwrap();
        let pq = sat_of("PQ");
        let peers: Vec<Vec<Option<f64>>> = case.peers.iter().map(|p| sat_of(p)).collect();
        let mut template = vec![Some(0.0); 48];
        for t in WINDOW {
            let mut r: Vec<f64> = peers.iter().filter_map(|b| b[t].map(|x| x / day_total(b))).collect();
            template[t] = match (pq[t], r.is_empty()) {
                (Some(v), false) => {
                    r.sort_by(f64::total_cmp);
                    let n = r.len();
                    let med = if n % 2 == 1 { r[n / 2] } else { (r[n / 2 - 1] + r[n / 2]) / 2.0 };
                    Some((v - day_total(&pq) * med).max(0.0))
                }
                _ => None,
            };
        }
        let sat = parents.iter().map(|p| sat_of(&p.id)).collect();
        EventModel { parents, sat, venues: venues.venues, template, source_sha256: sha256(&raw) }
    }

    pub fn scenario(&self, r: EventRequest) -> Result<EventResponse, String> {
        let Some(v) = self.venues.iter().find(|v| v.site_id == r.site_id) else {
            return Err(format!("site_id must be one of {:?}", self.venues.iter().map(|v| &v.site_id).collect::<Vec<_>>()));
        };
        if !(r.multiplier.is_finite() && (0.0..=3.0).contains(&r.multiplier)) { return Err("multiplier must be finite and in 0..=3".into()); }
        // L4b's unchanged flat distance and 1/max(d, 50 m) weights over the parents within 1,000 m.
        let kx = 111_320.0 * (-33.45f64).to_radians().cos();
        let near: Vec<(usize, f64)> = self.parents.iter().enumerate()
            .map(|(i, p)| (i, ((p.longitude - v.longitude) * kx).hypot((p.latitude - v.latitude) * 110_540.0)))
            .filter(|&(_, d)| d <= 1000.0).collect();
        let tot: f64 = near.iter().map(|&(_, d)| 1.0 / d.max(50.0)).sum();
        let known: f64 = self.template.iter().flatten().sum::<f64>() * r.multiplier;
        let stations: Vec<EventStation> = near.iter().map(|&(i, d)| {
            let w = (1.0 / d.max(50.0)) / tot;
            let obs = &self.sat[i];
            let delta: Vec<Option<f64>> = (0..48).map(|t| if r.multiplier == 0.0 { Some(0.0) } else { self.template[t].map(|e| r.multiplier * w * e) }).collect();
            let scen: Vec<Option<f64>> = (0..48).map(|t| match (obs[t], delta[t]) { (Some(o), Some(e)) => Some(o + e), _ => None }).collect();
            let common: Vec<usize> = (0..48).filter(|&t| scen[t].is_some()).collect();
            let (b, s): (f64, f64) = (common.iter().map(|&t| obs[t].unwrap()).sum(), common.iter().map(|&t| scen[t].unwrap()).sum());
            let p = &self.parents[i];
            EventStation {
                id: p.id.clone(), name: p.name.clone(), longitude: p.longitude, latitude: p.latitude, distance_m: d, weight: w,
                target_observed_mask: obs.iter().map(Option::is_some).collect(), common_known_bins: common.len(),
                baseline_common_sum: b, scenario_common_sum: s, delta_common_sum: s - b, observed_day_total: day_total(obs),
                observed: obs.clone(), event_delta: delta, scenario: scen,
            }
        }).collect();
        let assigned = if stations.is_empty() { 0.0 } else { known };
        Ok(EventResponse {
            site_id: v.site_id.clone(), site_name: v.name.clone(), longitude: v.longitude, latitude: v.latitude, multiplier: r.multiplier,
            source_day: "SABADO 2026-04-11 America/Santiago", source_sha256: self.source_sha256.clone(),
            template: self.template.clone(), source_template_mask: self.template.iter().map(Option::is_some).collect(),
            known_template_mass_times_multiplier: known, assigned_mass: assigned, unassigned_mass: known - assigned, stations, label: LABEL,
        })
    }
}
