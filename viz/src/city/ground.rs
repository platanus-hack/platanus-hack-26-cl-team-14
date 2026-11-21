use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};
use bevy::prelude::*;
use serde_json::Value;
use crate::city::{world, Surface};
use super::terrain::{terrain_from_xz, LandCover};
use flate2::read::GzDecoder;

type Color4 = [f32; 4];
type GroundTiles = BTreeMap<(i32, i32), Surface>;

// Half-widths are cartographic line widths in metres.
const ROAD: Color4 = [0.25, 0.27, 0.28, 1.0];
const SMALL_ROAD: Color4 = [0.31, 0.32, 0.32, 1.0];
const GRASS: Color4 = [0.23, 0.38, 0.18, 1.0];
const WATER: Color4 = [0.16, 0.41, 0.44, 1.0];
const STONE: Color4 = [0.62, 0.59, 0.48, 1.0];
const ROOF: Color4 = [0.33, 0.40, 0.37, 1.0];
const GLASS: Color4 = [0.13, 0.32, 0.37, 1.0];

fn coordinates(line: &Value) -> Vec<Vec3> {
    line.as_array().unwrap().iter().map(|pair| {
        world(pair[0].as_f64().unwrap(), pair[1].as_f64().unwrap())
    }).collect()
}

fn tile_for(tiles: &mut GroundTiles, point: Vec3) -> &mut Surface {
    let tile = ((point.x / 1700.0).floor() as i32, (point.z / 1700.0).floor() as i32);
    tiles.entry(tile).or_default()
}

// One shared pair of vertices at each join keeps streets continuous on bends.
fn ribbon(mesh: &mut Surface, points: &[Vec3], half_width: f32, elevation: f32, color: Color4) {
    let mut draped = Vec::with_capacity(points.len());
    for pair in points.windows(2) {
        let horizontal = Vec2::new(pair[1].x-pair[0].x, pair[1].z-pair[0].z).length();
        let parts = (horizontal / 35.0).ceil().max(1.0) as usize;
        for i in 0..parts {
            draped.push(pair[0].lerp(pair[1],i as f32 / parts as f32));
        }
    }
    if let Some(&last) = points.last() { draped.push(last); }
    let mut points = draped;
    points.dedup();
    if points.len() < 2 { return; }
    let normal = |a: Vec3, b: Vec3| Vec3::new(a.z - b.z, 0.0, b.x - a.x).normalize_or_zero();
    let first_vertex = mesh.positions.len() as u32;
    let last = points.len() - 1;
    for (i, &p) in points.iter().enumerate() {
        let previous = normal(points[i.saturating_sub(1)], p);
        let next = normal(p, points[(i + 1).min(last)]);
        let join = (previous + next).normalize_or_zero();
        let reference = if i == last { previous } else { next };
        let offset = join * (half_width / join.dot(reference).max(0.5));
        for mut edge in [p + offset, p - offset] {
            edge.y = terrain_from_xz(edge.x,edge.z) + elevation;
            mesh.positions.push(edge.to_array());
        }
        mesh.normals.extend([[0.0, 1.0, 0.0]; 2]);
        mesh.colors.extend([color; 2]);
        if i != last {
            let a = first_vertex + i as u32 * 2;
            mesh.indices.extend([a, a + 2, a + 1, a + 1, a + 2, a + 3]);
        }
    }
}

fn read_geojson(path: &Path) -> Value {
    let file = BufReader::new(File::open(path).unwrap());
    if path.extension().unwrap() == "gz" {
        serde_json::from_reader(BufReader::new(GzDecoder::new(file))).unwrap()
    } else {
        serde_json::from_reader(file).unwrap()
    }
}

fn open_rings(shape: &Value) -> Vec<Vec<Vec3>> {
    shape.as_array().unwrap().iter().map(|ring| {
        let mut points = coordinates(ring);
        points.pop(); // GeoJSON repeats its closing coordinate.
        points
    }).collect()
}

fn roads(tiles: &mut GroundTiles, path: &Path) {
    let fc = read_geojson(path);
    for feature in fc["features"].as_array().unwrap() {
        let points = coordinates(&feature["geometry"]["coordinates"]);
        let (half_width, color) = match feature["properties"]["kind"].as_str().unwrap() {
            "motorway" => (6.5, ROAD),
            "trunk" | "primary" => (5.0, ROAD),
            "secondary" => (4.0, ROAD),
            _ => (2.8, SMALL_ROAD),
        };
        ribbon(tile_for(tiles, points[0]), &points, half_width, 0.26, color);
    }
}

fn green_areas(tiles: &mut GroundTiles, terrain: &mut LandCover, path: &Path) {
    let data = read_geojson(path);
    for feature in data["features"].as_array().unwrap() {
        let color = match feature["properties"]["kind"].as_str().unwrap() {
            "wood" => [0.12,0.22,0.105,1.0],
            "scrub" | "heath" | "nature_reserve" => [0.37,0.40,0.20,1.0],
            "grass" | "meadow" | "grassland" => [0.34,0.43,0.22,1.0],
            _ => GRASS,
        };
        let rings = open_rings(&feature["geometry"]["coordinates"]);
        let (mut low,mut high) = (rings[0][0],rings[0][0]);
        for &p in &rings[0] { low = low.min(p); high = high.max(p); }
        let size = high-low;
        if size.x.max(size.z) > 180.0 {
            terrain.fill(&rings,color);
        }
        // Small lawns need vector outlines. Large hillside forests use terrain triangles.
        if size.x.max(size.z) < 720.0 {
            tile_for(tiles,rings[0][0]).polygon(&rings,0.18,0.0,color,color);
        }
    }
}

fn landmarks(tiles: &mut GroundTiles, path: &Path) {
    let fc = read_geojson(path);
    for feature in fc["features"].as_array().unwrap() {
        let props = &feature["properties"];
        let shape = &feature["geometry"]["coordinates"];
        let kind = props["kind"].as_str().unwrap();
        if kind == "peak" {
            // These are summit points. Terrain supplies their surfaces.
            continue;
        }
        if kind == "river" {
            let points = coordinates(shape);
            ribbon(tile_for(tiles, points[points.len() / 2]), &points, 15.0, 0.08, WATER);
            continue;
        }
        let rings = open_rings(shape);
        let tile = tile_for(tiles, rings[0][0]);
        match kind {
            "park" => (), // The citywide green pack includes these surveyed parks.
            "tower" => (),
            "palace" => tile.polygon(&rings, 0.0, props["levels"].as_f64().unwrap() as f32 * 3.5 + 0.05, STONE, ROOF),
            "stadium" => {
                // Mark the surveyed stadium perimeter. The source has no structure height.
                let mut perimeter = rings[0].clone();
                perimeter.push(perimeter[0]);
                ribbon(tile, &perimeter, 4.5, 0.18, STONE);
            }
            _ => (),
        }
    }
}

pub(crate) fn load_ground(root: &Path) -> Vec<(String, Mesh)> {
    let mut tiles = GroundTiles::new();
    let mut terrain = LandCover::default();
    green_areas(&mut tiles,&mut terrain,&root.join("web/data/green_areas.geojson.gz"));
    roads(&mut tiles,&root.join("web/data/residential_streets.geojson.gz"));
    roads(&mut tiles,&root.join("data/mvp/roads.geojson"));
    landmarks(&mut tiles, &root.join("web/data/santiago_landmarks.geojson"));
    let mut meshes = terrain.meshes();
    meshes.extend(tiles.into_iter().map(|((x,z),surface)| (format!("City ground {x} {z}"),surface.finish())));
    meshes
}
