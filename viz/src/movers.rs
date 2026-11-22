//! Replay of an engine result. Positions between departure bins are display interpolation.
#[path = "events.rs"]
pub(crate) mod events;
pub(crate) mod clock;
pub(crate) mod follow;
mod fleet;
mod motion;
mod car_geometry;
mod road_water;
mod road_edits;
mod articulated;
mod instanced;
mod levels;
mod riders;
mod stations;
mod street_people;
use riders::{RideKey, RiderNetwork, RiderPlan, RiderState};
use stations::Station;
use riders::bus_load;

use crate::{
    city::{Surface, terrain_from_xz, world},
    net::Scenario,
};
use bevy::{
    asset::RenderAssetUsages,
    image::{CompressedImageFormats, ImageSampler, ImageType},
    mesh::Indices,
    render::render_resource::PrimitiveTopology,
    camera::primitives::{Frustum, Sphere as Bounds},
    prelude::*,
};
use serde_json::Value;
use std::{collections::HashMap, fs, io::Read, sync::Arc};

// At street level one walker represents 48 workers and other figures represent 96.
const PERSONS_PER_STREET_WALKER: f64 = 48.0;
const PERSONS_PER_MOVER: f64 = 96.0;
// From 1 km and 5 km, use 4× and 12× as many street figures from the same group count.
const STREET_DENSITY: [f64; 3] = [1., 4., 12.];
const CITY_SECONDS_PER_SECOND: f32 = 8.;
const BIN: f32 = 1800.0;
const DAY: f32 = 48.0 * BIN;

pub struct MoversPlugin;
impl Plugin for MoversPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(instanced::MarksPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, levels::toggle_tracks)
            .init_resource::<Replay>()
            .add_systems(Update, (read_result, draw, stations::position_closed_signs).chain());
    }
}

fn json(file: &str) -> Value {
    serde_json::from_slice(&fs::read(file).unwrap()).unwrap()
}

fn point(p: &Value) -> Vec3 {
    world(p[0].as_f64().unwrap(), p[1].as_f64().unwrap())
}

#[derive(Clone, Copy, PartialEq)]
enum Vehicle {
    Car,
    Bus,
    Bicycle,
    Walk,
    Rail,
}

impl Vehicle {
    // Display speeds in metres per city second. GTFS supplies counts per bin, not a trip clock.
    fn speed(self) -> f32 {
        match self {
            Self::Car => 9.,
            Self::Bus => 6.,
            Self::Bicycle => 4.,
            Self::Walk => 1.4,
            Self::Rail => 11.,
        }
    }
}

struct Leg {
    scheduled: Option<usize>,
    walk_path: Option<Arc<Path>>,
    end: usize,
    vehicle: Vehicle,
}

struct Path {
    crossings: Vec<(f32, f32, road_water::Crossing)>,
    points: Vec<Vec3>,
    metres: Vec<f32>,
    seconds: Vec<f32>,
    legs: Vec<Leg>,
}

// Both distance and time are cumulative. Coaches use distance; multimodal riders use time.
impl Path {
    fn new(points: Vec<Vec3>, legs: Vec<Leg>) -> Arc<Self> {
        let mut metres = vec![0.];
        let mut seconds = vec![0.];
        let mut leg = 0;
        for i in 1..points.len() {
            while legs[leg].end < i {
                leg += 1;
            }
            let step = points[i].distance(points[i - 1]);
            metres.push(metres[i - 1] + step);
            seconds.push(seconds[i - 1] + step / legs[leg].vehicle.speed());
        }
        Arc::new(Self {
            crossings: Vec::new(),
            points,
            metres,
            seconds,
            legs,
        })
    }

    fn single(points: Vec<Vec3>, vehicle: Vehicle) -> Arc<Self> {
        let end = points.len() - 1;
        Self::new(points, vec![Leg { end, vehicle, scheduled: None, walk_path: None }])
    }

    fn nearest(&self, pos: Vec3) -> (Vec3, Vec3, f32) {
        let (i, p) = self
            .points
            .windows(2)
            .enumerate()
            .filter(|(_, p)| p[0] != p[1])
            .map(|(i, p)| (i, foot(pos, p[0], p[1])))
            .min_by(|(_, a), (_, b)| a.distance_squared(pos).total_cmp(&b.distance_squared(pos)))
            .unwrap();
        (
            p,
            (self.points[i + 1] - self.points[i]).normalize(),
            self.metres[i] + p.distance(self.points[i]),
        )
    }

    fn at(&self, value: f32, axis: &[f32]) -> Option<(Vec3, Vec3, Vehicle)> {
        let last = *axis.last()?;
        if value < 0. || value > last || last == 0. {
            return None;
        }
        let end = if value == last {
            axis.partition_point(|&v| v < last)
        } else {
            axis.partition_point(|&v| v <= value)
        };
        let from = self.points[end - 1];
        let to = self.points[end];
        let t = (value - axis[end - 1]) / (axis[end] - axis[end - 1]);
        let leg = self.legs.partition_point(|leg| leg.end < end);
        let mut pos = from.lerp(to, t);
        pos.y = terrain_from_xz(pos.x, pos.z);
        Some((pos, (to - from).normalize(), self.legs[leg].vehicle))
    }
}

struct Call {
    arrival: f32,
    departure: f32,
    distance: f32,
    station: usize,
}

struct RailRun {
    calls: Vec<Call>,
}

impl RailRun {
    // Hold the published arrival and departure. Ramp for up to six seconds at each end.
    fn distance(&self, now: f32) -> Option<f32> {
        let calls = &self.calls;
        if now < calls[0].arrival || now > calls.last()?.departure {
            return None;
        }
        let i = calls.partition_point(|call| call.departure < now);
        let next = &calls[i];
        if now >= next.arrival {
            return Some(next.distance);
        }
        let prev = &calls[i - 1];
        let duration = next.arrival - prev.departure;
        let t = now - prev.departure;
        let ramp = 6.0_f32.min(duration * 0.5);
        let travelled = if t < ramp {
            t * t / (2. * ramp)
        } else if t > duration - ramp {
            duration - ramp - (duration - t).powi(2) / (2. * ramp)
        } else {
            t - ramp * 0.5
        };
        Some(prev.distance + (next.distance - prev.distance) * travelled / (duration - ramp))
    }
}

