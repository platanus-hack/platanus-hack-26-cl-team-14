mod activity;
pub(crate) mod edits;
mod facade;
mod ground;
pub(crate) mod outskirts;
pub(crate) use ground::StadiumSeating;
pub(crate) mod metro_service;
pub(crate) mod route_activity;
pub(crate) mod street_mask;
mod streets;
mod terrain;
mod trees;
pub(crate) mod water;
use bevy::camera::primitives::MeshAabb;
use bevy::mesh::morph::{MeshMorphWeights, MorphAttributes};
use bevy::prelude::*;
use std::path::Path;
pub(crate) use terrain::{extent, ferry_depth_grid};
pub use terrain::{cursor_ground, ground_on_line, terrain_from_xz};

/// Drawing offset in metres above the DEM. OSM layers describe crossing order.
pub fn road_surface_lift(layer: i32) -> f32 {
    0.26 + 0.04 * layer as f32
}

/// Local Santiago coordinates in meters. East is +X and south is +Z.
pub fn world(lon: f64, lat: f64) -> Vec3 {
    let Vec2 { x, y: z } = world_xz(lon, lat);
    Vec3::new(x, terrain_from_xz(x, z), z)
}

fn world_xz(lon: f64, lat: f64) -> Vec2 {
    let x = ((lon + 70.65) * 92_915.0) as f32;
    let z = (-(lat + 33.45) * 110_905.0) as f32;
    Vec2::new(x, z)
}

/// The inverse of `world`: [longitude, latitude] of a map point.
#[cfg(target_family = "wasm")]
pub fn lon_lat(point: Vec3) -> [f64; 2] {
    [point.x as f64 / 92_915.0 - 70.65, -(point.z as f64) / 110_905.0 - 33.45]
}

use bevy::{asset::RenderAssetUsages, mesh::Indices, render::render_resource::PrimitiveTopology};
use flate2::read::GzDecoder;
use std::collections::HashSet;
use std::io::Read;

pub type Rgba = [f32; 4];

#[derive(Default)]
pub struct Surface {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<Rgba>,
    pub indices: Vec<u32>,
    textured: bool,
    displacements: Vec<(usize, f32)>,
    uvs: Vec<[f32; 2]>,
}

impl Surface {
    fn vertex(&mut self, point: Vec3, normal: Vec3, color: Rgba) {
        self.positions.push(point.to_array());
        if self.textured {
            self.uvs.push([0.00625, 0.00625]);
        }
        self.normals.push(normal.to_array());
        self.colors.push(color);
    }

    // Each roof keeps the polygon's courtyards. Earcut handles concave buildings.
    pub fn polygon(&mut self, rings: &[Vec<Vec3>], base: f32, height: f32, wall: Rgba, roof: Rgba) {
        if rings.is_empty() || rings[0].len() < 3 {
            return;
        }
        let top = Vec3::Y * (base + height);
        let first = self.positions.len() as u32;
        let mut coords = Vec::new();
        let mut holes = Vec::new();
        for (j, ring) in rings.iter().enumerate() {
            if j > 0 {
                holes.push(coords.len() / 2);
            }
            for p in ring {
                self.vertex(*p + top, Vec3::Y, roof);
                coords.extend([p.x as f64, -p.z as f64]);
            }
        }
        if rings.len() == 1 && rings[0].len() == 4 && convex_quad(&rings[0]) {
            self.indices
                .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        } else if let Ok(triangles) = earcutr::earcut(&coords, &holes, 2) {
            self.indices
                .extend(triangles.into_iter().map(|i| first + i as u32));
        }
        if height == 0.0 {
            return;
        }
        for ring in rings {
            let signed: f32 = ring
                .iter()
                .zip(ring.iter().cycle().skip(1))
                .map(|(a, b)| a.x * b.z - b.x * a.z)
                .sum();
            for (a, b) in ring.iter().zip(ring.iter().cycle().skip(1)) {
                let n = (*b - *a).cross(Vec3::Y).normalize_or_zero() * -signed.signum();
                let first = self.positions.len() as u32;
                for p in [*a + Vec3::Y * base, *b + Vec3::Y * base, *b + top, *a + top] {
                    self.vertex(p, n, wall);
                }
                if self.textured {
                    let bays = (a.distance(*b) / 3.6).round().max(1.) / 4.;
                    let floors = (height / 3.5).round().max(1.) / 4.;
                    self.uvs[first as usize..].copy_from_slice(&[
                        [0., floors],
                        [bays, floors],
                        [bays, 0.],
                        [0., 0.],
                    ]);
                }
                self.indices
                    .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
            }
        }
    }

