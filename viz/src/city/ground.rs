use super::terrain::{LandCover, terrain_from_xz};
use crate::city::{Surface, world};
use bevy::prelude::*;
use flate2::read::GzDecoder;
use serde_json::Value;
use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};

type Color4 = [f32; 4];
type GroundTiles = BTreeMap<(i32, i32), Surface>;

// Half-widths are cartographic line widths in metres.
const GRASS: Color4 = [0.23, 0.38, 0.18, 1.0];
const WATER: Color4 = [0.16, 0.41, 0.44, 1.0];
const STONE: Color4 = [0.62, 0.59, 0.48, 1.0];
const ROOF: Color4 = [0.33, 0.40, 0.37, 1.0];

fn coordinates(line: &Value) -> Vec<Vec3> {
    line.as_array()
        .unwrap()
        .iter()
        .map(|pair| world(pair[0].as_f64().unwrap(), pair[1].as_f64().unwrap()))
        .collect()
}

fn tile_for(tiles: &mut GroundTiles, point: Vec3) -> &mut Surface {
    let tile = (
        (point.x / 1700.0).floor() as i32,
        (point.z / 1700.0).floor() as i32,
    );
    tiles.entry(tile).or_default()
}

// One shared pair of vertices at each join keeps streets continuous on bends.
fn ribbon(mesh: &mut Surface, points: &[Vec3], half_width: f32, elevation: f32, color: Color4) {
    let mut draped = Vec::with_capacity(points.len());
    for pair in points.windows(2) {
        let horizontal = Vec2::new(pair[1].x - pair[0].x, pair[1].z - pair[0].z).length();
        let parts = (horizontal / 35.0).ceil().max(1.0) as usize;
        for i in 0..parts {
            draped.push(pair[0].lerp(pair[1], i as f32 / parts as f32));
        }
    }
    if let Some(&last) = points.last() {
        draped.push(last);
    }
    let mut points = draped;
    points.dedup();
    if points.len() < 2 {
        return;
    }
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
            edge.y = terrain_from_xz(edge.x, edge.z) + elevation;
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
    shape
        .as_array()
        .unwrap()
        .iter()
        .map(|ring| {
            let mut points = coordinates(ring);
            points.pop(); // GeoJSON repeats its closing coordinate.
            points
        })
        .collect()
}

fn land_areas(tiles: &mut GroundTiles, terrain: &mut LandCover, path: &Path) {
    let data = read_geojson(path);
    for feature in data["features"].as_array().unwrap() {
        let kind = feature["properties"]["kind"].as_str().unwrap();
        let color = match kind {
            "track" => [0.38, 0.13, 0.08, 1.0],
            "pitch" => [0.18, 0.38, 0.21, 1.0],
            "surface_parking" => [0.27, 0.28, 0.28, 1.0],
            "parking_unspecified" => [0.32, 0.32, 0.31, 1.0],
            "brownfield" => [0.32, 0.25, 0.17, 1.0],
            "greenfield" => [0.35, 0.37, 0.21, 1.0],
            "wood" => [0.12, 0.22, 0.105, 1.0],
            "scrub" | "heath" | "nature_reserve" => [0.37, 0.40, 0.20, 1.0],
            "grass" | "meadow" | "grassland" => [0.34, 0.43, 0.22, 1.0],
            _ => GRASS,
        };
        let rings = open_rings(&feature["geometry"]["coordinates"]);
        let (mut low, mut high) = (rings[0][0], rings[0][0]);
        for &p in &rings[0] {
            low = low.min(p);
            high = high.max(p);
        }
        let size = high - low;
        if size.x.max(size.z) > 180.0 {
            terrain.fill(&rings, color);
        }
        // Small lawns need vector outlines. Large hillside forests use terrain triangles.
        if size.x.max(size.z) < 720.0 {
            // Specific uses can overlap broad land use. Roads are drawn at 0.26 m.
            let lift = match kind {
                "surface_parking" | "parking_unspecified" | "track" => 0.22,
                // Some mapped tracks include the infield in their exterior polygon.
                "pitch" => 0.24,
                _ => 0.18,
            };
            super::terrain::drape_land(tile_for(tiles, rings[0][0]), &rings, lift, color);
        }
    }
}