struct RailLine {
    levels: levels::Levels,
    path: Arc<Path>,
    runs: Vec<RailRun>,
    stops: Vec<Option<f32>>,
    longest: f32,
    stations: Vec<String>,
}

impl RailLine {
    fn load(row: &Value, timetable: &Value, track_levels: &Value) -> Self {
        let path = Path::single(points(&row["path"]), Vehicle::Rail);
        let table = timetable["lines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|line| line["route"] == row["route"] && line["direction"] == row["direction"])
            .unwrap();
        let runs: Vec<_> = table["trips"]
            .as_array()
            .unwrap()
            .iter()
            .map(|trip| {
                let calls = trip[1]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|call| {
                        let vertex = call[2].as_f64().unwrap();
                        let a = path.metres[vertex.floor() as usize];
                        let b = path.metres[vertex.ceil() as usize];
                        Call {
                            arrival: call[0].as_f64().unwrap() as f32,
                            departure: call[1].as_f64().unwrap() as f32,
                            distance: a + (b - a) * vertex.fract() as f32,
                            station: call[3].as_u64().unwrap() as usize,
                        }
                    })
                    .collect();
                RailRun { calls }
            })
            .collect();
        let longest = runs
            .iter()
            .map(|r| r.calls.last().unwrap().departure - r.calls[0].arrival)
            .fold(0., f32::max);
        let mut stops = vec![None; row["parent_station_ids"].as_array().unwrap().len()];
        for run in &runs {
            for call in &run.calls {
                stops[call.station] = Some(call.distance);
            }
        }
        Self {
            levels: levels::Levels::load(row, track_levels, &path),
            path,
            runs,
            stops,
            longest,
            stations: row["parent_station_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| id.as_str().unwrap().to_owned())
                .collect(),
        }
    }

    // The GTFS gives a centreline shared by opposite directions. Project the car's two
    // bogies onto it so its yaw and the side offset change continuously through corners.
    fn centre(&self, distance: f32) -> Vec3 {
        let axis = &self.path.metres;
        let end = axis
            .partition_point(|&d| d <= distance)
            .clamp(1, axis.len() - 1);
        let start = end - 1;
        let a = self.path.points[start];
        let b = self.path.points[end];
        if axis[end] == axis[start] {
            return b;
        }
        let mut p = a.lerp(b, (distance - axis[start]) / (axis[end] - axis[start]));
        p.y = terrain_from_xz(p.x, p.z);
        p
    }

    fn coach(&self, distance: f32) -> (Vec3, Vec3) {
        let front = self.centre(distance + 5.);
        let rear = self.centre(distance - 5.);
        let dir = (front - rear).normalize_or_zero();
        let p = (front + rear) * 0.5 + right(dir) * 2.5;
        (Vec3::new(p.x, terrain_from_xz(p.x, p.z) + self.levels.at(distance) + 0.45, p.z), dir)
    }
}

type Table = HashMap<String, Arc<Path>>;

#[derive(Resource)]
struct Routes {
    water: road_water::Water,
    cars: Table,
    bikes: Table,
    public: Table,
    local: HashMap<LocalKey, Arc<Path>>,
    tracks: Vec<RailLine>,
    riders: RiderNetwork,
    entrances: stations::Entrances,
    station_walk_routes: street_people::WalkRoutes,
    streets: StreetIndex,
    cycleways: StreetIndex,
}

fn points(row: &Value) -> Vec<Vec3> {
    row.as_array().unwrap().iter().map(point).collect()
}

fn table(file: &str, mode: Vehicle) -> Table {
    json(file)
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, row)| (key.clone(), Path::single(points(row), mode)))
        .collect()
}

fn public_walk(leg: &Value, source: &Value) -> Option<Arc<Path>> {
    let mut walk = Path::single(points(leg.get("walk_path")?), Vehicle::Walk);
    if leg["kind"] == "egress"
        && leg["snap_m"][1].as_f64() == Some(0.)
        && source["entrance_osm_node"].is_u64()
        && let Some(door) = source["entrance"].as_array()
        && let [.., previous, end] = Arc::get_mut(&mut walk).unwrap().points.as_mut_slice()
    {
        // The footprint pack rounds OSM vertices to 1e-5 degrees. Keep the
        // person's centre one body radius outside that door. Preserve walk timing.
        let grid = |coordinate: &Value| {
            (coordinate.as_f64().unwrap() * 100_000.).round_ties_even() / 100_000.
        };
        let facade = world(grid(&door[0]), grid(&door[1]));
        *end = facade + (*previous - facade).clamp_length_max(0.25);
    }
    Some(walk)
}

fn public_path(row: &Value, riders: &RiderNetwork) -> Arc<Path> {
    let mut path = points(&row["path"]);
    // A walk to or from a Metro leg touches a published entrance. A direct interchange
    // between Metro lines stays inside the station.
    let segs = row["segments"].as_array().unwrap();
    let scheduled: Vec<_> = segs.iter().map(|s| riders.leg(row, s)).collect();
    for (i, seg) in segs.iter().enumerate().filter(|(_,s)| s["mode"]=="metro") {
        let id = scheduled[i].unwrap();
        let board = seg["start"].as_u64().unwrap() as usize;
        let exit = seg["end"].as_u64().unwrap() as usize;
        let previous = scheduled[..i].iter().rev().flatten().next().copied();
        let next = scheduled[i + 1..].iter().flatten().next().copied();
        if i > 0 && segs[i - 1]["mode"] == "walk" && !riders.interchange(previous, Some(id)) {
            path[board] = riders.gate(id, false, path[board.saturating_sub(1)]);
        }
        if i + 1 < segs.len() && segs[i + 1]["mode"] == "walk" && !riders.interchange(Some(id), next) {
            path[exit] = riders.gate(id, true, path[(exit + 1).min(path.len() - 1)]);
        }
    }
    let legs = segs.iter().zip(scheduled)
        .map(|(leg, scheduled)| Leg {
            scheduled,
            walk_path: public_walk(leg, &row["source"]),
            end: leg["end"].as_u64().unwrap() as usize,
            vehicle: match leg["mode"].as_str().unwrap() {
                "walk" => Vehicle::Walk,
                "bus" => Vehicle::Bus,
                "metro" | "rail" => Vehicle::Rail,
                mode => panic!("unknown public route mode {mode}"),
            },
        })
        .collect();
    Path::new(path, legs)
}

type LocalKey = (u64, String, [u64; 4]);

