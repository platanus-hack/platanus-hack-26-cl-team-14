use super::terrain::{LandCover, terrain_from_xz};
use crate::city::{Surface, world};
use bevy::prelude::*;
use flate2::read::GzDecoder;
use serde_json::Value;
use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};

type Color4 = [f32; 4];
type GroundTiles = BTreeMap<(i32, i32), Surface>;

// Half-widths are cartographic line widths in metres.
// Ink on paper: blue-violet ink, light paper, strong blue water (Emerson, V2 thread).
const GRASS: Color4 = [0.24, 0.54, 0.29, 1.];
const WATER: Color4 = [0.025, 0.36, 0.77, 1.];
const STONE: Color4 = [0.584, 0.558, 0.791, 1.];
const ROOF: Color4 = [0.270, 0.238, 0.584, 1.];

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

// Clip each strip quad to the DEM faces so terrain cannot cut through its interior.
pub(super) fn ribbon(
    mesh: &mut Surface,
    points: &[Vec3],
    half_width: f32,
    elevation: f32,
    color: Color4,
) {
    ribbon_faces(points, half_width, |ring| {
        super::terrain::drape_land(mesh, &[ring], elevation, color)
    });
}

fn ribbon_faces(points: &[Vec3], half_width: f32, mut face: impl FnMut(Vec<Vec3>)) {
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
    let mut previous_edges: Option<[Vec3; 2]> = None;
    let last = points.len() - 1;
    for (i, &p) in points.iter().enumerate() {
        let previous = normal(points[i.saturating_sub(1)], p);
        let next = normal(p, points[(i + 1).min(last)]);
        let join = (previous + next).normalize_or_zero();
        let reference = if i == last { previous } else { next };
        let offset = join * (half_width / join.dot(reference).max(0.5));
        let edges = [p + offset, p - offset].map(|mut edge| {
            edge.y = terrain_from_xz(edge.x, edge.z);
            edge
        });
        if let Some(previous) = previous_edges {
            face(vec![previous[0], previous[1], edges[1], edges[0]]);
        }
        previous_edges = Some(edges);
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
            "track" => [0.474, 0.323, 0.275, 1.],
            "pitch" => [0.376, 0.610, 0.462, 1.],
            "surface_parking" => [0.434, 0.423, 0.631, 1.],
            "parking_unspecified" => [0.497, 0.485, 0.687, 1.],
            "brownfield" => [0.651, 0.624, 0.768, 1.],
            "greenfield" => [0.32, 0.61, 0.36, 1.],
            "wood" => [0.12, 0.34, 0.17, 1.],
            "scrub" | "heath" | "nature_reserve" => [0.19, 0.42, 0.24, 1.],
            "grass" | "meadow" | "grassland" => [0.22, 0.50, 0.27, 1.],
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
    let mut banks = Vec::new();
    let mut water = super::water::Map::default();
    for feature in fc["features"].as_array().unwrap() {
        let props = &feature["properties"];
        let shape = &feature["geometry"]["coordinates"];
        let kind = props["kind"].as_str().unwrap();
        if kind == "water" {
            for polygon in shape.as_array().unwrap() {
                let rings = open_rings(polygon);
                super::terrain::drape_land(tile_for(tiles, rings[0][0]), &rings, 0.08, WATER);
                water.add_polygon(&rings);
                banks.push(rings[0].clone());
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
                [0.18, 0.19, 0.20, 1.],
            );
        }
    }
    // Keep each outer river symbol up to its first actual meeting with a detailed bank.
    // Only exterior rings stop the symbol: the central islands keep their holes.
    let landmarks = read_geojson(&root.join("web/data/santiago_landmarks.geojson"));
    for feature in landmarks["features"].as_array().unwrap() {
        if feature["properties"]["kind"] != "river" {
            continue;
        }
        let mut points = coordinates(&feature["geometry"]["coordinates"]);
        water.add_channel(&points);
        for _ in 0..2 {
            let mut reach = vec![points[0]];
            for pair in points.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let start = Vec2::new(a.x, a.z);
                let step = Vec2::new(b.x - a.x, b.z - a.z);
                let mut meeting = f32::INFINITY;
                for bank in &banks {
                    for (c, d) in bank
                        .iter()
                        .zip(bank.iter().cycle().skip(1))
                        .take(bank.len())
                    {
                        let edge = Vec2::new(d.x - c.x, d.z - c.z);
                        let denominator = step.perp_dot(edge);
                        if denominator == 0.0 {
                            continue;
                        }
                        let offset = Vec2::new(c.x, c.z) - start;
                        let t = offset.perp_dot(edge) / denominator;
                        let u = offset.perp_dot(step) / denominator;
                        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                            meeting = meeting.min(t);
                        }
                    }
                }
                if meeting <= 1.0 {
                    reach.push(a.lerp(b, meeting));
                    break;
                }
                reach.push(b);
            }
            let tile = tile_for(tiles, reach[0]);
            ribbon_faces(&reach, 15., |ring| {
                let polygon = [ring];
                water.add_polygon(&polygon);
                super::terrain::drape_land(tile, &polygon, 0.08, WATER);
            });
            points.reverse();
        }
    }
    let route: Value = serde_json::from_str(include_str!(
        "../../../context/research/mapocho/water-route.geojson"
    ))
    .unwrap();
    let detailed: Vec<_> = route["features"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|leg| coordinates(&leg["geometry"]["coordinates"]))
        .collect();
    water.add_channel(&detailed);
    water.install();
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
    let scenery = tiles
        .into_iter()
        .map(|((x, z), surface)| (format!("City ground {x} {z}"), surface.finish()))
        .collect();
    (terrain.meshes(), scenery)
}

