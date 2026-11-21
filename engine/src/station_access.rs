//! Walk access to open Metro parents, plus one direct bus for points outside walking range.
use super::{Fit, PointMass, Station, baseline_points, metres, parents_in_reach, predict_with_jobs, rows, sha256, JobId};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

const WALK_RADIUS_M: f64 = 1000.0;
const WALK_MPS: f64 = 4.8 / 3.6;
const MAX_SECONDS: f64 = 1800.0;

/// Inverse-distance access for a Metro-only leg, using the station model's walk radius.
pub(crate) fn walk_links(stations: &[Station], lon: f64, lat: f64) -> Vec<(usize, f64)> {
    parents_in_reach(stations, lon, lat, WALK_RADIUS_M, true).0
}

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

// Geometry is independent of the number of people at this point.
struct PointAccess {
    walk: Vec<(usize, f64)>,
    walk_sum: f64,
    bus: Option<usize>,
}

impl PointAccess {
    fn new(stations: &[Station], bus: &BusAccess, links: &[Option<(usize, f64)>], x: f64, y: f64) -> Self {
        let (walk, walk_sum) = parents_in_reach(stations, x, y, WALK_RADIUS_M, true);
        let bus = if walk.is_empty() { bus.fastest(x, y, links, stations) } else { None };
        Self { walk, walk_sum, bus }
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
    points: HashMap<(u64, u64), PointAccess>,
    pub bus_scale: f64,
    pub bus_mix: f64,
    pub training_cells: usize,
    jobs_file: &'static str,
    transport_factors: HashMap<String, f64>,
}

impl StationAccess {
    /// Fit once with all 126 parents open. The training file masks observations at the 31 held-out parents.
    pub fn load(data: &Path) -> Self {
        Self::fit(data, "jobs.csv", HashMap::new())
    }

    /// Fit with April destination weights. The city's job stocks and population still use jobs.csv.
    pub fn load_with_april_transport_jobs(data: &Path) -> Result<Self, String> {
        let april_file = "jobs_2026.csv";
        for (file, expected) in [
            ("jobs.csv", "4420a3956bc414601e8bf07bcdc99eddc3c865aebad47d2f137b2fc7ed15c481"),
            (april_file, "5ea069bc928a6d9677ce0f5d2e89542bfcb21761bf618b5d9681305d3a419866"),
        ] {
            let bytes = std::fs::read(data.join(file)).map_err(|e| format!("{file}: {e}"))?;
            if sha256(&bytes) != expected { return Err(format!("{file} differs from the April transport fit")); }
        }
        let mut factors: HashMap<_, _> = rows::<JobId>(&data.join(april_file)).into_iter()
            .map(|r| (r.block_id, r.jobs)).collect();
        for r in rows::<JobId>(&data.join("jobs.csv")) {
            let april = factors.get_mut(&r.block_id).unwrap();
            *april = if r.jobs > 0.0 { *april / r.jobs } else { 1.0 };
        }
        Ok(Self::fit(data, april_file, factors))
    }

    fn fit(data: &Path, jobs_file: &'static str, transport_factors: HashMap<String, f64>) -> Self {
        let stations: Vec<Station> = serde_json::from_reader(std::fs::File::open(data.join("metro_training.json")).unwrap()).unwrap();
        let (_, _, fit) = predict_with_jobs(data, jobs_file, &stations, WALK_RADIUS_M, true, true, true, true);
        let bus = BusAccess::load(data, &stations);
        let mut model = Self {
            stations, fit, bus, points: HashMap::new(), bus_scale: 0.0, bus_mix: 0.0,
            training_cells: 0, jobs_file, transport_factors,
        };
        let (homes, jobs) = model.baseline_points(data);
        let links: Vec<_> = model.bus.stops.iter().map(|s| s.links.first().copied()).collect();
        for &(x, y, _) in homes.iter().chain(&jobs) {
            model.points.entry((x.to_bits(), y.to_bits())).or_insert_with(|| {
                PointAccess::new(&model.stations, &model.bus, &links, x, y)
            });
        }
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
        baseline_points(data, self.jobs_file, &self.fit.share, true)
    }

    pub fn public_share(&self, cut: &str) -> f64 {
        let (public, total) = self.fit.share[cut];
        public / total
    }

    /// Apply only to the station projection's public-weighted job masses, in point order.
    /// New block IDs keep unit weight before each comuna's total is restored.
    pub fn weight_job_masses<'a>(&self, points: impl Iterator<Item = (&'a str, &'a str)> + Clone, mass: &mut [f64]) {
        if self.transport_factors.is_empty() { return; }
        let mut totals: HashMap<&str, (f64, f64)> = HashMap::new();
        for ((id, cut), value) in points.clone().zip(mass.iter_mut()) {
            let sum = totals.entry(cut).or_default();
            sum.0 += *value;
            *value *= self.transport_factors.get(id).copied().unwrap_or(1.0);
            sum.1 += *value;
        }
        for ((_, cut), value) in points.zip(mass) {
            let (before, after) = totals[cut];
            if after > 0.0 { *value *= before / after; }
        }
    }

    fn place(&self, homes: &[PointMass], jobs: &[PointMass], active: &HashSet<String>) -> Vec<Mass> {
        let open: Vec<_> = self.stations.iter().map(|s| active.contains(&s.id)).collect();
        let all_open = open.iter().all(|&o| o);
        let links: Vec<_> = self.bus.stops.iter().map(|s| s.links.iter().find(|(i, _)| open[*i]).copied()).collect();
        let mut masses = vec![Mass::default(); self.stations.len()];
        for (half, points) in [homes, jobs].into_iter().enumerate() {
            for &(x, y, mass) in points {
                if mass == 0.0 { continue; }
                let fresh;
                let point = if let Some(p) = self.points.get(&(x.to_bits(), y.to_bits())) {
                    p
                } else {
                    fresh = PointAccess::new(&self.stations, &self.bus, &links, x, y);
                    &fresh
                };
                let walks = || point.walk.iter().copied().filter(|&(i, _)| open[i]);
                let sum = if all_open { point.walk_sum } else { walks().map(|(_, w)| w).sum() };
                if sum > 0.0 {
                    for (i, w) in walks() { masses[i].walk[half] += mass * w / sum; }
                } else {
                    // A closure can only remove bus routes. An open winner is still fastest.
                    let bus = if point.walk.is_empty() && point.bus.is_none_or(|i| open[i]) {
                        point.bus
                    } else {
                        self.bus.fastest(x, y, &links, &self.stations)
                    };
                    if let Some(i) = bus { masses[i].bus[half] += mass; }
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