fn local_key(id: u64, mode: &str, origin: &Value, destination: &Value) -> LocalKey {
    let coords = [&origin[0], &origin[1], &destination[0], &destination[1]].map(|v| {
        let n = v.as_f64().unwrap();
        // The GTFS pack preserves endpoints to five decimal places. OSRM keeps the exact source fields.
        let endpoint = if mode == "public" {
            format!("{n:.5}").parse::<f64>().unwrap()
        } else {
            n
        };
        endpoint.to_bits()
    });
    (id, mode.to_owned(), coords)
}

fn gzip_json(file: &str) -> Value {
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(fs::File::open(file).unwrap())
        .read_to_end(&mut bytes)
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn local_routes(riders: &RiderNetwork, water: &road_water::Pack) -> HashMap<LocalKey, Arc<Path>> {
    let mut local = HashMap::new();
    let active = gzip_json("web/data/local_mover_routes.json.gz");
    for g in active["groups"].as_array().unwrap() {
        let mode = g["mode"].as_str().unwrap();
        let kind = match mode {
            "car" => Vehicle::Car,
            "bicycle" => Vehicle::Bicycle,
            "walk" => Vehicle::Walk,
            _ => unreachable!(),
        };
        let line = points(&active["paths"][g["route"].as_u64().unwrap() as usize]);
        let key = local_key(g["id"].as_u64().unwrap(), mode, &g["origin"], &g["destination"]);
        let mut path = Path::single(line, kind);
        if kind == Vehicle::Car {
            road_water::set_crossings(&mut path, water.local.get(&g["route"].to_string()));
        }
        local.insert(key, path);
    }
    let gtfs = gzip_json("web/data/local_public_routes.json.gz");
    let records = gtfs["paths"].as_array().unwrap();
    let paths: Vec<_> = records.iter().map(|r| public_path(r, riders)).collect();
    for state in ["base", "scenario"] {
        for (id, index) in gtfs["groups"][state].as_object().unwrap() {
            let i = index.as_u64().unwrap() as usize;
            let line = records[i]["path"].as_array().unwrap();
            local.insert(
                local_key(id.parse().unwrap(), "public", &line[0], line.last().unwrap()),
                paths[i].clone(),
            );
        }
    }
    local
}

impl Routes {
    fn load() -> Self {
        let track_levels = gzip_json("web/data/metro_tracks.geojson.gz");
        let timetable = gzip_json("web/data/metro_schedule.json.gz");
        let tracks: Vec<_> = json("web/data/transit_paths.json")
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["mode"] == "metro")
            .map(|row| RailLine::load(row, &timetable, &track_levels))
            .collect();
        let entrances = stations::entrances();
        let riders = RiderNetwork::load(&tracks, &entrances);
        let public = json("web/data/public_routes.json")
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, row)| (key.clone(), public_path(row, &riders)))
            .collect();
        let water: road_water::Pack = serde_json::from_reader(flate2::read::GzDecoder::new(fs::File::open("web/data/car_water_spans.json.gz").unwrap())).unwrap();
        let mut local = local_routes(&riders, &water);
        let mut cars = table("web/data/car_routes.json", Vehicle::Car);
        for (key, path) in &mut cars { road_water::set_crossings(path, water.od.get(key)); }
        car_geometry::restore(cars.values_mut().chain(
            local.iter_mut().filter(|(key, _)| key.1 == "car").map(|(_, path)| path),
        ));
        Self {
            public,
            local,
            tracks,
            riders,
            entrances,
            station_walk_routes: street_people::WalkRoutes::load(),
            water: road_water::Water::new(water),
            cars,
            bikes: table("web/data/bicycle_routes.json", Vehicle::Bicycle),
            streets: StreetIndex::load("data/mvp/roads.geojson", true),
            cycleways: StreetIndex::load("data/mvp/cycleways.geojson", false),
        }
    }

    fn group_path(&self, row: &Value) -> Option<Arc<Path>> {
        let pair = format!(
            "{}-{}",
            row["origin_cut"].as_str()?,
            row["destination_cut"].as_str()?
        );
        let mode = row["mode"].as_str()?;
        let key = local_key(row["id"].as_u64()?, mode, &row["origin"], &row["destination"]);
        if let Some(path) = self.local.get(&key) {
            return Some(path.clone());
        }
        let table = match mode {
            "car" => &self.cars,
            "bicycle" => &self.bikes,
            "public" => return self.public.get(&pair).cloned(),
            _ => return None,
        };
        if let Some(path) = table.get(&pair) {
            return Some(path.clone());
        }
        let origin = point(&row["origin"]);
        let destination = point(&row["destination"]);
        // Missing OD routes show only a mapped local segment at the origin.
        match mode {
            "bicycle" => self.cycleways.local_path(origin, destination, Vehicle::Bicycle),
            _ => self.streets.local_path(origin, destination, Vehicle::Car),
        }
    }
}

// Index actual street segments for local vehicle motion.
struct StreetIndex(HashMap<(i32, i32), Vec<(Vec3, Vec3)>>);

fn cell(p: Vec3) -> (i32, i32) {
    ((p.x / 200.).floor() as i32, (p.z / 200.).floor() as i32)
}

fn foot(p: Vec3, a: Vec3, b: Vec3) -> Vec3 {
    let d = b - a;
    a + d * ((p - a).dot(d) / d.length_squared()).clamp(0., 1.)
}

impl StreetIndex {
    fn load(file: &str, streets: bool) -> Self {
        let mut cells = HashMap::<_, Vec<_>>::new();
        for f in json(file)["features"].as_array().unwrap() {
            if streets
                && matches!(
                    f["properties"]["kind"].as_str(),
                    Some("motorway" | "motorway_link" | "trunk" | "trunk_link")
                )
            {
                continue;
            }
            for pair in points(&f["geometry"]["coordinates"]).windows(2) {
                let (a, b) = (pair[0], pair[1]);
                if a == b {
                    continue;
                }
                let (x0, z0) = cell(a.min(b));
                let (x1, z1) = cell(a.max(b));
                for x in x0..=x1 {
                    for z in z0..=z1 {
                        cells.entry((x, z)).or_default().push((a, b));
                    }
                }
            }
        }
        Self(cells)
    }