fn mapocho(tiles: &mut GroundTiles, root: &Path) {
    let fc = read_geojson(&root.join("web/data/mapocho_detail.geojson"));
    let mut extent = Vec2::new(f32::INFINITY, f32::NEG_INFINITY);
    for feature in fc["features"].as_array().unwrap() {
        let props = &feature["properties"];
        let shape = &feature["geometry"]["coordinates"];
        let kind = props["kind"].as_str().unwrap();
        if kind == "water" {
            for polygon in shape.as_array().unwrap() {
                let rings = open_rings(polygon);
                for p in &rings[0] {
                    extent.x = extent.x.min(p.x);
                    extent.y = extent.y.max(p.x);
                }
                super::terrain::drape_land(tile_for(tiles, rings[0][0]), &rings, 0.08, WATER);
            }
        } else if kind == "bridge" {
            let points = coordinates(shape);
            // Widths and the terrain offset are display symbols, not measured bridge dimensions.
            // OSM layer gives crossing order, not metres of clearance.
            let half_width = match props["highway"].as_str().unwrap() {
                "motorway" | "motorway_link" => 6.5,
                "trunk" | "trunk_link" | "primary" | "primary_link" => 5.0,
                "secondary" | "secondary_link" => 4.0,
                "footway" | "cycleway" | "path" | "steps" | "pedestrian" => 1.0,
                _ => 2.8,
            };
            ribbon(
                tile_for(tiles, points[points.len() / 2]),
                &points,
                half_width,
                crate::city::road_surface_lift(props["layer"].as_i64().unwrap() as i32) + 0.08,
                [0.38, 0.38, 0.36, 1.0],
            );
        }
    }
    // Keep the mapped river symbol beyond the detailed pack's west and east bounds.
    // The committed centerline crosses each bound once; each exterior reach is contiguous.
    let landmarks = read_geojson(&root.join("web/data/santiago_landmarks.geojson"));
    for feature in landmarks["features"].as_array().unwrap() {
        if feature["properties"]["kind"] != "river" {
            continue;
        }
        let points = coordinates(&feature["geometry"]["coordinates"]);
        for (bound, sign) in [(extent.x, -1.0), (extent.y, 1.0)] {
            let outside = |p: Vec3| (p.x - bound) * sign > 0.0;
            let mut reach = Vec::new();
            for pair in points.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                if outside(a) {
                    reach.push(a);
                }
                if outside(a) != outside(b) {
                    reach.push(a.lerp(b, (bound - a.x) / (b.x - a.x)));
                }
            }
            let last = *points.last().unwrap();
            if outside(last) {
                reach.push(last);
            }
            ribbon(tile_for(tiles, reach[0]), &reach, 15.0, 0.08, WATER);
        }
    }
}

pub(super) fn load_ground(root: &Path) -> (Vec<(String, Mesh)>, Vec<(String, Mesh)>) {
    let mut tiles = GroundTiles::new();
    let mut terrain = LandCover::default();
    land_areas(
        &mut tiles,
        &mut terrain,
        &root.join("web/data/green_areas.geojson.gz"),
    );
    land_areas(
        &mut tiles,
        &mut terrain,
        &root.join("web/data/open_surfaces.geojson.gz"),
    );
    mapocho(&mut tiles, root);
    mapped_trees(&mut tiles, &root.join("web/data/city_trees.bin.gz"));
    let scenery = tiles
        .into_iter()
        .map(|((x, z), surface)| (format!("City ground {x} {z}"), surface.finish()))
        .collect();
    (terrain.meshes(), scenery)
}