    fn height_transition(&mut self, first: usize, base: f32, from: f32, to: f32) {
        for (i, p) in self.positions[first..].iter_mut().enumerate() {
            let height = p[1] - base;
            p[1] = base + height * from;
            if from != to && height != 0. {
                self.displacements.push((first + i, height * (to - from)));
            }
        }
    }

    pub fn finish(self) -> Mesh {
        let n = self.positions.len();
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_indices(Indices::U32(self.indices));
        if !self.displacements.is_empty() {
            let mut offsets = vec![MorphAttributes::default(); n];
            for (i, y) in self.displacements {
                offsets[i] = MorphAttributes::new(Vec3::Y * y, Vec3::ZERO, Vec3::ZERO);
            }
            mesh.set_morph_targets(offsets);
        }
        if self.textured {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        }
        mesh
    }
}

fn convex_quad(p: &[Vec3]) -> bool {
    let cross = |i: usize| {
        (p[(i + 1) % 4] - p[i])
            .cross(p[(i + 2) % 4] - p[(i + 1) % 4])
            .y
    };
    (1..4).all(|i| cross(0) * cross(i) > 0.)
}

fn word(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(b[at..at + 2].try_into().unwrap())
}
fn integer(b: &[u8], at: usize) -> usize {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize
}

struct Pack {
    bytes: Vec<u8>,
    offsets: Vec<usize>,
    cols: usize,
    origin: Vec3,
    step: usize,
}

impl Pack {
    fn load(path: &str) -> Self {
        let mut bytes = Vec::new();
        GzDecoder::new(crate::data::open(path).unwrap())
            .read_to_end(&mut bytes)
            .unwrap();
        let cols = word(&bytes, 4) as usize;
        let n = cols * word(&bytes, 6) as usize;
        let coord = |offset| {
            i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as f64 / 100000.
        };
        let origin = world(coord(8), coord(12));
        let step = integer(&bytes, 24);
        let offsets = (0..=n).map(|i| integer(&bytes, 28 + i * 4)).collect();
        Self {
            bytes,
            offsets,
            cols,
            origin,
            step,
        }
    }

    fn position(&self, x: f32, y: f32) -> Vec3 {
        let x = self.origin.x + x * 0.92915;
        let z = self.origin.z - y * 1.10905;
        Vec3::new(x, terrain_from_xz(x, z), z)
    }

    fn center(&self, tile: usize) -> Vec3 {
        self.position(
            ((tile % self.cols) as f32 + 0.5) * self.step as f32,
            ((tile / self.cols) as f32 + 0.5) * self.step as f32,
        )
    }

    fn slice(&self, tile: usize) -> &[u8] {
        &self.bytes[self.offsets[tile]..self.offsets[tile + 1]]
    }
}

fn paint(x: u16, y: u16, height: f32) -> (Rgba, Rgba) {
    let seed = (x as u32).wrapping_mul(73856093) ^ (y as u32).wrapping_mul(19349663);
    let roof = if height > 30. {
        [
            [0.397, 0.497, 0.815, 1.],
            [0.527, 0.610, 0.871, 1.],
            [0.292, 0.386, 0.730, 1.],
        ][seed as usize % 3]
    } else {
        [
            [0.905, 0.896, 0.965, 1.],
            [0.791, 0.776, 0.905, 1.],
            [0.680, 0.658, 0.855, 1.],
            [0.584, 0.558, 0.791, 1.],
        ][seed as usize % 4]
    };
    let wall = [
        [0.102, 0.084, 0.381, 1.],
        [0.150, 0.133, 0.485, 1.],
        [0.068, 0.056, 0.275, 1.],
        [0.212, 0.195, 0.552, 1.],
        [0.050, 0.042, 0.202, 1.],
        [0.270, 0.238, 0.584, 1.],
    ][(seed >> 5) as usize % 6];
    (wall, roof)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Detail {
    Overview,
    Far,
    Near,
}

#[derive(Resource)]
pub(crate) struct Buildings {
    boxes: Pack,
    polygons: Pack,
    #[cfg(target_family = "wasm")]
    full_polygons: bool,
    tiles: Vec<Option<(Detail, Entity)>>,
    material: Handle<StandardMaterial>,
    facade_material: Handle<StandardMaterial>,
    pub ready: bool,
    pub pending_tiles: usize,
    dirty: Vec<bool>,
    heights: activity::HeightScales,
    hidden: HashSet<(usize, usize)>,
}

const FULL_POLYGONS: &str = "web/data/city_building_polygons.bin.gz";

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = santiagoPolygonsReady)]
    fn page_polygons_ready() -> bool;
}