    fn local_path(&self, origin: Vec3, destination: Vec3, vehicle: Vehicle) -> Option<Arc<Path>> {
        let (x, z) = cell(origin);
        let mut closest = None;
        let mut distance = 200_f32.powi(2);
        for i in x - 1..=x + 1 {
            for j in z - 1..=z + 1 {
                for &(a, b) in self.0.get(&(i, j)).into_iter().flatten() {
                    let p = foot(origin, a, b);
                    let d = p.distance_squared(origin);
                    if d < distance {
                        let (start, end) = if (b - a).dot(destination - origin) >= 0. {
                            (a, b)
                        } else {
                            (b, a)
                        };
                        let direction = (end - start).normalize();
                        let from = p - direction * p.distance(start).min(55.);
                        let to = p + direction * p.distance(end).min(55.);
                        closest = Some((from, to));
                        distance = d;
                    }
                }
            }
        }
        closest.map(|(a, b)| Path::single(vec![a, b], vehicle))
    }
}

struct Group {
    cars: Vec<u16>,
    itineraries: Vec<RiderPlan>,
    duration: f32,
    path: Arc<Path>,
    copies: [usize; 3],
    phase: f32,
    changed: [usize; 3],
    tint: Paint,
    lifecycle: Life,
}

#[derive(Clone, Copy)]
enum Life {
    Stable,
    Enter,
    Leave,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
enum Paint {
    #[default]
    Plain,
    Gain,
    Loss,
    Rerouted,
}

#[derive(Resource, Default)]
struct Replay {
    base_bus_loads: Option<bus_load::BaseBusLoads>,
    road_edits: road_edits::TrafficEdits,
    departures: Vec<Vec<Group>>,
    platforms: Vec<Station>,
    longest: f32,
    clock_offset: Option<f32>,
    metro_closure: Option<std::ops::Range<i32>>,
    revision_started: f32,
}

fn read_result(
    mut commands: Commands,
    signs: Query<Entity, With<stations::StationPin>>,
    mut base_access: Local<Option<Value>>,
    time: Res<Time>,
    rate: Res<PlaybackRate>,
    result: Res<Scenario>,
    routes: Res<Routes>,
    fleet: Res<fleet::Fleet>,
    mut replay: ResMut<Replay>,
    mut road_catalog: Local<road_edits::Catalog>,
) {
    if !result.is_changed() || result.0.is_null() {
        return;
    }
    let r = &result.0;
    replay.road_edits = road_edits::TrafficEdits::from_result(r, &mut road_catalog);
    let metro_removed = r["applied"].as_array().into_iter().flatten().any(|change| {
        change["kind"] == "line"
            && change["op"] == "remove"
            && change["target"]["type"] == "mode"
            && change["target"]["id"] == "metro"
    });
    replay.metro_closure = metro_removed.then(|| match r["closure_window"]["half_hours"].as_array() {
        Some(window) => window[0].as_u64().unwrap() as i32..window[1].as_u64().unwrap() as i32,
        None => 0..48,
    });
    let stations = r["stations"].as_array().unwrap();
    let elapsed = time.elapsed_secs() * rate.0;
    if let Some(bin) = r["scene"]["half_hour"].as_u64() {
        // Replay each result from the same selected scene time so the two runs compare.
        replay.clock_offset = Some(bin as f32 * BIN - elapsed);
    } else if replay.clock_offset.is_none() {
        let mut boardings = [0.; 48];
        for station in stations {
            for (total, count) in boardings
                .iter_mut()
                .zip(station["base_half_hour"].as_array().unwrap())
            {
                *total += count.as_f64().unwrap();
            }
        }
        let peak = (0..boardings.len())
            .rev()
            .max_by(|&a, &b| boardings[a].total_cmp(&boardings[b]))
            .unwrap();
        replay.clock_offset = Some(peak as f32 * BIN - elapsed);
    }
    let mut closed = std::collections::HashSet::new();
    for change in r["applied"].as_array().into_iter().flatten().filter(|c| c["kind"] == "station") {
        let state = &change["understood"];
        // Planned lines, corridors and proposed points use their own IDs and have no GTFS runs.
        if state.get("id").is_some() {
            continue;
        }
        let Some(name) = state["station"].as_str() else { continue };
        let Some(station) = stations.iter().find(|s| s["name"] == name) else { continue };
        let id = station["id"].as_str().unwrap();
        if state["state"] == "closed" {
            closed.insert(id);
        } else {
            closed.remove(id);
        }
    }
    for entity in &signs {
        commands.entity(entity).despawn();
    }
    for station in stations.iter().filter(|s| closed.contains(s["id"].as_str().unwrap())) {
        stations::spawn_closed_sign(&mut commands, station);
    }
    for row in stations.iter().filter(|s| matches!(s["id"].as_str(), Some("TB-TOB" | "GO" | "LE-LEN"))) {
        stations::spawn_name(&mut commands, row);
    }
    let mut deltas = HashMap::new();
    for d in r["trips_delta"].as_array().into_iter().flatten() {
        deltas.insert(
            d[0].as_u64().unwrap(),
            (d[1].as_f64().unwrap(), d[2].as_u64().unwrap()),
        );
    }
    replay.base_bus_loads = (!deltas.is_empty()).then(|| bus_load::BaseBusLoads::new(r["trips_base"].as_array().unwrap(), &routes));
    replay.departures = (0..48).map(|_| Vec::new()).collect();
    replay.longest = 0.;
    replay.revision_started = time.elapsed_secs();
    // A removed ID has no scenario row. Its last pose comes from the matching base row.
    let removed = r["trips_base"].as_array().unwrap().iter().filter(|row| {
        deltas
            .get(&row["id"].as_u64().unwrap())
            .is_some_and(|&(_, flags)| flags & 16 != 0)
    });
    for row in r["trips_scenario"].as_array().unwrap().iter().chain(removed) {
        let Some(path) = routes.group_path(row) else {
            continue;
        };
        let id = row["id"].as_u64().unwrap();
        // Stable fractional rounding keeps the same represented people at the same phase on revision.
        let seed = (id as u32).wrapping_mul(2654435761) as f64 / 4294967296.;
        let scale = if row["mode"] == "walk" {
            PERSONS_PER_STREET_WALKER
        } else {
            PERSONS_PER_MOVER
        };
        let density = match row["mode"].as_str() {
            Some("car" | "walk" | "bicycle") => STREET_DENSITY,
            _ => [1.; 3],
        };
        let count = row["count"].as_f64().unwrap();
        let copies = density.map(|factor| (count * factor / scale + seed).floor() as usize);
        let (delta, flags) = deltas.get(&id).copied().unwrap_or_default();
        let tint = if flags & 8 != 0 || delta > 0. {
            Paint::Gain
        } else if delta < 0. {
            Paint::Loss
        } else if flags != 0 {
            Paint::Rerouted
        } else {
            Paint::Plain
        };
        let changed = if flags != 0 {
            copies
        } else {
            density.map(|factor| (delta.abs() * factor / scale).ceil() as usize)
        };
        let bin = row["half_hour"].as_u64().unwrap() as usize;
        let itineraries: Vec<_> = if row["mode"] == "public" {
            let legs = routes.riders.accessible_legs(&path, &closed);
            (0..copies[0])
                .map(|j| {
                    let start = bin as f32 * BIN + departure_offset(seed as f32, j);
                    RiderPlan::build(&path, legs, start, j, &routes)
                })
                .collect()
        } else {
            Vec::new()
        };
        let duration = itineraries
            .iter()
            .map(RiderPlan::duration)
            .fold(*path.seconds.last().unwrap(), f32::max);
        replay.longest = replay.longest.max(duration);
        replay.departures[bin].push(Group {
            cars: if row["mode"] == "car" {
                fleet.cars(row["origin_cut"].as_str().unwrap(), id, copies[2])
            } else {
                Vec::new()
            },
            itineraries,
            duration,
            path,
            copies,
            phase: seed as f32,
            changed,
            tint,
            lifecycle: if flags & 16 != 0 {
                Life::Leave
            } else if flags & 8 != 0 {
                Life::Enter
            } else {
                Life::Stable
            },
        });
    }
    // Old cached replays contain scenario shares only. Use the matching engine baseline
    // when the client opens directly on a closure. A live baseline supersedes the pack.
    let reference = base_access.get_or_insert_with(|| gzip_json("web/data/station_access_baseline.json.gz"));
    if r["applied"].as_array().is_some_and(Vec::is_empty) {
        for row in stations.iter().filter(|s| matches!(s["id"].as_str(), Some("GO" | "LE-LEN" | "TB-TOB"))) {
            reference[row["id"].as_str().unwrap()] = serde_json::json!({"bins": row["base_half_hour"], "origins": row["access_origins"]});
        }
    }
    replay.platforms = stations
        .iter()
        .filter(|station| routes.entrances.contains_key(station["id"].as_str().unwrap()))
        .map(|row| {
            let packed = &reference[row["id"].as_str().unwrap()];
            let origins = row.get("base_access_origins").or_else(|| (packed["bins"] == row["base_half_hour"]).then(|| &packed["origins"]));
            Station::new(row, origins, &routes)
        })
        .collect();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Model {
    Car(u16),
    CarChange,
    Bus,
    BusTail,
    BusJoint,
    BusInterior,
    BusPassenger,
    StationEntries,
    WalkTrace,
    Bike,
    Person,
    Walking(u8),
    Coach,
    Window,
    Platform,
    Entrance,
}

#[derive(Resource)]
struct Models {
    ink: bool,
    mesh: HashMap<Model, Handle<Mesh>>,
    paint: HashMap<(Model, Paint), Handle<StandardMaterial>>,
}

type Rgb = [u8; 3];
const CAR_INK: Rgb = [22, 75, 86];
const PERSON_INK: Rgb = [113, 69, 117];
const PAPER: Rgb = [249, 248, 244];
const GLASS: Rgb = [36, 55, 70];
// Same order as the atlas generated by tools/pack/car_sheets.py.
const CAR_SWATCHES: [Rgb; 15] = [
    [236, 235, 226], [111, 119, 121], [184, 194, 197], [47, 50, 54], [184, 54, 52],
    [54, 91, 145], [126, 88, 67], [66, 106, 83], [197, 179, 146], [214, 125, 55],
    [222, 183, 60], [187, 151, 85], [129, 176, 199], [211, 143, 158], [124, 91, 145],
];

fn painted(mut mesh: Mesh, offset: [f32; 3], rgb: Rgb) -> Mesh {
    let color = Color::srgb_u8(rgb[0], rgb[1], rgb[2]).to_linear().to_f32_array();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![color; mesh.count_vertices()]);
    mesh.translated_by(Vec3::from_array(offset))
}

fn block(size: [f32; 3], at: [f32; 3], rgb: Rgb) -> Mesh {
    painted(Mesh::from(Cuboid::new(size[0], size[1], size[2])), at, rgb)
}

fn oval(radii: [f32; 3], at: [f32; 3], rgb: Rgb) -> Mesh {
    painted(
        Sphere::new(1.)
            .mesh()
            .ico(1)
            .unwrap()
            .scaled_by(Vec3::from_array(radii)),
        at,
        rgb,
    )
}

fn shape(parts: impl IntoIterator<Item = Mesh>) -> Mesh {
    let mut parts = parts.into_iter();
    let mut shape = parts.next().unwrap();
    for mesh in parts {
        shape.merge(&mesh).unwrap();
    }
    shape
}

// Every art pixel is 12 cm in all three axes. One horizontal quad samples one PNG tile.
const ART_PIXEL: f32 = 0.12;
const BUS_FRONT_TO_HINGE: f32 = 5.6;
const BUS_TAIL_FRACTION: f32 = 0.58;
const BUS_TAIL_HALF: f32 = 96. * ART_PIXEL * BUS_TAIL_FRACTION * 0.5;

fn sprite_stack(bytes: &[u8], slices: u32, grid: [u32; 2]) -> (Mesh, Image) {
    sprite_stack_width(bytes, slices, grid, 1.)
}

fn sprite_stack_width(bytes: &[u8], slices: u32, [columns, rows]: [u32; 2], fraction: f32) -> (Mesh, Image) {
    let image = Image::from_buffer(
        bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::nearest(),
        RenderAssetUsages::default(),
    )
    .unwrap();
    let x = (image.width() / columns / slices) as f32 * ART_PIXEL * fraction / 2.;
    let z = (image.height() / rows) as f32 * ART_PIXEL / 2.;
    let mut positions = Vec::new();
    let mut uv = Vec::new();
    let mut indices = Vec::new();
    for slice in 0..slices {
        let y = slice as f32 * ART_PIXEL;
        let u_left = slice as f32 / slices as f32;
        let u_right = (slice as f32 + fraction) / slices as f32;
        positions.extend([[-x, y, -z], [-x, y, z], [x, y, z], [x, y, -z]]);
        uv.extend([[u_left, 0.], [u_left, 1.], [u_right, 1.], [u_right, 0.]]);
        let i = slice * 4;
        indices.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
    }
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; positions.len()])
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_indices(Indices::U32(indices));
    (mesh, image)
}

