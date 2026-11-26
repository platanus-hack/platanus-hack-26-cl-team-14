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
use serde::Deserialize;
use std::{collections::BTreeMap, io::BufReader, path::Path};

#[derive(Deserialize)]
struct Roads {
    features: Vec<Road>,
}

#[derive(Deserialize)]
struct Road {
    geometry: Polygon,
    properties: RoadProperties,
}

#[derive(Deserialize)]
struct Polygon {
    coordinates: Vec<Vec<[f64; 2]>>,
}

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
        if mask.intersects(&tile.bounds) && tile.faces.iter().any(|f| mask.intersects(&f.bounds)) {
            current.0 = meshes.add(mesh_for(&tile.faces, Some(mask)));
        } else if current.0 != tile.original {
            current.0 = tile.original.clone();
        }
    }
}

pub(super) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    path: &Path,
    street_material: &Handle<StandardMaterial>,
    bridges: Vec<(Vec<Vec3>, f32, f32)>,
    scenery: Entity,
    scenery_material: &Handle<StandardMaterial>,
) {
    let gz = GzDecoder::new(crate::data::open(path).unwrap());
    let roads: Roads = serde_json::from_reader(BufReader::new(gz)).unwrap();
    let mut tiles = BTreeMap::<usize, Vec<StreetFace>>::new();
    for road in roads.features {
        let p = road.properties;
        if p.structure == "tunnel" {
            continue;
        }
        let (color, extra_lift) = match p.road_use.as_str() {
            "sidewalk" => ([0.66, 0.67, 0.68, 1.], 0.03),
            "foot" => ([0.47, 0.48, 0.49, 1.], 0.02),
            "cycle" => ([0.30, 0.31, 0.32, 1.], 0.01),
            _ => (ASPHALT, 0.),
        };
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
        let lift = super::road_surface_lift(p.layer) + extra_lift;
        tiles
            .entry(p.tile)
            .or_default()
            .push(StreetFace::new(rings, lift, color));
    }
    for faces in tiles.into_values() {
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