// Append these functions to the ground module. Tree size and foliage are display symbols.
// The packed coordinates are individual OSM tree nodes.
fn mapped_trees(tiles: &mut GroundTiles, path: &Path) {
    use std::io::Read;
    let mut bytes = Vec::new();
    GzDecoder::new(File::open(path).unwrap())
        .read_to_end(&mut bytes)
        .unwrap();
    const LEAVES: [Color4; 4] = [
        [0.16, 0.31, 0.10, 1.],
        [0.10, 0.24, 0.085, 1.],
        [0.24, 0.36, 0.12, 1.],
        [0.13, 0.28, 0.15, 1.],
    ];
    let bark = [0.19, 0.13, 0.08, 1.];
    for pair in bytes.chunks_exact(4) {
        let x = u16::from_le_bytes([pair[0], pair[1]]) as u32;
        let y = u16::from_le_bytes([pair[2], pair[3]]) as u32;
        let base = world(-70.85 + x as f64 / 100_000., -33.65 + y as f64 / 100_000.);
        let mesh = tile_for(tiles, base);
        let seed = x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663);
        let height = 5.8 + (seed % 41) as f32 * 0.10;
        let radius = height * (0.31 + ((seed >> 8) % 8) as f32 * 0.012);
        let phase = (seed % 313) as f32 * 0.020;
        let leaves = LEAVES[(seed as usize >> 7) % LEAVES.len()];
        let mut ring = |sides: u32, width: f32, elevation: f32, normal_y: f32, color: Color4| {
            let start = mesh.positions.len() as u32;
            for n in 0..sides {
                let (s, c) = (phase + n as f32 * std::f32::consts::TAU / sides as f32).sin_cos();
                let dir = Vec3::new(c, 0., s);
                mesh.positions
                    .push((base + dir * width + Vec3::Y * elevation).to_array());
                mesh.normals
                    .push((dir + Vec3::Y * normal_y).normalize().to_array());
                mesh.colors.push(color);
            }
            start
        };
        let trunk = ring(3, 0.17, 0.0, 0.0, bark);
        ring(3, 0.14, height * 0.47, 0.0, bark);
        let crown = ring(5, radius * 0.55, height * 0.32, -0.75, leaves);
        ring(5, radius, height * 0.60, 0.0, leaves);
        ring(5, radius * 0.65, height * 0.86, 0.64, leaves);
        for (start, sides) in [(trunk, 3u32), (crown, 5), (crown + 5, 5)] {
            for n in 0..sides {
                let a = start + n;
                let b = start + (n + 1) % sides;
                mesh.indices
                    .extend([a, a + sides, b, b, a + sides, b + sides]);
            }
        }
        let tip = mesh.positions.len() as u32;
        mesh.positions.push((base + Vec3::Y * height).to_array());
        mesh.normals.push(Vec3::Y.to_array());
        mesh.colors.push(leaves);
        for n in 0..5 {
            mesh.indices
                .extend([crown + 10 + n, tip, crown + 10 + (n + 1) % 5]);
        }
    }
}

fn arena(mesh: &mut Surface, rings: &[Vec<Vec3>], height: f32) {
    // The long entrance wings belong to the OSM outline but not to the dome.
    // Fit the dome ellipse to points on its north and south arcs.
    let ring = &rings[0];
    let north = ring.iter().min_by(|a, b| a.z.total_cmp(&b.z)).unwrap();
    let south = ring.iter().max_by(|a, b| a.z.total_cmp(&b.z)).unwrap();
    let center = (*north + *south) * 0.5;
    let rz = (south.z - north.z) * 0.5;
    let mut radii: Vec<f32> = ring
        .iter()
        .filter_map(|p| {
            let t = ((p.z - center.z) / rz).abs();
            (0.35..0.9)
                .contains(&t)
                .then(|| (p.x - center.x).abs() / (1. - t * t).sqrt())
        })
        .collect();
    radii.sort_by(f32::total_cmp);
    let rx = radii[radii.len() / 2];
    let roof = [0.761, 0.738, 0.738, 1.]; // OSM roof colour #e2dfdf, linear RGB
    let plinth = height * 0.24;
    mesh.polygon(rings, 0., plinth, STONE, roof);
    let n = 36u32;
    let bottom = mesh.positions.len() as u32;
    let center_y = terrain_from_xz(center.x, center.z);
    for level in 0..5 {
        let alpha = level as f32 * std::f32::consts::FRAC_PI_2 / 5.;
        let (vertical, radial) = alpha.sin_cos();
        for i in 0..n {
            let (s, c) = (i as f32 * std::f32::consts::TAU / n as f32).sin_cos();
            mesh.positions.push([
                center.x + rx * c * radial,
                center_y + plinth + (height - plinth) * vertical,
                center.z + rz * s * radial,
            ]);
            mesh.normals.push(
                Vec3::new(
                    c * radial / rx,
                    vertical / (height - plinth),
                    s * radial / rz,
                )
                .normalize()
                .to_array(),
            );
            mesh.colors.push(roof);
            if level < 4 {
                let a = bottom + level * n + i;
                let b = bottom + level * n + (i + 1) % n;
                mesh.indices.extend([a, a + n, b, b, a + n, b + n]);
            }
        }
    }
    let tip = mesh.positions.len() as u32;
    mesh.positions.push([center.x, center_y + height, center.z]);
    mesh.normals.push(Vec3::Y.to_array());
    mesh.colors.push(roof);
    for i in 0..n {
        mesh.indices
            .extend([bottom + 4 * n + i, tip, bottom + 4 * n + (i + 1) % n]);
    }
}