#[derive(Resource)]
struct FixedClock(Option<f32>);

#[derive(Resource)]
struct PlaybackRate(f32);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let seconds = std::env::var("SANTIAGO_CLOCK_SECONDS").ok().map(|v| {
        v.parse::<f32>()
            .expect("SANTIAGO_CLOCK_SECONDS must be seconds after midnight")
    });
    commands.insert_resource(FixedClock(seconds));
    let rate = std::env::var("SANTIAGO_PLAYBACK_RATE").map_or(CITY_SECONDS_PER_SECOND, |s| {
        s.parse().expect("SANTIAGO_PLAYBACK_RATE must be a number")
    });
    assert!(rate.is_finite() && rate > 0., "Playback rate must be finite and positive");
    commands.insert_resource(PlaybackRate(rate));
    let ink = std::env::var("SANTIAGO_PIXEL").as_deref() != Ok("0");
    let sprites: [(Model, &[u8], u32); 4] = [
        (Model::BusTail, include_bytes!("../assets/bus.png"), 27),
        (Model::Bus, include_bytes!("../assets/bus.png"), 27),
        (Model::Coach, include_bytes!("../assets/metro.png"), 28),
        (Model::Bike, include_bytes!("../assets/bicycle.png"), 16),
    ];
    let models = [
        (Model::BusInterior, bus_load::interior_mesh(), None),
        (Model::BusPassenger, bus_load::passenger_mesh(), None),
        (Model::WalkTrace, block([1., 0.20, 1.], [0.; 3], [255; 3]), None),
        (Model::StationEntries, street_people::entry_disc(), None),
        (Model::Person, motion::pedestrian(None, ink), None),
        (Model::BusJoint, shape((-2..=2).map(|i| {
            block([0.22, 2.45, 2.45], [i as f32 * 0.21, 1.5, 0.], if i % 2 == 0 { GLASS } else { [70, 78, 82] })
        })), None),
        // A pair of road marks carries the scenario colour beside the car's actual paint.
        (Model::CarChange, shape([-1.5, 1.5].map(|z| {
            painted(Plane3d::default().mesh().size(3.6, 0.28).build(), [0., 0.04, z], [255; 3])
        })), None),
        (
            Model::Window,
            painted(
                Plane3d::default().mesh().size(1., 1.35).build(),
                [0.; 3],
                [251, 183, 67],
            ),
            None,
        ),
        (Model::Entrance, shape([
            block([4.4,0.25,3.2], [0.,0.13,0.], [52,56,60]),
            block([4.4,0.95,0.38], [0.,0.48,1.55], [179,69,61]),
            block([4.4,0.95,0.38], [0.,0.48,-1.55], [179,69,61]),
            block([0.32,0.95,3.2], [2.06,0.48,0.], [179,69,61]),
        ]), None),
        (
            Model::Platform,
            block([1., 0.55, 1.], [0., 0.28, 0.], [132, 137, 142]),
            None,
        ),
    ]
    .into_iter()
    .chain((0..motion::GAIT_FRAMES).map(|frame| (Model::Walking(frame), motion::pedestrian(Some(frame), ink), None)))
    .chain(sprites.into_iter().map(|(model, bytes, layers)| {
        let fraction = if model == Model::BusTail { BUS_TAIL_FRACTION } else { 1. };
        let (mesh, image) = sprite_stack_width(bytes, layers, [1, 1], fraction);
        (model, mesh, Some(images.add(image)))
    }));
    let mut mesh = HashMap::new();
    let mut paint = HashMap::new();
    for (model, geometry, texture) in models {
        mesh.insert(model, meshes.add(geometry));
        for (tint, [r, g, b]) in [
            (Paint::Plain, [255, 255, 255]),
            (Paint::Gain, [103, 255, 169]),
            (Paint::Loss, [255, 136, 144]),
            (Paint::Rerouted, [144, 217, 255]),
        ] {
            let sprite = texture.is_some();
            let person_ink = (ink && matches!(model, Model::Person | Model::Walking(_))) || matches!(model, Model::StationEntries | Model::BusPassenger);
            let accent = ink && tint != Paint::Plain;
            let color = if accent {
                Color::linear_rgb(0.85, 0.13, 0.09)
            } else if person_ink {
                Color::srgb_u8(PERSON_INK[0], PERSON_INK[1], PERSON_INK[2])
            } else {
                Color::srgb_u8(r, g, b)
            };
            paint.insert(
                (model, tint),
                mats.add(StandardMaterial {
                    base_color: color,
                    base_color_texture: texture.clone(),
                    unlit: sprite || person_ink || accent || matches!(model, Model::CarChange | Model::BusInterior),
                    alpha_mode: if person_ink || accent {
                        AlphaMode::Blend
                    } else if sprite {
                        AlphaMode::Mask(0.5)
                    } else {
                        AlphaMode::Opaque
                    },
                    perceptual_roughness: 0.8,
                    ..default()
                }),
            );
        }
    }
    let mut models = Models { ink, mesh, paint };
    let fleet = fleet::Fleet::load(&mut models, &mut meshes, &mut mats, &mut images);
    commands.insert_resource(fleet);
    commands.insert_resource(models);
    let routes = Routes::load();
    let track_material = mats.add(StandardMaterial {
        perceptual_roughness: 0.85,
        ..default()
    });
    for line in &routes.tracks {
        for (underground, mesh) in levels::track_meshes(line).into_iter().enumerate() {
            if mesh.positions.is_empty() {continue;}
            let mut entity=commands.spawn((Mesh3d(meshes.add(mesh.finish())), MeshMaterial3d(track_material.clone())));
            if underground == 1 { entity.insert((levels::Underground, Visibility::Hidden)); }
        }
    }
    commands.insert_resource(routes);
}