#[cfg(target_family = "wasm")]
impl Buildings {
    fn load_full_polygons(&mut self) {
        if !self.full_polygons && page_polygons_ready() {
            self.polygons = Pack::load(FULL_POLYGONS);
            self.full_polygons = true;
        }
    }
}

fn box_mesh(
    pack: &Pack,
    tile: usize,
    detail: Detail,
    scale: &impl Fn(usize) -> (f32, f32),
    hidden: &HashSet<(usize, usize)>,
    only: Option<usize>,
) -> Surface {
    let bytes = pack.slice(tile);
    let count = integer(bytes, 0);
    let mut mesh = Surface::default();
    let overview = detail == Detail::Overview;
    let indices = only.map_or(0..count, |i| i..i + 1);
    mesh.positions.reserve(indices.len() * if overview { 4 } else { 8 });
    mesh.indices.reserve(indices.len() * if overview { 6 } else { 30 });
    for i in indices {
        if ground::NAMED.contains(&(tile, i)) || ground::HIDDEN.contains(&(tile, i)) || hidden.contains(&(tile, i)) {
            continue;
        }
        let at = 4 + i * 12;
        let x = word(bytes, at);
        let y = word(bytes, at + 2);
        let center = pack.position(x as f32, y as f32);
        let width = word(bytes, at + 4) as f32 * 0.05;
        let depth = word(bytes, at + 6) as f32 * 0.05;
        let (sin, cos) = (word(bytes, at + 8) as f32 * 0.01).to_radians().sin_cos();
        let u = Vec3::new(cos, 0., -sin) * width;
        let v = Vec3::new(sin, 0., cos) * depth;
        let h = word(bytes, at + 10) as f32 * 0.1;
        let (wall, roof) = paint(x, y, h);
        let base = mesh.positions.len() as u32;
        let corners = [
            center - u - v,
            center + u - v,
            center + u + v,
            center - u + v,
        ];
        let (from, to) = scale(i);
        // At 45 km a center pixel spans 26 m. Retain every roof and taller silhouettes.
        let roof_only = overview && (2. * width.max(depth)).max(h * from.max(to)) < 25.;
        if roof_only {
            for p in corners {
                mesh.vertex(p + Vec3::Y * h, Vec3::Y, roof);
            }
            mesh.height_transition(base as usize, center.y, from, to);
            mesh.indices.extend([0, 2, 1, 0, 3, 2].map(|n| base + n));
            continue;
        }
        for p in corners {
            mesh.vertex(p, Vec3::Y, wall);
        }
        for p in corners {
            mesh.vertex(p + Vec3::Y * h, Vec3::Y, roof);
        }
        mesh.height_transition(base as usize, center.y, from, to);
        mesh.indices.extend(
            [
                4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7, 3, 3, 7, 4, 3,
                4, 0,
            ]
            .map(|n| base + n),
        );
    }
    mesh
}