// Ground module helpers. Colors are linear RGB. The facade divisions are display geometry.
fn face(mesh: &mut Surface, p: [Vec3; 4], towards: Vec3, color: Color4) {
    let mut normal = (p[1] - p[0]).cross(p[3] - p[0]).normalize_or_zero();
    let reverse = normal.dot(towards) < 0.;
    if reverse {
        normal = -normal;
    }
    let first = mesh.positions.len() as u32;
    for v in p {
        mesh.positions.push(v.to_array());
        mesh.normals.push(normal.to_array());
        mesh.colors.push(color);
    }
    let indices = if reverse {
        [0, 2, 1, 0, 3, 2]
    } else {
        [0, 1, 2, 0, 2, 3]
    };
    mesh.indices.extend(indices.map(|i| first + i));
}

// The architect describes faceted walls, folded centerlines, open corners and a latticed crown.
// https://pcparch.com/work/gran-torre-santiago
fn gran_torre(mesh: &mut Surface, rings: &[Vec<Vec3>], height: f32, levels: usize) {
    let ring = &rings[0];
    let center = ring.iter().copied().sum::<Vec3>() / ring.len() as f32;
    let offsets: Vec<_> = ring
        .iter()
        .map(|p| Vec3::new(p.x - center.x, 0., p.z - center.z))
        .collect();
    let glass = [
        [0.085, 0.21, 0.29, 1.],
        [0.10, 0.23, 0.30, 1.],
        [0.13, 0.26, 0.32, 1.],
        [0.095, 0.20, 0.285, 1.],
    ];
    let frame = [0.23, 0.34, 0.385, 1.];
    let spandrel = [0.105, 0.18, 0.23, 1.];
    for i in 0..ring.len() {
        let a = offsets[i];
        let b = offsets[(i + 1) % ring.len()];
        let normal = (a + b).normalize_or_zero();
        let point = |u: f32, t: f32, lift: f32| {
            // The OSM shell has four corners at even indices and four face ridges between them.
            let gap = ((t - 0.84) / 0.16).clamp(0., 1.) * 0.22;
            let u = if i % 2 == 0 {
                gap + u * (1. - gap)
            } else {
                u * (1. - gap)
            };
            center + a.lerp(b, u) * (1. - 0.28 * t * t) + Vec3::Y * (height * t) + normal * lift
        };
        let mut panel = |u0, u1, t0, t1, lift, c| {
            face(
                mesh,
                [
                    point(u0, t0, lift),
                    point(u1, t0, lift),
                    point(u1, t1, lift),
                    point(u0, t1, lift),
                ],
                normal,
                c,
            );
        };
        for floor in 0..levels {
            let t0 = floor as f32 / levels as f32;
            let t1 = (floor + 1) as f32 / levels as f32;
            let window_top = t1 - (t1 - t0) * 0.09;
            panel(0., 1., t0, window_top, 0., glass[(i / 2 + i % 2) % 4]);
            panel(0., 1., window_top, t1, 0., spandrel);
        }
        // Short lengths follow the taper; vertical ribs remain visible at close zoom.
        for level in 0..8 {
            let t0 = level as f32 / 8.;
            let t1 = (level + 1) as f32 / 8.;
            for pane in 0..6 {
                let u = pane as f32 / 6.;
                panel(u, u + 0.010, t0, t1, 0.035, frame);
            }
        }
    }
    // The four glass walls rise above an inset roof. The lattice crosses their open corners.
    let deck: Vec<Vec3> = offsets.iter().map(|p| center + *p * 0.70).collect();
    mesh.polygon(&[deck.clone()], height * 0.885, 0., frame, spandrel);
    for (a, b) in deck.iter().zip(deck.iter().cycle().skip(1)) {
        let normal = (*a + *b) * 0.5 - center;
        for k in 0..7 {
            let low = Vec3::Y * (height * (0.90 + 0.012 * k as f32));
            let high = low + Vec3::Y * (height * 0.002);
            face(
                mesh,
                [*a + low, *b + low, *b + high, *a + high],
                normal,
                frame,
            );
        }
    }
}

