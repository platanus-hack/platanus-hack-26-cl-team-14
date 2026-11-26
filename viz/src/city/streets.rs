use super::{
    Rgba, Surface, ground::ribbon_faces, street_mask::StreetMask, terrain::drape_land,
    terrain_from_xz, world_xz,
};
use bevy::{
    camera::{
        primitives::{Aabb, MeshAabb},
        visibility::NoAutoAabb,
    },
    prelude::*,
};
use flate2::read::GzDecoder;
use geo::Rect as GeoRect;
#[cfg(not(target_family = "wasm"))]
use serde::Deserialize;
use std::collections::BTreeMap;
#[cfg(not(target_family = "wasm"))]
use std::io::BufReader;
#[cfg(target_family = "wasm")]
use std::io::Read;

#[cfg(not(target_family = "wasm"))]
#[derive(Deserialize)]
struct Roads {
    features: Vec<Road>,
}

#[cfg(not(target_family = "wasm"))]
#[derive(Deserialize)]
struct Road {
    geometry: Polygon,
    properties: RoadProperties,
}

#[cfg(not(target_family = "wasm"))]
#[derive(Deserialize)]
struct Polygon {
    coordinates: Vec<Vec<[f64; 2]>>,
}

#[cfg(not(target_family = "wasm"))]
#[derive(Deserialize)]
struct RoadProperties {
    tile: usize,
    structure: String,
    layer: i32,
    #[serde(rename = "use")]
    road_use: String,
}

const ASPHALT: Rgba = [0.18, 0.19, 0.20, 1.];

struct StreetFace {
    rings: Vec<Vec<Vec2>>,
    bounds: GeoRect<f32>,
    lift: f32,
    color: Rgba,
}

impl StreetFace {
    fn road(rings: Vec<Vec<Vec2>>, layer: i32, road_use: &str) -> Self {
        let (color, extra_lift) = match road_use {
            "sidewalk" => ([0.66, 0.67, 0.68, 1.], 0.03),
            "foot" => ([0.47, 0.48, 0.49, 1.], 0.02),
            "cycle" => ([0.30, 0.31, 0.32, 1.], 0.01),
            _ => (ASPHALT, 0.),
        };
        Self::new(rings, super::road_surface_lift(layer) + extra_lift, color)
    }

    fn new(rings: Vec<Vec<Vec2>>, lift: f32, color: Rgba) -> Self {
        let (mut low, mut high) = (rings[0][0], rings[0][0]);
        for &p in &rings[0] {
            low = low.min(p);
            high = high.max(p);
        }
        Self {
            rings,
            bounds: GeoRect::new((low.x, low.y), (high.x, high.y)),
            lift,
            color,
        }
    }

    fn draw(&self, surface: &mut Surface, mask: Option<&StreetMask>) {
        let rings: Vec<_> = self
            .rings
            .iter()
            .map(|ring| {
                ring.iter()
                    .map(|p| Vec3::new(p.x, terrain_from_xz(p.x, p.y), p.y))
                    .collect()
            })
            .collect();
        if let Some(mask) = mask.filter(|mask| mask.intersects(&self.bounds)) {
            mask.drape(surface, &rings, self.lift, self.color);
        } else {
            drape_land(surface, &rings, self.lift, self.color);
        }
    }
}

#[derive(Component)]
pub(crate) struct StreetTile {
    faces: Vec<StreetFace>,
    bounds: GeoRect<f32>,
    original: Handle<Mesh>,
}

fn mesh_for(faces: &[StreetFace], mask: Option<&StreetMask>) -> Mesh {
    let mut surface = Surface::default();
    for face in faces {
        face.draw(&mut surface, mask);
    }
    surface.finish()
}

impl StreetTile {
    fn masked(&self, mask: &StreetMask) -> Option<Mesh> {
        (mask.intersects(&self.bounds) && self.faces.iter().any(|f| mask.intersects(&f.bounds)))
            .then(|| mesh_for(&self.faces, Some(mask)))
    }