fn polygon_mesh(
    boxes: &Pack,
    polys: &Pack,
    tile: usize,
    scale: &impl Fn(usize) -> (f32, f32),
    hidden: &HashSet<(usize, usize)>,
    only: Option<usize>,
) -> Surface {
    let bytes = polys.slice(tile);
    let boxbytes = boxes.slice(tile);
    let count = integer(bytes, 0);
    let ring_table = 12 + (count + 1) * 4;
    let vertices = ring_table + (integer(bytes, 4) + 1) * 4;
    let heights = vertices + integer(bytes, 8) * 4;
    let mut mesh = Surface {
        textured: true,
        ..default()
    };
    for i in only.map_or(0..count, |i| i..i + 1) {
        if ground::NAMED.contains(&(tile, i)) || ground::HIDDEN.contains(&(tile, i)) || hidden.contains(&(tile, i)) {
            continue;
        }
        let first = integer(bytes, 12 + i * 4);
        let end = integer(bytes, 16 + i * 4);
        let center = boxes.position(
            word(boxbytes, 4 + i * 12) as f32,
            word(boxbytes, 6 + i * 12) as f32,
        );
        let rings: Vec<Vec<Vec3>> = (first..end)
            .map(|r| {
                (integer(bytes, ring_table + r * 4)..integer(bytes, ring_table + (r + 1) * 4))
                    .map(|v| {
                        let at = vertices + 4 * v;
                        let p = polys.position(word(bytes, at) as f32, word(bytes, at + 2) as f32);
                        Vec3::new(p.x, center.y, p.z)
                    })
                    .collect()
            })
            .collect();
        let h = word(bytes, heights + 2 * i) as f32 * 0.1;
        let (wall, roof) = paint(word(boxbytes, 4 + i * 12), word(boxbytes, 6 + i * 12), h);
        let start = mesh.positions.len();
        facade::building(&mut mesh, &rings, h, wall, roof);
        let (from, to) = scale(i);
        mesh.height_transition(start, center.y, from, to);
    }
    mesh
}

pub fn load_buildings(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let boxes = Pack::load("web/data/city_building_boxes.bin.gz");
    #[cfg(target_family = "wasm")]
    let polygons = Pack::load("web/wasm/building-polygons-startup.bin.gz");
    #[cfg(not(target_family = "wasm"))]
    let polygons = Pack::load(FULL_POLYGONS);
    let tiles = vec![None; boxes.offsets.len() - 1];
    let count: usize = (0..tiles.len())
        .map(|i| boxes.slice(i))
        .filter(|s| !s.is_empty())
        .map(|s| integer(s, 0))
        .sum();
    println!("Loaded {count} buildings from {} tiles", tiles.len());
    let material = materials.add(StandardMaterial {
        cull_mode: None,
        double_sided: false,
        perceptual_roughness: 0.94,
        ..default()
    });
    let facade_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(facade::window_texture(false))),
        emissive_texture: Some(images.add(facade::window_texture(true))),
        cull_mode: None,
        perceptual_roughness: 0.9,
        ..default()
    });
    let dirty = vec![false; tiles.len()];
    commands.insert_resource(Buildings {
        boxes,
        polygons,
        #[cfg(target_family = "wasm")]
        full_polygons: false,
        tiles,
        material,
        facade_material,
        ready: false,
        pending_tiles: 0,
        dirty,
        heights: activity::HeightScales::load(Path::new("web/data/city_building_block_ids.bin.gz")),
        hidden: HashSet::new(),
    });
}

#[derive(Component)]
pub(crate) struct HeightTransition(f32);

#[derive(Resource, Default)]
pub struct SubwayView(pub bool);

#[derive(Resource)]
pub(crate) struct SurfaceLayers {
    scenery: Entity,
    materials: [(Handle<StandardMaterial>, f32); 3],
}