// Intersect an azimuth from the infield with a surveyed stadium boundary.
fn rim_point(ring: &[Vec3], center: Vec3, dir: Vec3) -> Vec3 {
    let cross = |a: Vec3, b: Vec3| a.x * b.z - a.z * b.x;
    let mut far = 0.0f32;
    for (a, b) in ring.iter().zip(ring.iter().cycle().skip(1)) {
        let edge = *b - *a;
        let den = cross(dir, edge);
        if den.abs() < 0.0001 {
            continue;
        }
        let t = cross(*a - center, edge) / den;
        let u = cross(*a - center, dir) / den;
        if (0.0..=1.0).contains(&u) {
            far = far.max(t);
        }
    }
    center + dir * far
}

fn stadium_bowl(mesh: &mut Surface, rings: &[Vec<Vec3>], floor: f32, height: f32) {
    let mut center = rings[1].iter().copied().sum::<Vec3>() / rings[1].len() as f32;
    center.y = floor;
    let concrete = [0.63, 0.64, 0.61, 1.];
    let aisle = [0.47, 0.47, 0.43, 1.];
    let seats = [0.48, 0.060, 0.055, 1.];
    let shade = [0.24, 0.24, 0.22, 1.];
    let mut inner = rings[1].clone();
    for p in &mut inner {
        p.y = floor;
    }
    mesh.polygon(&[inner], 0., 0., GRASS, GRASS);
    const N: usize = 96;
    let mut rim = Vec::with_capacity(N);
    for i in 0..N {
        let (sin, cos) = (i as f32 * std::f32::consts::TAU / N as f32).sin_cos();
        let d = Vec3::new(cos, 0., sin);
        rim.push((
            rim_point(&rings[0], center, d),
            rim_point(&rings[1], center, d),
        ));
    }
    for i in 0..N {
        let (outer, inner) = rim[i];
        let (next_outer, next_inner) = rim[(i + 1) % N];
        let facing = (outer + next_outer) * 0.5 - center;
        let bottom = |mut p: Vec3| {
            p.y = terrain_from_xz(p.x, p.z) - 0.05;
            p
        };
        let at = |p: Vec3, h: f32| Vec3::new(p.x, floor + h, p.z);
        face(
            mesh,
            [
                bottom(outer),
                bottom(next_outer),
                at(next_outer, height),
                at(outer, height),
            ],
            facing,
            concrete,
        );
        // Rows of open concrete bays under the rim follow the public exterior photographs.
        for (low, high) in [(0.32, 0.48), (0.65, 0.80)] {
            let p = outer.lerp(next_outer, 0.15);
            let q = outer.lerp(next_outer, 0.86);
            let lift = facing.normalize_or_zero() * 0.055;
            face(
                mesh,
                [
                    at(p, height * low) + lift,
                    at(q, height * low) + lift,
                    at(q, height * high) + lift,
                    at(p, height * high) + lift,
                ],
                facing,
                shade,
            );
        }
        let row = |a: Vec3, b: Vec3, t: f32| at(a.lerp(b, t), height * (0.07 + 0.90 * t));
        for tier in 0..12 {
            let low = tier as f32 / 12.;
            let high = (tier + 1) as f32 / 12.;
            let color = if tier == 5 || i % 8 == 0 {
                aisle
            } else {
                seats
            };
            face(
                mesh,
                [
                    row(inner, outer, low),
                    row(next_inner, next_outer, low),
                    row(next_inner, next_outer, high),
                    row(inner, outer, high),
                ],
                Vec3::Y,
                color,
            );
        }
        // The covered western stand is visible in the MOP archive plan and aerial photo.
        if facing.x < 0. && facing.z.abs() < -facing.x * 0.76 {
            let roof = [0.76, 0.76, 0.71, 1.];
            face(
                mesh,
                [
                    at(outer, height),
                    at(next_outer, height),
                    at(next_inner.lerp(next_outer, 0.18), height * 0.97),
                    at(inner.lerp(outer, 0.18), height * 0.97),
                ],
                Vec3::Y,
                roof,
            );
        }
    }
}