    fn bundle(
        faces: Vec<StreetFace>,
        meshes: &mut Assets<Mesh>,
    ) -> (Self, Mesh3d, Aabb, NoAutoAabb) {
        let mut bounds = faces[0].bounds;
        for face in &faces[1..] {
            let (lo, hi) = (face.bounds.min(), face.bounds.max());
            bounds.set_min((lo.x.min(bounds.min().x), lo.y.min(bounds.min().y)));
            bounds.set_max((hi.x.max(bounds.max().x), hi.y.max(bounds.max().y)));
        }
        let mesh = mesh_for(&faces, None);
        // Clipping can only shrink the tile. Keep its full bounds even after
        // the original vertices have been uploaded and dropped from CPU memory.
        let aabb = mesh.get_aabb().unwrap();
        let original = meshes.add(mesh);
        let mesh = Mesh3d(original.clone());
        (
            Self {
                faces,
                bounds,
                original,
            },
            mesh,
            aabb,
            NoAutoAabb,
        )
    }
}

// Rebuild from the packed baseline after a placement changes. A reset restores
// the original GPU handle and releases the temporary mesh.
pub(super) fn apply(
    mask: &StreetMask,
    meshes: &mut Assets<Mesh>,
    tiles: &mut Query<(&StreetTile, &mut Mesh3d)>,
) {
    for (tile, mut current) in tiles {
        if let Some(mesh) = tile.masked(mask) {
            current.0 = meshes.add(mesh);
        } else if current.0 != tile.original {
            current.0 = tile.original.clone();
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn load_faces() -> BTreeMap<usize, Vec<StreetFace>> {
    let file = crate::data::open("web/data/road_surface.geojson.gz").unwrap();
    let gz = GzDecoder::new(file);
    let roads: Roads = serde_json::from_reader(BufReader::new(gz)).unwrap();
    let mut tiles = BTreeMap::<usize, Vec<StreetFace>>::new();
    for road in roads.features {
        let p = road.properties;
        if p.structure == "tunnel" {
            continue;
        }
        let rings = road
            .geometry
            .coordinates
            .into_iter()
            .map(|mut ring| {
                ring.pop(); // GeoJSON repeats the first vertex at the end.
                ring.into_iter()
                    .map(|[lon, lat]| world_xz(lon, lat))
                    .collect()
            })
            .collect();
        tiles
            .entry(p.tile)
            .or_default()
            .push(StreetFace::road(rings, p.layer, &p.road_use));
    }
    tiles
}

#[cfg(target_family = "wasm")]
fn load_faces() -> BTreeMap<usize, Vec<StreetFace>> {
    let file = crate::data::open("web/wasm/road-surface.bin.gz").unwrap();
    decode_faces(file)
}

#[cfg(target_family = "wasm")]
fn decode_faces(file: impl Read) -> BTreeMap<usize, Vec<StreetFace>> {
    fn varint(bytes: &mut &[u8]) -> u32 {
        let mut value = 0;
        let mut shift = 0;
        loop {
            let (&byte, rest) = bytes.split_first().unwrap();
            *bytes = rest;
            value |= u32::from(byte & 127) << shift;
            if byte < 128 {
                return value;
            }
            shift += 7;
        }
    }

    fn signed(value: u32) -> i32 {
        (value >> 1) as i32 ^ -((value & 1) as i32)
    }

    fn coordinate(bytes: &mut &[u8], previous: &mut i32) -> f64 {
        let mut delta = varint(bytes);
        // A zero marks an exact correction for an off-grid coordinate.
        let correction = if delta == 0 {
            let correction = signed(varint(bytes));
            delta = varint(bytes);
            correction
        } else {
            0
        };
        *previous += signed(delta - 1);
        let rounded = f64::from(*previous) / 1_000_000.;
        f64::from_bits(rounded.to_bits().wrapping_add_signed(i64::from(correction)))
    }

    let mut unpacked = Vec::new();
    GzDecoder::new(file).read_to_end(&mut unpacked).unwrap();
    let mut bytes = unpacked.as_slice();
    let mut tiles = BTreeMap::<usize, Vec<StreetFace>>::new();
    while !bytes.is_empty() {
        let tile = varint(&mut bytes) as usize;
        let layer = signed(varint(&mut bytes));
        let road_use = ["road", "cycle", "foot", "sidewalk"][varint(&mut bytes) as usize];
        let rings = (0..varint(&mut bytes))
            .map(|_| {
                let (mut lon, mut lat) = (0, 0);
                (0..varint(&mut bytes))
                    .map(|_| {
                        world_xz(
                            coordinate(&mut bytes, &mut lon),
                            coordinate(&mut bytes, &mut lat),
                        )
                    })
                    .collect()
            })
            .collect();
        tiles
            .entry(tile)
            .or_default()
            .push(StreetFace::road(rings, layer, road_use));
    }
    tiles
}

#[cfg(target_family = "wasm")]
#[derive(Default)]
pub(crate) struct DeferredRoads {
    loaded: bool,
    bytes: Vec<u8>,
    tiles: Vec<(Vec2, std::ops::Range<usize>)>,
}

#[cfg(target_family = "wasm")]
pub(crate) fn draw_deferred(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    rig: Res<crate::Rig>,
    scenario: Res<crate::net::Scenario>,
    buildings: Res<super::Buildings>,
    layers: Res<super::SurfaceLayers>,
    mask: Res<StreetMask>,
    mut deferred: Local<DeferredRoads>,
) {
    const DEFERRED: &str = "web/wasm/road-surface-deferred.bin";
    if !deferred.loaded {
        let moved = rig.distance != 1650. || rig.target.xz() != world_xz(-70.6505, -33.443);
        let edited = scenario.0["city_edits"].as_array().into_iter().flatten().any(|edit| {
            matches!(edit["kind"].as_str(), Some("road" | "building"))
        });
        if !(moved || edited) || !crate::data::load_later(DEFERRED) {
            return;
        }
        let bytes = crate::data::read(DEFERRED).unwrap();
        let mut at = 0;
        while at < bytes.len() {
            let tile = super::word(&bytes, at) as usize;
            let len = super::integer(&bytes, at + 2);
            let start = at + 6;
            deferred.tiles.push((buildings.boxes.center(tile).xz(), start..start + len));
            at = start + len;
        }
        deferred.bytes = bytes;
        deferred.loaded = true;
    }
    let target = rig.target.xz();
    let Some((index, _)) = deferred.tiles.iter().enumerate().min_by(|(_, a), (_, b)| {
        a.0.distance_squared(target).total_cmp(&b.0.distance_squared(target))
    }) else {
        return;
    };
    // Decode and triangulate one nearby tile per frame.
    let (_, range) = deferred.tiles.swap_remove(index);
    let faces = decode_faces(&deferred.bytes[range]).into_values().next().unwrap();
    let (tile, mut mesh, bounds, no_aabb) = StreetTile::bundle(faces, &mut meshes);
    if let Some(masked) = tile.masked(&mask) {
        mesh.0 = meshes.add(masked);
    }
    commands.spawn((tile, mesh, bounds, no_aabb, MeshMaterial3d(layers.materials[2].0.clone())));
    if deferred.tiles.is_empty() {
        deferred.bytes = Vec::new();
    }
}

pub(super) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    street_material: &Handle<StandardMaterial>,
    bridges: Vec<(Vec<Vec3>, f32, f32)>,
    scenery: Entity,
    scenery_material: &Handle<StandardMaterial>,
) {
    for faces in load_faces().into_values() {
        commands.spawn((
            StreetTile::bundle(faces, meshes),
            MeshMaterial3d(street_material.clone()),
        ));
    }

    // Detailed river bridges keep their scenery material, lift and subway visibility.
    let mut bridge_tiles = BTreeMap::<(i32, i32), Vec<StreetFace>>::new();
    for (points, half_width, lift) in bridges {
        let p = points[points.len() / 2];
        let key = ((p.x / 1700.).floor() as i32, (p.z / 1700.).floor() as i32);
        let faces = bridge_tiles.entry(key).or_default();
        ribbon_faces(&points, half_width, |ring| {
            let outline = ring.into_iter().map(|p| p.xz()).collect();
            faces.push(StreetFace::new(vec![outline], lift, ASPHALT));
        });
    }
    for faces in bridge_tiles.into_values().filter(|faces| !faces.is_empty()) {
        commands.spawn((
            ChildOf(scenery),
            StreetTile::bundle(faces, meshes),
            MeshMaterial3d(scenery_material.clone()),
        ));
    }
}