/// Camera-facing tree prints anchored to the packed OSM coordinates.
pub(super) fn mapped_trees(path: &Path) -> impl Iterator<Item = Mesh> {
    let mut tiles = GroundTiles::new();
    use std::io::Read;
    let mut bytes = Vec::new();
    GzDecoder::new(File::open(path).unwrap())
        .read_to_end(&mut bytes)
        .unwrap();
    for pair in bytes.chunks_exact(4) {
        let x = u16::from_le_bytes([pair[0], pair[1]]) as u32;
        let y = u16::from_le_bytes([pair[2], pair[3]]) as u32;
        let base = world(-70.85 + x as f64 / 100_000., -33.65 + y as f64 / 100_000.);
        let mesh = tile_for(&mut tiles, base);
        let seed = x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663);
        let height = 5.8 + (seed % 41) as f32 * 0.10;
        let radius = height * (0.31 + ((seed >> 8) % 8) as f32 * 0.012);
        let right = Vec3::new(0.51, 0., -0.32).normalize() * radius;
        let up = Vec3::Y * height;
        let first = mesh.positions.len() as u32;
        let u = (seed & 3) as f32 * 0.25;
        mesh.textured = true;
        for (position, uv) in [
            (base - right, [u, 1.]),
            (base + right, [u + 0.25, 1.]),
            (base + right + up, [u + 0.25, 0.]),
            (base - right + up, [u, 0.]),
        ] {
            mesh.positions.push(position.to_array());
            mesh.normals.push(Vec3::Y.to_array());
            mesh.colors.push([1.; 4]);
            mesh.uvs.push(uv);
        }
        mesh.indices.extend([0, 1, 2, 0, 2, 3].map(|i| first + i));
    }
    tiles.into_values().map(Surface::finish)
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

// `stands`: outline and height of each mapped stand; a rim segment takes the stand that contains it, else the nearest. None: one height and the Nacional roof.
// Even-odd test of a point against a ring in the ground plane.
fn inside(ring: &[Vec3], p: Vec2) -> bool {
    let mut odd = false;
    for (a, b) in ring
        .iter()
        .zip(ring.iter().cycle().skip(1))
        .take(ring.len())
    {
        if (a.z > p.y) != (b.z > p.y) && p.x < a.x + (p.y - a.z) * (b.x - a.x) / (b.z - a.z) {
            odd = !odd;
        }
    }
    odd
}

fn stadium_bowl(
    mesh: &mut Surface,
    rings: &[Vec<Vec3>],
    floor: f32,
    height: f32,
    stands: &[(Vec<Vec3>, f32)],
) -> Vec<bool> {
    let mut used = vec![false; stands.len()];
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
        let mid = ((outer + next_outer + inner + next_inner) * 0.25).xz();
        let centroid = |r: &[Vec3]| (r.iter().copied().sum::<Vec3>() / r.len() as f32).xz();
        let hit = stands.iter().position(|s| inside(&s.0, mid));
        if let Some(k) = hit {
            used[k] = true;
        }
        let height = hit
            .map(|k| &stands[k])
            .or_else(|| {
                stands.iter().min_by(|a, b| {
                    centroid(&a.0)
                        .distance_squared(mid)
                        .total_cmp(&centroid(&b.0).distance_squared(mid))
                })
            })
            .map_or(height, |s| s.1);
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
        if stands.is_empty() && facing.x < 0. && facing.z.abs() < -facing.x * 0.76 {
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
    used
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
    let stands: Vec<(Vec<Vec3>, f32)> = parts
        .iter()
        .filter(|f| f["properties"]["kind"] == "stand")
        .map(|f| {
            (
                open_rings(&f["geometry"]["coordinates"]).swap_remove(0),
                f["properties"]["height_m"].as_f64().unwrap() as f32 * scale,
            )
        })
        .collect();

    let used = stadium_bowl(
        mesh,
        &rings,
        floor,
        shell["properties"]["height_m"].as_f64().unwrap() as f32 * scale,
        &stands,
    );
    // A stand that no bowl sector reaches (Monumental's outer Rapa Nui tier) is drawn as its own block.
    for ((ring, h), _) in stands.iter().zip(&used).filter(|(_, u)| !**u) {
        mesh.polygon(
            &[ring.clone()],
            0.,
            *h,
            [0.63, 0.64, 0.61, 1.],
            [0.48, 0.060, 0.055, 1.],
        );
    }
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

// These tile-local identities match the named OSM footprints in SGBX and SGB1.
pub(super) const NAMED: [(usize, usize); 5] =
    [(232, 2677), (209, 71), (189, 0), (191, 1), (152, 317)];
// The other Estadio Monumental footprints (stands); its parts mesh replaces them.
pub(super) const HIDDEN: [(usize, usize); 10] = [
    (152, 316),
    (152, 1173),
    (152, 1174),
    (152, 1175),
    (152, 1176),
    (152, 1177),
    (152, 1178),
    (152, 1179),
    (152, 1180),
    (152, 1181),
];
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
    // Estadio Monumental, from tools/pack/pitch_sites.py; it replaces the footprints in NAMED[4] and HIDDEN.
    let path = root.join("web/data/estadio_monumental_parts.geojson");
    let fc = read_geojson(&path);
    let shell = fc["features"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["properties"]["kind"] == "stadium_shell")
        .unwrap();
    let base = open_rings(&shell["geometry"]["coordinates"])[0][0].y;
    let mut tiles = GroundTiles::new();
    estadio_nacional(&mut tiles, &path, 1.0);
    let mut shape = tiles.into_values().next().unwrap();
    for pos in &mut shape.positions {
        pos[1] -= base;
    }
    objects.push((NAMED[4], base, shape.finish()));
    objects
}