fn field_line(mesh: &mut Surface, a: Vec3, b: Vec3, width: f32, color: Color4) {
    let d = (b - a).normalize_or_zero();
    let side = Vec3::new(-d.z, 0., d.x) * width * 0.5;
    face(
        mesh,
        [a - side, b - side, b + side, a + side],
        Vec3::Y,
        color,
    );
}

fn field_markings(mesh: &mut Surface, ring: &[Vec3]) {
    let a = ring[0];
    let (long, short) = if a.distance(ring[1]) > a.distance(ring[3]) {
        (ring[1] - a, ring[3] - a)
    } else {
        (ring[3] - a, ring[1] - a)
    };
    let (length, width) = (long.length(), short.length());
    let (u, v) = (long.normalize(), short.normalize());
    let white = [0.88, 0.89, 0.80, 1.];
    let mut line = |x0: f32, z0: f32, x1: f32, z1: f32| {
        let o = a + Vec3::Y * 0.055;
        field_line(mesh, o + u * x0 + v * z0, o + u * x1 + v * z1, 0.32, white)
    };
    for x in [0., length * 0.5, length] {
        line(x, 0., x, width);
    }
    for z in [0., width] {
        line(0., z, length, z);
    }
    for (end, direction) in [(0., 1.), (length, -1.)] {
        for (depth, span) in [(16.5, 40.32), (5.5, 18.32)] {
            let (near, far) = ((width - span) * 0.5, (width + span) * 0.5);
            let edge = end + direction * depth;
            line(end, near, edge, near);
            line(edge, near, edge, far);
            line(edge, far, end, far);
        }
    }
    for i in 0..32 {
        let (s, c) = (i as f32 * std::f32::consts::TAU / 32.).sin_cos();
        let (sn, cn) = ((i + 1) as f32 * std::f32::consts::TAU / 32.).sin_cos();
        line(
            length * 0.5 + 9.15 * c,
            width * 0.5 + 9.15 * s,
            length * 0.5 + 9.15 * cn,
            width * 0.5 + 9.15 * sn,
        );
    }
}

fn floodlight(mesh: &mut Surface, p: Vec3, center: Vec3, height: f32) {
    let concrete = [0.77, 0.77, 0.72, 1.];
    let frame = [0.30, 0.13, 0.10, 1.];
    let bulb = [0.80, 0.82, 0.73, 1.];
    for n in 0..8 {
        let (s, c) = (n as f32 * std::f32::consts::TAU / 8.).sin_cos();
        let (s2, c2) = ((n + 1) as f32 * std::f32::consts::TAU / 8.).sin_cos();
        let (a, b) = (Vec3::new(c, 0., s), Vec3::new(c2, 0., s2));
        face(
            mesh,
            [
                p + a * 1.25,
                p + b * 1.25,
                p + b * 0.64 + Vec3::Y * (height - 5.4),
                p + a * 0.64 + Vec3::Y * (height - 5.4),
            ],
            a + b,
            concrete,
        );
    }
    let d = Vec3::new(center.x - p.x, 0., center.z - p.z).normalize();
    let right = Vec3::new(-d.z, 0., d.x);
    let point = |x: f32, y: f32, depth: f32| p + right * x + Vec3::Y * (height + y) + d * depth;
    face(
        mesh,
        [
            point(-4.6, -6., 0.),
            point(4.6, -6., 0.),
            point(4.6, 0., 0.),
            point(-4.6, 0., 0.),
        ],
        d,
        frame,
    );
    for col in 0..6 {
        for row in 0..3 {
            let x = -3.75 + 1.5 * col as f32;
            let y = -4.8 + 1.65 * row as f32;
            face(
                mesh,
                [
                    point(x - 0.44, y - 0.36, 0.12),
                    point(x + 0.44, y - 0.36, 0.12),
                    point(x + 0.44, y + 0.36, 0.12),
                    point(x - 0.44, y + 0.36, 0.12),
                ],
                d,
                bulb,
            );
        }
    }
}

