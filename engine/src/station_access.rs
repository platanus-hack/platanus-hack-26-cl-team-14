//! Walk access to open Metro parents, plus one direct bus for points outside walking range.
use super::{Fit, PointMass, Station, baseline_points, metres, parents_in_reach, predict, rows};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

const WALK_RADIUS_M: f64 = 1000.0;
const WALK_MPS: f64 = 4.8 / 3.6;
const MAX_SECONDS: f64 = 1800.0;

#[derive(Deserialize)]
struct Link {
    stop_id: String,
    longitude: f64,
    latitude: f64,
    parent_id: String,
    seconds: f64,
}

struct Stop {
    longitude: f64,
    latitude: f64,
    links: Vec<(usize, f64)>,
}

struct BusAccess {
    stops: Vec<Stop>,
    grid: HashMap<(i32, i32), Vec<usize>>,
}

// At Santiago's latitude each cell is wider and taller than a 1,000 m walk.
fn cell(lon: f64, lat: f64) -> (i32, i32) {
    ((lon / 0.012).floor() as i32, (lat / 0.012).floor() as i32)
}

impl BusAccess {
    fn load(data: &Path, stations: &[Station]) -> Self {
        let parents: HashMap<_, _> = stations.iter().enumerate().map(|(i, s)| (s.id.as_str(), i)).collect();
        let mut access = Self { stops: Vec::new(), grid: HashMap::new() };
        let mut stop_ids = HashMap::new();
        for link in rows::<Link>(&data.join("bus_station_access.csv")) {
            if link.seconds > MAX_SECONDS { continue; }
            let i = *stop_ids.entry(link.stop_id).or_insert_with(|| {
                let i = access.stops.len();
                access.grid.entry(cell(link.longitude, link.latitude)).or_default().push(i);
                access.stops.push(Stop { longitude: link.longitude, latitude: link.latitude, links: Vec::new() });
                i
            });
            access.stops[i].links.push((parents[link.parent_id.as_str()], link.seconds));
        }
        for s in &mut access.stops {
            s.links.sort_by(|(a, ta), (b, tb)| ta.total_cmp(tb).then_with(|| stations[*a].id.cmp(&stations[*b].id)));
        }
        access
    }

    fn fastest(&self, x: f64, y: f64, links: &[Option<(usize, f64)>], stations: &[Station]) -> Option<usize> {
        let (cx, cy) = cell(x, y);
        let mut best: Option<(f64, &str, usize)> = None;
        for col in cx - 1..=cx + 1 {
            for row in cy - 1..=cy + 1 {
                let Some(stops) = self.grid.get(&(col, row)) else { continue };
                for &i in stops {
                    let Some((parent, seconds)) = links[i] else { continue };
                    let stop = &self.stops[i];
                    let d = metres(x, y, stop.longitude, stop.latitude);
                    let total = d / WALK_MPS + seconds;
                    if d <= WALK_RADIUS_M && total <= MAX_SECONDS {
                        let path = (total, stations[parent].id.as_str(), parent);
                        if best.is_none_or(|old| path < old) { best = Some(path); }
                    }
                }
            }
        }
        best.map(|(_, _, parent)| parent)
    }
}

#[derive(Clone, Default)]
struct Mass {
    walk: [f64; 2],
    bus: [f64; 2],
}

pub struct StationAccess {
    stations: Vec<Station>,
    fit: Fit,
    bus: BusAccess,
    pub bus_scale: f64,
    pub bus_mix: f64,
    pub training_cells: usize,
}