struct Train<'a> {
    run_index: usize,
    line: &'a RailLine,
    distance: f32,
    pos: Vec3,
    riders: usize,
    paint: Paint,
}

fn trains(routes: &Routes, now: f32) -> Vec<Vec<Train<'_>>> {
    let mut out = Vec::new();
    for line in &routes.tracks {
        let mut running = Vec::new();
        let first = line
            .runs
            .partition_point(|run| run.calls[0].arrival < now - line.longest);
        let last = line.runs.partition_point(|run| run.calls[0].arrival <= now);
        for (offset,run) in line.runs[first..last].iter().enumerate() {
            if let Some(distance) = run.distance(now) {
                running.push(Train {
                    run_index:first + offset,
                    line,
                    distance,
                    pos: line.centre(distance),
                    riders: 0,
                    paint: Paint::Plain,
                });
            }
        }
        out.push(running);
    }
    out
}

// Bevy batches entities that share a mesh and a material. Reuse them as figures enter and leave.
#[derive(Default)]
struct Batch {
    entities: Vec<Entity>,
    used: usize,
}

#[derive(Default)]
struct Pool(HashMap<(Model, Paint), Batch>);

impl Pool {
    fn place(
        &mut self,
        model: Model,
        paint: Paint,
        transform: Transform,
        models: &Models,
        commands: &mut Commands,
        query: &mut Query<(&mut Transform, &mut Visibility), With<Figure>>,
    ) {
        let batch = self.0.entry((model, paint)).or_default();
        if let Some(&id) = batch.entities.get(batch.used) {
            let (mut t, mut vis) = query.get_mut(id).unwrap();
            t.set_if_neq(transform);
            vis.set_if_neq(Visibility::Inherited);
        } else {
            let id = commands
                .spawn((
                    Figure,
                    Mesh3d(models.mesh[&model].clone()),
                    MeshMaterial3d(models.paint[&(model, paint)].clone()),
                    transform,
                ))
                .id();
            batch.entities.push(id);
        }
        batch.used += 1;
    }
}