fn estadio_nacional(tiles: &mut GroundTiles, path: &Path, scale: f32) {
    let fc = read_geojson(path);
    let parts = fc["features"].as_array().unwrap();
    let shell = parts
        .iter()
        .find(|f| f["properties"]["kind"] == "stadium_shell")
        .unwrap();
    let rings = open_rings(&shell["geometry"]["coordinates"]);
    let center = rings[1].iter().copied().sum::<Vec3>() / rings[1].len() as f32;
    // The 90 m DSM includes the stadium structure. Keep the surveyed infield level.
    let floor = rings[1].iter().map(|p| p.y).fold(f32::MIN, f32::max) + 0.30;
    let mesh = tile_for(tiles, center);
    stadium_bowl(
        mesh,
        &rings,
        floor,
        shell["properties"]["height_m"].as_f64().unwrap() as f32 * scale,
    );
    for part in parts {
        let p = &part["properties"];
        let coords = &part["geometry"]["coordinates"];
        match p["kind"].as_str().unwrap() {
            "floodlight" => {
                let spot = world(coords[0].as_f64().unwrap(), coords[1].as_f64().unwrap());
                floodlight(mesh, spot, center, p["height_m"].as_f64().unwrap() as f32);
            }
            kind @ ("track" | "pitch") => {
                let mut rings = open_rings(coords);
                for ring in &mut rings {
                    for point in ring {
                        point.y = floor + if kind == "pitch" { 0.10 } else { 0.05 };
                    }
                }
                let color = if kind == "pitch" {
                    [0.15, 0.38, 0.15, 1.]
                } else {
                    [0.46, 0.11, 0.074, 1.]
                };
                mesh.polygon(&rings, 0., 0., color, color);
                if kind == "pitch" {
                    field_markings(mesh, &rings[0]);
                }
            }
            _ => (),
        }
    }
}

// These four tile-local identities match the named OSM footprints in SGBX and SGB1.
pub(super) const NAMED: [(usize, usize); 4] = [(232, 2677), (209, 71), (189, 0), (191, 1)];
pub(super) fn named_buildings(root: &Path) -> Vec<((usize, usize), f32, Mesh)> {
    let fc = read_geojson(&root.join("web/data/santiago_landmarks.geojson"));
    let mut objects = Vec::new();
    for feature in fc["features"].as_array().unwrap() {
        let p = &feature["properties"];
        let Some(kind) = p["kind"].as_str() else {
            continue;
        };
        let index = match kind {
            "tower" => 0,
            "palace" => 1,
            "arena" => 2,
            "stadium" => 3,
            _ => continue,
        };
        let rings = open_rings(&feature["geometry"]["coordinates"]);
        let mut shape = Surface::default();
        match kind {
            "tower" => gran_torre(
                &mut shape,
                &rings,
                p["height_m"].as_f64().unwrap() as f32,
                p["levels"].as_f64().unwrap() as usize,
            ),
            "arena" => arena(&mut shape, &rings, p["height_m"].as_f64().unwrap() as f32),
            "palace" => shape.polygon(
                &rings,
                0.,
                p["levels"].as_f64().unwrap() as f32 * 3.5,
                STONE,
                ROOF,
            ),
            _ => {
                let mut tiles = GroundTiles::new();
                estadio_nacional(
                    &mut tiles,
                    &root.join("web/data/estadio_nacional_parts.geojson"),
                    1.0,
                );
                shape = tiles.into_values().next().unwrap();
            }
        }
        let base = rings[0][0].y;
        for pos in &mut shape.positions {
            pos[1] -= base;
        }
        objects.push((NAMED[index], base, shape.finish()));
    }
    objects
}