pub(crate) fn toggle_subway(
    mut mode: ResMut<SubwayView>,
    layers: Res<SurfaceLayers>,
    buildings: Res<Buildings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut visibility: Query<&mut Visibility>,
    surface_edits: Query<Entity, With<edits::SurfaceEdit>>,
) {
    mode.0 = !mode.0;
    *visibility.get_mut(layers.scenery).unwrap() = if mode.0 {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for entity in &surface_edits {
        *visibility.get_mut(entity).unwrap() = if mode.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    for (handle, alpha) in layers
        .materials
        .iter()
        .map(|(handle, alpha)| (handle, *alpha))
        .chain([(&buildings.facade_material, 0.045)])
    {
        let mut material = materials.get_mut(handle).unwrap();
        material
            .base_color
            .set_alpha(if mode.0 { alpha } else { 1. });
        material.alpha_mode = if mode.0 {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        };
        material.unlit = mode.0;
    }
}

pub(crate) fn paper_light(
    night: Res<crate::Night>,
    title: Res<crate::boot::TitleLight>,
    mode: Res<SubwayView>,
    buildings: Res<Buildings>,
    layers: Res<SurfaceLayers>,
    mut sky: ResMut<ClearColor>,
    mut sun: Single<&mut DirectionalLight>,
    mut ambient: Single<&mut AmbientLight>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !night.is_changed() && !mode.is_changed() && !title.is_changed() {
        return;
    }
    let dark = title.0.unwrap_or(if night.0 && !mode.0 { 1. } else { 0. });
    let blend = |day: Color, night: Color| Color::from(day.to_srgba().mix(&night.to_srgba(), dark));
    sky.0 = if mode.0 && title.0.is_none() {
        Color::srgb_u8(32, 43, 54)
    } else {
        blend(crate::DAY_SKY, Color::srgb_u8(57, 65, 91))
    };
    sun.illuminance = 12_000. - 10_300. * dark;
    sun.color = blend(Color::srgb_u8(255, 245, 231), Color::srgb_u8(184, 184, 190));
    ambient.brightness = 950. - 650. * dark;
    ambient.color = blend(Color::srgb_u8(226, 228, 255), Color::srgb_u8(164, 164, 173));
    let (street, _) = &layers.materials[2];
    materials.get_mut(street).unwrap().emissive = LinearRgba::rgb(0.13 * dark, 0.14 * dark, 0.15 * dark);
    materials.get_mut(&buildings.facade_material).unwrap().emissive = LinearRgba::rgb(1.4 * dark, 1.15 * dark, 0.84 * dark);
}

pub(crate) fn draw_buildings(
    mut commands: Commands,
    mut buildings: ResMut<Buildings>,
    mut meshes: ResMut<Assets<Mesh>>,
    rig: Res<crate::Rig>,
    scenario: Res<crate::net::Scenario>,
    layers: Res<SurfaceLayers>,
    time: Res<Time>,
    mut weights: Query<(&HeightTransition, &mut MeshMorphWeights)>,
    mut named: Query<(&NamedBuilding, &mut Transform)>,
) {
    if scenario.is_changed()
        && buildings.heights.update(&scenario.0, time.elapsed_secs())
    {
        for tile in 0..buildings.tiles.len() {
            if buildings.tiles[tile].is_some() && buildings.heights.tile_changed(tile) {
                buildings.dirty[tile] = true;
            }
        }
    }
    let progress = buildings.heights.progress(time.elapsed_secs());
    for (key, mut transform) in &mut named {
        let (from, to) = buildings.heights.scale(key.0, key.1);
        transform.scale.y = from + (to - from) * progress;
    }
    for (transition, mut morph) in &mut weights {
        let t = ((time.elapsed_secs() - transition.0) / 1.2).clamp(0., 1.);
        let t = t * t * (3. - 2. * t);
        if let MeshMorphWeights::Value { weights } = morph.as_ref()
            && weights[0] != t
        {
            if let MeshMorphWeights::Value { weights } = &mut *morph {
                weights[0] = t;
            }
        }
    }
    let mut jobs = Vec::new();
    #[cfg(target_family = "wasm")]
    let mut pending_polygons = 0;
    for tile in 0..buildings.tiles.len() {
        if buildings.boxes.slice(tile).is_empty() {
            continue;
        }
        let d = buildings.boxes.center(tile).distance(rig.target);
        // A tile is roughly 2 km wide. Include full tiles on either side of the visible frustum.
        let desired = if d > rig.distance * 1.5 + 1700. {
            None
        } else if rig.distance < 4200. && d < rig.distance * 0.72 + 1500. {
            Some(Detail::Near)
        } else if rig.distance >= 45_000. {
            Some(Detail::Overview)
        } else {
            Some(Detail::Far)
        };
        #[cfg(target_family = "wasm")]
        let desired = if desired == Some(Detail::Near) && buildings.polygons.slice(tile).is_empty() {
            if pending_polygons == 0 {
                buildings.load_full_polygons();
            }
            if buildings.polygons.slice(tile).is_empty() {
                pending_polygons += 1;
                Some(buildings.tiles[tile].map_or(Detail::Far, |(detail, _)| detail))
            } else {
                desired
            }
        } else {
            desired
        };
        if !buildings.dirty[tile] && buildings.tiles[tile].map(|t| t.0) == desired {
            continue;
        }
        jobs.push((d, tile, desired));
    }
    buildings.pending_tiles = jobs.len();
    #[cfg(target_family = "wasm")]
    {
        buildings.pending_tiles += pending_polygons;
    }
    buildings.ready = buildings.pending_tiles == 0;
    jobs.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut near = false;
    for (_, tile, detail) in jobs.into_iter().take(8) {
        // A detailed tile costs about 30 ms. Build one per frame to keep a flight smooth.
        if detail == Some(Detail::Near) && std::mem::replace(&mut near, true) {
            continue;
        }
        buildings.dirty[tile] = false;
        if let Some((_, entity)) = buildings.tiles[tile].take() {
            commands.entity(entity).despawn();
        }
        if let Some(detail) = detail {
            let mesh = match detail {
                Detail::Overview | Detail::Far => box_mesh(
                    &buildings.boxes,
                    tile,
                    detail,
                    &|i| buildings.heights.scale(tile, i),
                    &buildings.hidden,
                    None,
                ),
                Detail::Near => polygon_mesh(
                    &buildings.boxes,
                    &buildings.polygons,
                    tile,
                    &|i| buildings.heights.scale(tile, i),
                    &buildings.hidden,
                    None,
                ),
            };
            let mat = if detail == Detail::Near {
                &buildings.facade_material
            } else {
                &buildings.material
            };
            let animated = !mesh.displacements.is_empty();
            let mut chunk = commands.spawn((
                Mesh3d(meshes.add(mesh.finish())),
                MeshMaterial3d(mat.clone()),
            ));
            if detail != Detail::Near {
                chunk.insert((ChildOf(layers.scenery), bevy::light::NotShadowCaster));
            }
            if animated {
                chunk.insert((
                    MeshMorphWeights::Value {
                        weights: vec![progress],
                    },
                    HeightTransition(buildings.heights.started),
                ));
            }
            let entity = chunk.id();
            buildings.tiles[tile] = Some((detail, entity));
        }
    }
}

#[derive(Component)]
pub(crate) struct NamedBuilding(usize, usize);

// Keep seating, route lookup and activity height on the landmark's stable anchor.
#[derive(Component)]
pub(crate) struct BuildingVisual(pub(crate) Entity);

impl NamedBuilding {
    pub(crate) fn is_monumental(&self) -> bool {
        (self.0, self.1) == ground::NAMED[4]
    }
}

pub(crate) fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let surface = StandardMaterial {
        cull_mode: None,
        perceptual_roughness: 1.,
        ..default()
    };
    let terrain_mat = materials.add(surface.clone());
    let street_mat = materials.add(surface.clone());
    let mat = materials.add(surface);
    let root = commands
        .spawn((Transform::default(), Visibility::Inherited))
        .id();
    let foliage = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(trees::crowns())),
        alpha_mode: AlphaMode::Mask(0.45),
        cull_mode: None,
        perceptual_roughness: 1.,
        ..default()
    });
    for mesh in ground::mapped_trees(Path::new("web/data/city_trees.bin.gz")) {
        commands.spawn((ChildOf(root), Mesh3d(meshes.add(mesh)), MeshMaterial3d(foliage.clone())));
    }
    for ((tile, i), base, mesh, seating) in ground::named_buildings(Path::new(".")) {
        let visual = commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(mat.clone()),
        )).id();
        let mut entity = commands.spawn((
            NamedBuilding(tile, i),
            BuildingVisual(visual),
            Transform::from_xyz(0., base, 0.),
            Visibility::Inherited,
        ));
        entity.add_child(visual);
        if let Some(seating) = seating {
            entity.insert(seating);
        }
    }
    let (terrain, scenery, bridges) = ground::load_ground(Path::new("."));
    for (name, mesh) in terrain {
        let bounds = mesh.get_aabb().unwrap();
        let mesh = meshes.add(mesh);
        outskirts::spawn(&mut commands, root, extent(), bounds, &mesh, &terrain_mat);
        commands.spawn((
            Name::new(name),
            Mesh3d(mesh),
            MeshMaterial3d(terrain_mat.clone()),
        ));
    }
    for (name, mesh) in scenery {
        commands.spawn((
            ChildOf(root),
            Name::new(name),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(mat.clone()),
        ));
    }
    streets::spawn(
        &mut commands,
        &mut meshes,
        &street_mat,
        bridges,
        root,
        &mat,
    );
    commands.insert_resource(SurfaceLayers {
        scenery: root,
        materials: [(mat, 0.045), (terrain_mat, 0.08), (street_mat, 0.32)],
    });
}
