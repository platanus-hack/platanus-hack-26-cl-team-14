//! Replay of an engine result. Positions between departure bins are display interpolation.
use crate::{city::world, net::Scenario};
use bevy::{
    camera::primitives::{Frustum, Sphere as Bounds},
    prelude::*,
};
use serde_json::Value;
use std::{collections::HashMap, fs, sync::Arc};

// One moving figure represents 96 workers. The engine's counts stay in persons.
const PERSONS_PER_MOVER: f64 = 96.0;
const BIN: f32 = 1800.0;
const DAY: f32 = 48.0 * BIN;

pub struct MoversPlugin;
impl Plugin for MoversPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .init_resource::<Replay>()
            .add_systems(Update, (read_result, draw).chain());
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
    Metro { track: usize, board: f32, exit: f32 },
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
            Self::Metro { .. } | Self::Rail => 11.,
        }
    }
}

struct Leg {
    end: usize,
    vehicle: Vehicle,
}

struct Path {
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
            points,
            metres,
            seconds,
            legs,
        })
    }

    fn single(points: Vec<Vec3>, vehicle: Vehicle) -> Arc<Self> {
        let end = points.len() - 1;
        Self::new(points, vec![Leg { end, vehicle }])
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
        if value < 0. || value >= *axis.last()? {
            return None;
        }
        let end = axis.partition_point(|&v| v <= value);
        let from = self.points[end - 1];
        let to = self.points[end];
        let t = (value - axis[end - 1]) / (axis[end] - axis[end - 1]);
        let leg = self.legs.partition_point(|leg| leg.end < end);
        Some((from.lerp(to, t), (to - from).normalize(), self.legs[leg].vehicle))
    }
}

struct RailLine {
    path: Arc<Path>,
    service: String,
    departures: Vec<usize>,
    stations: Vec<String>,
}

type Table = HashMap<String, Arc<Path>>;

#[derive(Resource)]
struct Routes {
    cars: Table,
    bikes: Table,
    public: Table,
    tracks: Vec<RailLine>,
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

impl Routes {
    fn load() -> Self {
        let mut tracks = Vec::new();
        for row in json("web/data/transit_paths.json").as_array().unwrap() {
            if row["mode"] != "metro" {
                continue;
            }
            tracks.push(RailLine {
                service: row["route"].as_str().unwrap().to_owned(),
                path: Path::single(points(&row["path"]), Vehicle::Rail),
                departures: row["trips"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as usize)
                    .collect(),
                stations: row["parent_station_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect(),
            });
        }
        let public = json("web/data/public_routes.json")
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, row)| {
                let path = points(&row["path"]);
                let legs = row["segments"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|leg| Leg {
                        end: leg["end"].as_u64().unwrap() as usize,
                        vehicle: match leg["mode"].as_str().unwrap() {
                            "walk" => Vehicle::Walk,
                            "bus" => Vehicle::Bus,
                            "metro" => {
                                let first = path[leg["start"].as_u64().unwrap() as usize];
                                let last = path[leg["end"].as_u64().unwrap() as usize];
                                tracks
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, t)| t.service == leg["service"].as_str().unwrap())
                                    .find_map(|(track, t)| {
                                        let board = t.path.nearest(first).2;
                                        let exit = t.path.nearest(last).2;
                                        (board < exit).then_some(Vehicle::Metro { track, board, exit })
                                    })
                                    .unwrap()
                            }
                            "rail" => Vehicle::Rail,
                            mode => panic!("unknown public route mode {mode}"),
                        },
                    })
                    .collect();
                (key.clone(), Path::new(path, legs))
            })
            .collect();
        Self {
            public,
            tracks,
            cars: table("web/data/car_routes.json", Vehicle::Car),
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
        let table = match mode {
            "car" => Some(&self.cars),
            "bicycle" => Some(&self.bikes),
            "public" => return self.public.get(&pair).cloned(),
            "walk" => None,
            _ => return None,
        };
        if let Some(path) = table.and_then(|t| t.get(&pair)) {
            return Some(path.clone());
        }
        let origin = point(&row["origin"]);
        let destination = point(&row["destination"]);
        // Missing OD routes show only a mapped local segment at the origin.
        match mode {
            "walk" => self.streets.local_path(origin, destination, Vehicle::Walk),
            "bicycle" => self.cycleways.local_path(origin, destination, Vehicle::Bicycle),
            _ => self.streets.local_path(origin, destination, Vehicle::Car),
        }
    }
}