impl StationAccess {
    /// Fit once with all 126 parents open. The training file masks observations at the 31 held-out parents.
    pub fn load(data: &Path) -> Self {
        let stations: Vec<Station> = serde_json::from_reader(std::fs::File::open(data.join("metro_training.json")).unwrap()).unwrap();
        let (_, _, fit) = predict(data, &stations, WALK_RADIUS_M, true, true, true, true);
        let bus = BusAccess::load(data, &stations);
        let mut model = Self { stations, fit, bus, bus_scale: 0.0, bus_mix: 0.0, training_cells: 0 };
        let (homes, jobs) = model.baseline_points(data);
        let masses = model.place(&homes, &jobs, &model.all_parent_ids());
        let mut cells = Vec::new();
        for (i, s) in model.stations.iter().enumerate().filter(|(_, s)| !s.holdout) {
            for (t, observed) in s.laboral.as_ref().unwrap().iter().enumerate() {
                if let Some(y) = observed {
                    let (walk, full) = model.components(&masses[i], i, t);
                    cells.push((*y, walk, full));
                }
            }
        }
        model.training_cells = cells.len();
        model.bus_scale = cells.iter().map(|c| c.0).sum::<f64>() / cells.iter().map(|c| c.2).sum::<f64>();
        // The weighted median is the smallest mix that minimizes absolute training error.
        let mut ratios: Vec<_> = cells.iter().filter_map(|&(y, a, full)| {
            let diff = model.bus_scale * full - a;
            (diff != 0.0).then(|| ((y - a) / diff, diff.abs()))
        }).collect();
        ratios.sort_by(|a, b| a.0.total_cmp(&b.0));
        let half = ratios.iter().map(|r| r.1).sum::<f64>() / 2.0;
        let mut weight = 0.0;
        for (mix, w) in ratios {
            weight += w;
            if weight >= half {
                model.bus_mix = mix.clamp(0.0, 1.0);
                break;
            }
        }
        model
    }

    pub fn all_parent_ids(&self) -> HashSet<String> {
        self.stations.iter().map(|s| s.id.clone()).collect()
    }

    pub fn walk_scale(&self) -> f64 { self.fit.a }

    pub fn baseline_points(&self, data: &Path) -> (Vec<PointMass>, Vec<PointMass>) {
        baseline_points(data, &self.fit.share, true)
    }

    fn place(&self, homes: &[PointMass], jobs: &[PointMass], active: &HashSet<String>) -> Vec<Mass> {
        let open: Vec<_> = self.stations.iter().map(|s| active.contains(&s.id)).collect();
        let links: Vec<_> = self.bus.stops.iter().map(|s| s.links.iter().find(|(i, _)| open[*i]).copied()).collect();
        let mut masses = vec![Mass::default(); self.stations.len()];
        for (half, points) in [homes, jobs].into_iter().enumerate() {
            for &(x, y, mass) in points {
                if mass == 0.0 { continue; }
                let (mut near, _) = parents_in_reach(&self.stations, x, y, WALK_RADIUS_M, true);
                near.retain(|(i, _)| open[*i]);
                if near.is_empty() {
                    if let Some(i) = self.bus.fastest(x, y, &links, &self.stations) {
                        masses[i].bus[half] += mass;
                    }
                } else {
                    let sum: f64 = near.iter().map(|n| n.1).sum();
                    for (i, w) in near { masses[i].walk[half] += mass * w / sum; }
                }
            }
        }
        masses
    }

    /// Networkwide access for an event attendee who could use Metro. Walk shares sum to one.
    /// A one-bus connection outside walking range receives the fitted bus mix as a transfer assumption.
    pub fn event_access_share(&self, longitude: f64, latitude: f64, active: &HashSet<String>) -> f64 {
        self.place(&[(longitude, latitude, 1.0)], &[], active).iter()
            .map(|m| m.walk[0] + self.bus_mix * m.bus[0]).sum()
    }

    fn components(&self, mass: &Mass, i: usize, t: usize) -> (f64, f64) {
        let profile = self.fit.p[self.fit.class[i]][t];
        let half = t / 24;
        (profile * mass.walk[half] * self.fit.a, profile * (mass.walk[half] + mass.bus[half]))
    }

    /// Re-enable or remove known GTFS parents by changing the active set. Closed parents return 48 zero bins.
    pub fn project_points(&self, homes: &[PointMass], jobs: &[PointMass], active: &HashSet<String>) -> Vec<(String, String, Vec<f64>)> {
        let masses = self.place(homes, jobs, active);
        self.stations.iter().enumerate().map(|(i, s)| {
            let bins = (0..48).map(|t| {
                let (a, full) = self.components(&masses[i], i, t);
                (1.0 - self.bus_mix) * a + self.bus_mix * (self.bus_scale * full)
            }).collect();
            (s.id.clone(), s.name.clone(), bins)
        }).collect()
    }
}