#[derive(Component)]
struct Figure;

fn pose(pos: Vec3, dir: Vec3, scale: Vec3) -> Transform {
    Transform::from_translation(pos)
        .with_rotation(Quat::from_rotation_y((-dir.z).atan2(dir.x)))
        .with_scale(scale)
}

fn pedestrian_scale(distance: f32, distant_scale: f32) -> f32 {
    // Keep people at street scale within 500 m. Reach the map scale at the 1 km tier.
    let blend = ((distance - 500.) / 500.).clamp(0., 1.);
    1. + (distant_scale - 1.) * blend
}

fn right(dir: Vec3) -> Vec3 {
    Vec3::new(-dir.z, 0., dir.x)
}

fn departure_offset(phase: f32, copy: usize) -> f32 {
    BIN * (phase + copy as f32 * 0.7548777).fract()
}

fn draw(
    time: Res<Time>,
    replay: Res<Replay>,
    rig: Res<crate::Rig>,
    clock: Res<FixedClock>,
    rate: Res<PlaybackRate>,
    subway: Res<crate::city::SubwayView>,
    routes: Res<Routes>,
    models: Res<Models>,
    camera: Query<(&GlobalTransform, &Frustum), With<Camera3d>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Visibility), With<Figure>>,
    mut pool: Local<Pool>,
    mut marks: ResMut<instanced::Marks>,
) {
    marks.clear();
    if replay.departures.is_empty() {
        return;
    }
    let Ok((camera, frustum)) = camera.single() else {
        return;
    };
    let eye = camera.translation();
    let tier = [1_000., 5_000.].partition_point(|&distance| rig.distance >= distance);
    let visible = |pos: Vec3, radius| {
        frustum.intersects_sphere(
            &Bounds {
                center: pos.into(),
                radius,
            },
            true,
        )
    };
    let size_at = |p: Vec3| (eye.distance(p) * 0.0024).clamp(1.35, 11.);
    // Anchor the accelerated playback to the first result's peak across scenario revisions.
    let now = clock
        .0
        .unwrap_or_else(|| replay.clock_offset.unwrap() + time.elapsed_secs() * rate.0)
        .rem_euclid(DAY);
    let slot = (now / BIN) as i32;
    let mut trains = if replay
        .metro_closure
        .as_ref()
        .is_some_and(|bins| bins.contains(&slot))
    {
        // Rider plans index the train lists by track, including while service is suspended.
        (0..routes.tracks.len()).map(|_| Vec::new()).collect()
    } else {
        trains(&routes, now)
    };
    let mut surface_load = std::collections::BTreeMap::new();
    for batch in pool.0.values_mut() {
        batch.used = 0;
    }
    let mut place = |model, paint, transform| {
        pool.place(model, paint, transform, &models, &mut commands, &mut query)
    };
    let transition = if clock.0.is_some() {
        1.
    } else {
        ((time.elapsed_secs() - replay.revision_started) / 1.2).clamp(0., 1.)
    };
    for bin in slot - (replay.longest / BIN).ceil() as i32..=slot {
        let since_bin = now - bin as f32 * BIN;
        for g in &replay.departures[bin.rem_euclid(48) as usize] {
            let presence = match g.lifecycle {
                Life::Stable => 1.,
                Life::Enter => transition,
                Life::Leave => 1. - transition,
            };
            if presence == 0. {
                continue;
            }
            if since_bin > BIN + g.duration {
                continue;
            }
            let copies = g.copies[tier];
            for j in 0..copies {
                let age = since_bin - departure_offset(g.phase, j);
                let paint = if j >= copies.saturating_sub(g.changed[tier]) {
                    g.tint
                } else {
                    Paint::Plain
                };
                let movement = if let Some(plan) = g.itineraries.get(j) {
                    match plan.at(age) {
                        Some(state @ (RiderState::Walking(p, dir) | RiderState::Waiting(p, dir))) => {
                            if visible(p, 100.) && (subway.0 || p.y >= terrain_from_xz(p.x, p.z) - 0.6) {
                                let scale = pedestrian_scale(eye.distance(p), 1.2) * presence;
                                let model = if matches!(state, RiderState::Walking(..)) {
                                    plan.walking_model(age, scale)
                                } else {
                                    Model::Person
                                };
                                place(model, paint, pose(p, dir, Vec3::splat(scale)));
                            }
                            None
                        }
                        Some(RiderState::Riding(RideKey::Metro { track, run })) => {
                            if let Some(t) = trains[track].iter_mut().find(|t| t.run_index == run) {
                                t.riders += 1;
                                if paint != Paint::Plain {
                                    t.paint = paint;
                                }
                            }
                            None
                        }
                        Some(RiderState::Riding(RideKey::Surface { template, start })) => {
                            let key = (template, start + bin.div_euclid(48) * DAY as i32);
                            let (applied, count) = surface_load.entry(key).or_insert((Paint::Plain, 0));
                            if !matches!(g.lifecycle, Life::Leave) { *count += 1; }
                            if paint != Paint::Plain {
                                *applied = paint;
                            }
                            None
                        }
                        None => None,
                    }
                } else {
                    g.path.trip_pose(age, j)
                };
                let Some((pos, dir, kind, metres)) = movement else {
                    continue;
                };
                if !visible(pos, 100.) || (kind == Vehicle::Car && replay.road_edits.hides(pos, dir)) {
                    continue;
                }
                let model = match kind {
                    Vehicle::Car => Model::Car(g.cars[j]),
                    Vehicle::Bus => Model::Bus,
                    Vehicle::Bicycle => Model::Bike,
                    Vehicle::Walk => Model::Person,
                    Vehicle::Rail => Model::Coach,
                };
                // In the overview keep moving figures legible when each pixel spans many metres.
                let scale = if tier == 2 {
                    size_at(pos).max(eye.distance(pos) * 0.0012)
                } else if kind == Vehicle::Walk {
                    pedestrian_scale(eye.distance(pos), size_at(pos))
                } else {
                    size_at(pos)
                } * presence;
                let lane = if kind == Vehicle::Walk { 3.4 } else { 1.6 };
                let mut position = pos + right(dir) * lane + Vec3::Y * 0.25;
                if kind == Vehicle::Car {
                    let Some(surface) = routes.water.surface_position(&g.path, metres, position, dir) else { continue; };
                    position = surface;
                }
                let transform = pose(
                    position,
                    dir,
                    Vec3::splat(scale),
                );
                if tier == 2 && matches!(kind, Vehicle::Car | Vehicle::Bicycle | Vehicle::Walk) {
                    let (rgb, radius) = match model {
                        Model::Car(style) => (if models.ink { CAR_INK } else { CAR_SWATCHES[style as usize % CAR_SWATCHES.len()] }, 2.5),
                        Model::Bike => ([55, 70, 76], 1.4),
                        Model::Person => (if models.ink { PERSON_INK } else { [36, 134, 171] }, 1.25),
                        _ => unreachable!(),
                    };
                    let tint = match paint {
                        Paint::Plain => None,
                        Paint::Gain => Some([103, 255, 169]),
                        Paint::Loss => Some([255, 136, 144]),
                        Paint::Rerouted => Some([144, 217, 255]),
                    };
                    // Lift the billboard toward the eye along the lane's own projection ray.
                    // This preserves the overview's depth clearance without shifting a bank
                    // car sideways over the river on screen.
                    let lane = transform.translation;
                    let center = if kind == Vehicle::Car {
                        let to_eye = eye - lane;
                        lane + to_eye * (scale / to_eye.y.max(1.))
                    } else {
                        lane + Vec3::Y * scale
                    };
                    if let Some([r, g, b]) = tint {
                        let accent = if models.ink {
                            Color::linear_rgb(0.85, 0.13, 0.09)
                        } else {
                            Color::srgb_u8(r, g, b)
                        };
                        marks.push(center, (radius + 1.) * presence, accent.to_linear());
                    }
                    let [r, g, b] = rgb;
                    marks.push(center, radius * presence, Color::srgb_u8(r, g, b).to_linear());
                    continue;
                }
                if kind == Vehicle::Car {
                    place(model, Paint::Plain, transform);
                    if paint != Paint::Plain {
                        place(Model::CarChange, paint, transform);
                    }
                } else {
                    let model = if kind == Vehicle::Walk {
                        Model::Walking(motion::gait_frame(metres, scale, j))
                    } else {
                        model
                    };
                    place(model, paint, transform);
                }
            }
        }
    }
    for ((template, start), (paint, count)) in surface_load {
        if let Some((p, dir, kind)) = routes.riders.surface_pose(template, start, now) {
            if visible(p, 100.) {
                let model = if kind == Vehicle::Rail {
                    Model::Coach
                } else {
                    Model::Bus
                };
                let scale = size_at(p);
                let front = p + right(dir) * 1.6 + Vec3::Y * 0.25;
                place(model, paint, pose(front, dir, Vec3::splat(scale)));
                if kind == Vehicle::Bus && count > 0 {
                    let base = replay.base_bus_loads.as_ref().map_or(count, |loads| loads.count(template, start, now));
                    bus_load::draw(count, base, front, dir, scale, &mut place);
                }
                if let Some((rear, rear_dir)) = routes.riders.bus_rear(template, start, now, p, dir) {
                    let hinge = front - dir * BUS_FRONT_TO_HINGE * scale;
                    place(Model::BusTail, paint, pose(front + (rear - p) * scale, rear_dir, Vec3::splat(scale)));
                    place(Model::BusJoint, paint, pose(hinge, dir, Vec3::splat(scale)));
                }
            }
        }
    }
    for train in trains.into_iter().flatten() {
        if !visible(train.pos, 125.) {
            continue;
        }
        // Keep each three-car set centred at the station, with room for the opposite track.
        let scale = 1.35;
        for offset in [14.3, 0., -14.3] {
            let distance = train.distance + offset * scale;
            if !subway.0 && train.line.levels.at(distance) < -0.6 {
                continue;
            }
            let (p, dir) = train.line.coach(distance);
            place(Model::Coach, train.paint, pose(p, dir, Vec3::splat(scale)));
            // The illuminated roof strip shows the workers riding this service in this frame.
            if train.riders > 0 {
                let length = 10. * train.riders as f32 / (train.riders as f32 + 4.);
                place(
                    Model::Window,
                    train.paint,
                    pose(
                        p + Vec3::Y * (27. * ART_PIXEL + 0.02) * scale,
                        dir,
                        Vec3::new(length, 1., 1.) * scale,
                    ),
                );
            }
        }
    }
    for station in &replay.platforms {
        station.draw(now, subway.0, eye, rig.distance, &visible, &mut place);
    }
    for batch in pool.0.values() {
        for &id in &batch.entities[batch.used..] {
            let (_, mut vis) = query.get_mut(id).unwrap();
            vis.set_if_neq(Visibility::Hidden);
        }
    }
}