// Index actual street segments. Short local walks follow the street nearest the sampled origin.
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
    path: Arc<Path>,
    copies: usize,
    phase: f32,
    changed: usize,
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
    departures: Vec<Vec<Group>>,
    platforms: Vec<Platform>,
    longest: f32,
    revision_started: f32,
}

struct Platform {
    pos: Vec3,
    dir: Vec3,
    daily: f64,
    delta: Vec<f32>,
}

fn read_result(time: Res<Time>, result: Res<Scenario>, routes: Res<Routes>, mut replay: ResMut<Replay>) {
    if !result.is_changed() || result.0.is_null() {
        return;
    }
    let r = &result.0;
    let mut deltas = HashMap::new();
    for d in r["trips_delta"].as_array().into_iter().flatten() {
        deltas.insert(
            d[0].as_u64().unwrap(),
            (d[1].as_f64().unwrap(), d[2].as_u64().unwrap()),
        );
    }
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
        let copies = (row["count"].as_f64().unwrap() / PERSONS_PER_MOVER + seed).floor() as usize;
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
            (delta.abs() / PERSONS_PER_MOVER).ceil() as usize
        };
        replay.longest = replay.longest.max(*path.seconds.last().unwrap());
        replay.departures[row["half_hour"].as_u64().unwrap() as usize].push(Group {
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
    replay.platforms = r["stations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            let pos = world(s["longitude"].as_f64().unwrap(), s["latitude"].as_f64().unwrap());
            let id = s["id"].as_str().unwrap();
            let (pos, dir, _) = routes
                .tracks
                .iter()
                .filter(|line| line.stations.iter().any(|v| v == id))
                .map(|line| line.path.nearest(pos))
                .min_by(|(a, _, _), (b, _, _)| a.distance_squared(pos).total_cmp(&b.distance_squared(pos)))
                .unwrap_or((pos, Vec3::X, 0.));
            Platform {
                pos,
                dir,
                daily: s["scenario_daily"].as_f64().unwrap(),
                delta: s["delta"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|v| v.as_f64().unwrap() as f32)
                    .collect(),
            }
        })
        .collect();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Model {
    Car,
    Bus,
    Bike,
    Person,
    Coach,
    Window,
    Platform,
}

#[derive(Resource)]
struct Models {
    mesh: HashMap<Model, Handle<Mesh>>,
    paint: HashMap<Paint, Handle<StandardMaterial>>,
}

type Rgb = [u8; 3];
const GLASS: Rgb = [36, 55, 70];
const IVORY: Rgb = [249, 237, 210];

fn painted(mut mesh: Mesh, offset: [f32; 3], rgb: Rgb) -> Mesh {
    let color = Color::srgb_u8(rgb[0], rgb[1], rgb[2]).to_linear().to_f32_array();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![color; mesh.count_vertices()]);
    mesh.translated_by(Vec3::from_array(offset))
}

fn block(size: [f32; 3], at: [f32; 3], rgb: Rgb) -> Mesh {
    painted(Mesh::from(Cuboid::new(size[0], size[1], size[2])), at, rgb)
}

fn head(at: [f32; 3]) -> Mesh {
    painted(Sphere::new(0.31).mesh().ico(1).unwrap(), at, [248, 189, 137])
}

fn coat(at: [f32; 3], rgb: Rgb) -> Mesh {
    painted(
        Capsule3d::new(0.38, 0.5)
            .mesh()
            .longitudes(8)
            .latitudes(4)
            .build(),
        at,
        rgb,
    )
}

