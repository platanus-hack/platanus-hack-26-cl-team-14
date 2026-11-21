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
            "arena" => arena(tile, &rings, props["height_m"].as_f64().unwrap() as f32),
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
    mapped_trees(&mut tiles,&root.join("web/data/city_trees.bin.gz"));
    let mut meshes = terrain.meshes();
    meshes.extend(tiles.into_iter().map(|((x,z),surface)| (format!("City ground {x} {z}"),surface.finish())));
    meshes
}

// Append these functions to the ground module. Tree size and foliage are display symbols.
// The packed coordinates are individual OSM tree nodes.
fn mapped_trees(tiles: &mut GroundTiles, path: &Path) {
    use std::io::Read;
    let mut bytes = Vec::new();
    GzDecoder::new(File::open(path).unwrap()).read_to_end(&mut bytes).unwrap();
    const LEAVES: [Color4;4] = [
        [0.16, 0.31, 0.10, 1.], [0.10, 0.24, 0.085, 1.],
        [0.24, 0.36, 0.12, 1.], [0.13, 0.28, 0.15, 1.],
    ];
    let bark = [0.19,0.13,0.08,1.];
    for pair in bytes.chunks_exact(4) {
        let x = u16::from_le_bytes([pair[0],pair[1]]) as u32;
        let y = u16::from_le_bytes([pair[2],pair[3]]) as u32;
        let base = world(-70.85 + x as f64/100_000.,-33.65 + y as f64/100_000.);
        let mesh = tile_for(tiles,base);
        let seed = x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663);
        let height = 5.8 + (seed % 41) as f32 * 0.10;
        let radius = height * (0.31 + ((seed>>8)%8) as f32 * 0.012);
        let phase = (seed%313) as f32 * 0.020; 
        let leaves = LEAVES[(seed as usize >> 7) % LEAVES.len()];
        let mut ring = |sides: u32, width: f32, elevation: f32, normal_y: f32, color: Color4| {
            let start = mesh.positions.len() as u32;
            for n in 0..sides {
                let (s,c) = (phase + n as f32 * std::f32::consts::TAU / sides as f32).sin_cos();
                let dir = Vec3::new(c,0.,s);
                mesh.positions.push((base + dir*width + Vec3::Y*elevation).to_array());
                mesh.normals.push((dir + Vec3::Y*normal_y).normalize().to_array());
                mesh.colors.push(color);
            }
            start
        };
        let trunk = ring(3,0.17,0.0,0.0,bark);
        ring(3,0.14,height*0.47,0.0,bark);
        let crown = ring(5,radius*0.55,height*0.32,-0.75,leaves);
        ring(5,radius,height*0.60,0.0,leaves);
        ring(5,radius*0.65,height*0.86,0.64,leaves);
        for (start,sides) in [(trunk,3u32),(crown,5),(crown+5,5)] {
            for n in 0..sides {
                let a = start+n;
                let b = start+(n+1)%sides;
                mesh.indices.extend([a,a+sides,b,b,a+sides,b+sides]);
            }
        }
        let tip = mesh.positions.len() as u32;
        mesh.positions.push((base+Vec3::Y*height).to_array());
        mesh.normals.push(Vec3::Y.to_array());
        mesh.colors.push(leaves);
        for n in 0..5 {
            mesh.indices.extend([crown+10+n,tip,crown+10+(n+1)%5]);
        }
    }
}

fn arena(mesh: &mut Surface, rings: &[Vec<Vec3>], height: f32) {
    // The long entrance wings belong to the OSM outline but not to the dome.
    // Fit the dome ellipse to points on its north and south arcs.
    let ring = &rings[0];
    let north = ring.iter().min_by(|a,b|a.z.total_cmp(&b.z)).unwrap();
    let south = ring.iter().max_by(|a,b|a.z.total_cmp(&b.z)).unwrap();
    let center = (*north+*south)*0.5;
    let rz = (south.z - north.z)*0.5;
    let mut radii: Vec<f32> = ring.iter().filter_map(|p| {
        let t = ((p.z-center.z)/rz).abs();
        (0.35..0.9).contains(&t).then(|| (p.x-center.x).abs() / (1.-t*t).sqrt())
    }).collect();
    radii.sort_by(f32::total_cmp);
    let rx = radii[radii.len()/2];
    let roof = [0.761,0.738,0.738,1.]; // OSM roof colour #e2dfdf, linear RGB
    let plinth = height*0.24;
    mesh.polygon(rings,0.,plinth,STONE,roof);
    let n = 36u32;
    let bottom = mesh.positions.len() as u32;
    let center_y = terrain_from_xz(center.x,center.z);
    for level in 0..5 {
        let alpha = level as f32 * std::f32::consts::FRAC_PI_2 / 5.;
        let (vertical,radial) = alpha.sin_cos();
        for i in 0..n {
            let (s,c) = (i as f32 * std::f32::consts::TAU/n as f32).sin_cos();
            mesh.positions.push([center.x+rx*c*radial,center_y+plinth+(height-plinth)*vertical,center.z+rz*s*radial]);
            mesh.normals.push(Vec3::new(c*radial/rx,vertical/(height-plinth),s*radial/rz).normalize().to_array());
            mesh.colors.push(roof);
            if level < 4 {
                let a = bottom + level*n+i;
                let b = bottom + level*n+(i+1)%n;
                mesh.indices.extend([a,a+n,b,b,a+n,b+n]);
            }
        }
    }
    let tip = mesh.positions.len() as u32;
    mesh.positions.push([center.x,center_y+height,center.z]);
    mesh.normals.push(Vec3::Y.to_array());
    mesh.colors.push(roof);
    for i in 0..n { mesh.indices.extend([bottom+4*n+i,tip,bottom+4*n+(i+1)%n]); }
}
