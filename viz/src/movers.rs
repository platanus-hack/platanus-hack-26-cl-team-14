//! Replay of an engine result. Positions between departure bins are display interpolation.
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
        let mut pos = from.lerp(to, t);
        pos.y = terrain_from_xz(pos.x, pos.z);
        Some((pos, (to - from).normalize(), self.legs[leg].vehicle))
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
    local: HashMap<LocalKey, Arc<Path>>,
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

fn public_path(row: &Value, tracks: &[RailLine]) -> Arc<Path> {
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

fn local_routes(tracks: &[RailLine]) -> HashMap<LocalKey, Arc<Path>> {
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
        let mut line = points(&active["paths"][g["route"].as_u64().unwrap() as usize]);
        if kind == Vehicle::Walk {
            // The close view shows the first block of the real walking route.
            let mut remaining = 180.;
            for i in 1..line.len() {
                let length = line[i - 1].distance(line[i]);
                if length > remaining {
                    line[i] = line[i - 1].lerp(line[i], remaining / length);
                    line.truncate(i + 1);
                    break;
                }
                remaining -= length;
            }
        }
        let key = local_key(g["id"].as_u64().unwrap(), mode, &g["origin"], &g["destination"]);
        local.insert(key, Path::single(line, kind));
    }
    let gtfs = gzip_json("web/data/local_public_routes.json.gz");
    let records = gtfs["paths"].as_array().unwrap();
    let paths: Vec<_> = records.iter().map(|r| public_path(r, tracks)).collect();
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
            .map(|(key, row)| (key.clone(), public_path(row, &tracks)))
            .collect();
        let local = local_routes(&tracks);
        Self {
            public,
            local,
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
        let key = local_key(row["id"].as_u64()?, mode, &row["origin"], &row["destination"]);
        if let Some(path) = self.local.get(&key) {
            return Some(path.clone());
        }
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
    half_hour: Vec<f64>,
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
                half_hour: s["scenario_half_hour"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|v| v.as_f64().unwrap())
                    .collect(),
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
    paint: HashMap<(Model, Paint), Handle<StandardMaterial>>,
}

type Rgb = [u8; 3];
const GLASS: Rgb = [36, 55, 70];

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

fn sprite_stack(bytes: &[u8], slices: u32) -> (Mesh, Image) {
    let image = Image::from_buffer(
        bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::nearest(),
        RenderAssetUsages::default(),
    )
    .unwrap();
    let x = (image.width() / slices) as f32 * ART_PIXEL / 2.;
    let z = image.height() as f32 * ART_PIXEL / 2.;
    let mut positions = Vec::new();
    let mut uv = Vec::new();
    let mut indices = Vec::new();
    for slice in 0..slices {
        let y = slice as f32 * ART_PIXEL;
        let u_left = slice as f32 / slices as f32;
        let u_right = (slice + 1) as f32 / slices as f32;
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
    let person = shape([
        block([0.44, 0.58, 0.51], [0., 0.3, 0.], GLASS),
        coat([0., 1.12, 0.], [36, 134, 171]),
        head([0., 1.97, 0.]),
    ]);
    let sprites: [(Model, &[u8], u32); 4] = [
        (Model::Car, include_bytes!("../assets/car.png"), 15),
        (Model::Bus, include_bytes!("../assets/bus.png"), 27),
        (Model::Coach, include_bytes!("../assets/metro.png"), 28),
        (Model::Bike, include_bytes!("../assets/bicycle.png"), 16),
    ];
    let models = [
        (Model::Person, person, None),
        (
            Model::Window,
            painted(
                Plane3d::default().mesh().size(1., 1.35).build(),
                [0.; 3],
                [251, 183, 67],
            ),
            None,
        ),
        (
            Model::Platform,
            block([1., 0.55, 1.], [0., 0.28, 0.], [132, 137, 142]),
            None,
        ),
    ]
    .into_iter()
    .chain(sprites.into_iter().map(|(model, bytes, layers)| {
        let (mesh, image) = sprite_stack(bytes, layers);
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
            paint.insert(
                (model, tint),
                mats.add(StandardMaterial {
                    base_color: Color::srgb_u8(r, g, b),
                    base_color_texture: texture.clone(),
                    unlit: sprite,
                    alpha_mode: if sprite {
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
    commands.insert_resource(Models { mesh, paint });
    let routes = Routes::load();
    let track_material = mats.add(StandardMaterial {
        perceptual_roughness: 0.85,
        ..default()
    });
    for line in &routes.tracks {
        commands.spawn((
            Mesh3d(meshes.add(track_mesh(&line.path))),
            MeshMaterial3d(track_material.clone()),
        ));
    }
    commands.insert_resource(routes);
}

// A surface cutaway exposes the published Metro alignment, including underground sections.
fn track_mesh(path: &Path) -> Mesh {
    let mut surface = Surface::default();
    let bed = Color::srgb_u8(93, 91, 84).to_linear().to_f32_array();
    let steel = Color::srgb_u8(159, 162, 155).to_linear().to_f32_array();
    for edge in path.points.windows(2) {
        let delta = edge[1] - edge[0];
        let side = right(delta).normalize_or_zero();
        let steps = (delta.length() / 50.).ceil() as usize;
        for i in 0..steps {
            let a = edge[0] + delta * (i as f32 / steps as f32);
            let b = edge[0] + delta * ((i + 1) as f32 / steps as f32);
            for (left, right, lift, color) in [
                (1.0, 4.0, 0.32, bed),
                (1.59, 1.8, 0.38, steel),
                (3.2, 3.41, 0.38, steel),
            ] {
                let start = surface.positions.len() as u32;
                for mut p in [
                    a + side * left,
                    a + side * right,
                    b + side * right,
                    b + side * left,
                ] {
                    p.y = terrain_from_xz(p.x, p.z) + lift;
                    surface.positions.push(p.to_array());
                    surface.normals.push([0., 1., 0.]);
                    surface.colors.push(color);
                }
                surface
                    .indices
                    .extend([start, start + 1, start + 2, start, start + 2, start + 3]);
            }
        }
    }
    surface.finish()
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

fn right(dir: Vec3) -> Vec3 {
    Vec3::new(-dir.z, 0., dir.x)
}

fn draw(
    time: Res<Time>,
    replay: Res<Replay>,
    clock: Res<FixedClock>,
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
    let now = clock.0
        .unwrap_or(8. * 3600. + 10. * 60. + time.elapsed_secs() * 12.)
        .rem_euclid(DAY);
    let slot = (now / BIN) as i32;
    let mut trains = trains(&routes, now);
    for batch in pool.0.values_mut() {
        batch.used = 0;
    }
    let mut place =
        |model, paint, transform| pool.place(model, paint, transform, &models, &mut commands, &mut query);
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
            let d = train.distance - n as f32 * 14.3 * scale;
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
                    pose(p + Vec3::Y * (27. * ART_PIXEL + 0.02) * scale, dir, Vec3::new(length, 1., 1.) * scale),
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
        // With a half-hour profile each figure uses the same scale as a moving group.
        // Results that publish daily totals show one figure per 1,600 daily boardings.
        let count = p
            .half_hour
            .get(slot as usize)
            .map_or(p.daily / 1600., |n| n / PERSONS_PER_MOVER)
            .round() as usize;
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