fn wheel(x: f32) -> Mesh {
    let disc = Cylinder::new(0.43, 0.13)
        .mesh()
        .resolution(10)
        .build()
        .rotated_by(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    painted(disc, [x, 0.44, 0.], GLASS)
}

fn shape(parts: impl IntoIterator<Item = Mesh>) -> Mesh {
    let mut parts = parts.into_iter();
    let mut shape = parts.next().unwrap();
    for mesh in parts {
        shape.merge(&mesh).unwrap();
    }
    shape
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let car = shape([
        block([4.7, 0.38, 1.85], [0., 0.36, 0.], GLASS),
        block([4.9, 0.78, 2.05], [0., 0.86, 0.], [241, 169, 58]),
        block([2.55, 0.62, 1.69], [-0.12, 1.48, 0.], GLASS),
        block([1.95, 0.15, 1.66], [-0.25, 1.85, 0.], IVORY),
    ]);
    let bus = shape([
        block([11.6, 1.55, 2.65], [0., 1.05, 0.], [205, 53, 49]),
        block([11.35, 1.12, 2.63], [0., 2.36, 0.], GLASS),
        block([11.4, 0.28, 2.72], [0., 3.05, 0.], IVORY),
        block([2.8, 0.25, 1.55], [-1.8, 3.3, 0.], [151, 164, 171]),
    ]);
    let bike = shape([
        wheel(-0.7),
        wheel(0.7),
        block([1.35, 0.22, 0.22], [0., 0.65, 0.], [69, 158, 116]),
        coat([0., 1.24, 0.], [41, 133, 90]),
        head([0.14, 2.02, 0.]),
    ]);
    let person = shape([
        block([0.44, 0.58, 0.51], [0., 0.3, 0.], GLASS),
        coat([0., 1.12, 0.], [36, 134, 171]),
        head([0., 1.97, 0.]),
    ]);
    let coach = shape([
        block([13.3, 0.72, 2.92], [0., 0.48, 0.], [187, 46, 52]),
        block([13.5, 1.52, 2.96], [0., 1.62, 0.], IVORY),
        block([12.93, 0.97, 3.01], [0., 2.1, 0.], GLASS),
        block([13.5, 0.47, 3.04], [0., 2.83, 0.], [220, 220, 211]),
    ]);
    let mesh = [
        (Model::Car, car),
        (Model::Bus, bus),
        (Model::Bike, bike),
        (Model::Person, person),
        (Model::Coach, coach),
        (
            Model::Window,
            block([1., 0.18, 1.35], [0., 0., 0.], [251, 183, 67]),
        ),
        (
            Model::Platform,
            block([1., 0.55, 1.], [0., 0.28, 0.], [132, 137, 142]),
        ),
    ]
    .into_iter()
    .map(|(id, mesh)| (id, meshes.add(mesh)))
    .collect();
    let paint = [
        (Paint::Plain, [255, 255, 255]),
        (Paint::Gain, [103, 255, 169]),
        (Paint::Loss, [255, 136, 144]),
        (Paint::Rerouted, [144, 217, 255]),
    ]
    .into_iter()
    .map(|(key, c)| {
        (
            key,
            mats.add(StandardMaterial {
                base_color: Color::srgb_u8(c[0], c[1], c[2]),
                perceptual_roughness: 0.8,
                ..default()
            }),
        )
    })
    .collect();
    commands.insert_resource(Models { mesh, paint });
    commands.insert_resource(Routes::load());
}

struct Train<'a> {
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
        let duration = *line.path.seconds.last().unwrap();
        let slot = (now / BIN).floor() as i32;
        for bin in slot - (duration / BIN).ceil() as i32..=slot {
            let count = line.departures[bin.rem_euclid(48) as usize];
            for i in 0..count {
                let start = (bin as f32 + i as f32 / count as f32) * BIN;
                let distance = (now - start) * Vehicle::Rail.speed();
                if let Some((pos, _, _)) = line.path.at(distance, &line.path.metres) {
                    running.push(Train {
                        line,
                        distance,
                        pos,
                        riders: 0,
                        paint: Paint::Plain,
                    });
                }
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
                    MeshMaterial3d(models.paint[&paint].clone()),
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

fn right(dir: Vec3) -> Vec3 {
    Vec3::new(-dir.z, 0., dir.x)
}

fn draw(
    time: Res<Time>,
    replay: Res<Replay>,
    routes: Res<Routes>,
    models: Res<Models>,
    camera: Query<(&GlobalTransform, &Frustum), With<Camera3d>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Visibility), With<Figure>>,
    mut pool: Local<Pool>,
) {
    if replay.departures.is_empty() {
        return;
    }
    let Ok((camera, frustum)) = camera.single() else {
        return;
    };
    let eye = camera.translation();
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
    // Twelve city seconds per real second, starting in the morning peak.
    let now = (8. * 3600. + 10. * 60. + time.elapsed_secs() * 12.) % DAY;
    let slot = (now / BIN) as i32;
    let mut trains = trains(&routes, now);
    for batch in pool.0.values_mut() {
        batch.used = 0;
    }
    let mut place =
        |model, paint, transform| pool.place(model, paint, transform, &models, &mut commands, &mut query);
    let transition = ((time.elapsed_secs() - replay.revision_started) / 1.2).clamp(0., 1.);
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
            let duration = *g.path.seconds.last().unwrap();
            if since_bin > BIN + duration {
                continue;
            }
            for j in 0..g.copies {
                let age = since_bin - BIN * (g.phase + j as f32 * 0.7548777).fract();
                let Some((pos, dir, kind)) = g.path.at(age, &g.path.seconds) else {
                    continue;
                };
                let paint = if j >= g.copies.saturating_sub(g.changed) {
                    g.tint
                } else {
                    Paint::Plain
                };
                if let Vehicle::Metro { track, board, exit } = kind {
                    if !visible(pos, 3500.) {
                        continue;
                    }
                    // Riders share the closest scheduled train on their line in their travel direction.
                    if let Some(train) = trains[track]
                        .iter_mut()
                        .filter(|t| {
                            t.distance >= board
                                && t.distance <= exit
                                && t.pos.distance_squared(pos) <= 3000_f32.powi(2)
                        })
                        .min_by(|a, b| {
                            a.pos
                                .distance_squared(pos)
                                .total_cmp(&b.pos.distance_squared(pos))
                        })
                    {
                        train.riders += 1;
                        if paint != Paint::Plain {
                            train.paint = paint;
                        }
                    }
                    continue;
                }
                if !visible(pos, 100.) {
                    continue;
                }
                let model = match kind {
                    Vehicle::Car => Model::Car,
                    Vehicle::Bus => Model::Bus,
                    Vehicle::Bicycle => Model::Bike,
                    Vehicle::Walk => Model::Person,
                    Vehicle::Rail => Model::Coach,
                    Vehicle::Metro { .. } => unreachable!(),
                };
                let scale = size_at(pos) * presence;
                let lane = if kind == Vehicle::Walk { 3.4 } else { 1.6 };
                place(
                    model,
                    paint,
                    pose(pos + right(dir) * lane + Vec3::Y * 0.25, dir, Vec3::splat(scale)),
                );
            }
        }
    }
    for train in trains.into_iter().flatten() {
        if !visible(train.pos, 125.) {
            continue;
        }
        let scale = size_at(train.pos).min(2.5);
        for n in 0..3 {
            let d = train.distance - n as f32 * 15.2 * scale;
            let Some((p, dir, _)) = train.line.path.at(d, &train.line.path.metres) else {
                continue;
            };
            let p = p + right(dir) * 2.5 + Vec3::Y * 0.45;
            place(Model::Coach, train.paint, pose(p, dir, Vec3::splat(scale)));
            // The illuminated roof strip shows the workers riding this service in this frame.
            if train.riders > 0 {
                let length = 10. * train.riders as f32 / (train.riders as f32 + 4.);
                place(
                    Model::Window,
                    train.paint,
                    pose(p + Vec3::Y * 3.18 * scale, dir, Vec3::new(length, 1., 1.) * scale),
                );
            }
        }
    }
    for p in &replay.platforms {
        if !visible(p.pos, 70.) {
            continue;
        }
        let scale = size_at(p.pos).min(2.);
        let side = right(p.dir);
        let center = p.pos + side * 9. + Vec3::Y * 0.25;
        place(
            Model::Platform,
            Paint::Plain,
            pose(center, p.dir, Vec3::new(52., 1., 6.)),
        );
        // Platform figures represent daily boardings; the engine publishes only changes by half hour.
        let count = (p.daily / 1600.).round() as usize;
        let delta = p.delta.get(slot as usize).copied().unwrap_or_default();
        let accent = if delta > 0. {
            Paint::Gain
        } else if delta < 0. {
            Paint::Loss
        } else {
            Paint::Plain
        };
        for i in 0..count {
            let along = (i % 20) as f32 * 2.04 - 19.4;
            let across = (i / 20) as f32 * 1.15 - 1.8;
            let at = center + p.dir * along + side * across + Vec3::Y * 0.58;
            place(Model::Person, accent, pose(at, -side, Vec3::splat(scale)));
        }
    }
    for batch in pool.0.values() {
        for &id in &batch.entities[batch.used..] {
            let (_, mut vis) = query.get_mut(id).unwrap();
            vis.set_if_neq(Visibility::Hidden);
        }
    }
}
