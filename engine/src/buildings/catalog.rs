//! Read the same quantized footprint and block-key packs as the native city.
use geo::{Area, BooleanOps, LineString, Polygon};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

type Point = [f64; 2];
type Rings = Vec<Vec<Point>>;

fn gunzip(path: &Path) -> Option<Vec<u8>> {
    let out = Command::new("gzip").arg("-dc").arg(path).output().ok()?;
    out.status.success().then_some(out.stdout)
}

fn word(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(b[at..at + 2].try_into().unwrap())
}

fn uint(b: &[u8], at: usize) -> usize {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize
}

#[derive(Clone, Copy)]
struct Bounds<T> {
    min: [T; 2],
    max: [T; 2],
}

impl<T: PartialOrd + Copy> Bounds<T> {
    fn new(p: [T; 2]) -> Self {
        Self { min: p, max: p }
    }

    fn include(&mut self, p: [T; 2]) {
        for (i, value) in p.into_iter().enumerate() {
            if value < self.min[i] {
                self.min[i] = value;
            }
            if value > self.max[i] {
                self.max[i] = value;
            }
        }
    }
}

impl Bounds<f64> {
    fn contains(&self, p: Point) -> bool {
        (0..2).all(|i| self.min[i] <= p[i] && p[i] <= self.max[i])
    }
}

impl Bounds<u16> {
    fn contains_grid(&self, p: Point) -> bool {
        (0..2)
            .all(|i| f64::from(self.min[i]) - 1e-7 <= p[i] && p[i] <= f64::from(self.max[i]) + 1e-7)
    }
}

struct Tile {
    polygons: usize,
    rings: usize,
    vertices: usize,
    heights: usize,
    flags: usize,
    keys: usize,
    bounds: Bounds<u16>,
    footprints: Vec<Bounds<u16>>,
}

struct Mesh {
    bytes: Vec<u8>,
    keys: Vec<u8>,
    blocks_at: usize,
    origin: Point,
    tiles: Vec<Option<Tile>>,
    centers: HashMap<String, Point>,
}

impl Mesh {
    fn load(dir: &Path) -> Option<Self> {
        let web = dir.join("../../web/data");
        let bytes = gunzip(&web.join("city_building_polygons.bin.gz"))?;
        let keys = gunzip(&web.join("city_building_block_ids.bin.gz"))?;
        if bytes.get(..4)? != b"SGB1" || keys.get(..4)? != b"SGBI" || bytes[4..8] != keys[4..8] {
            return None;
        }
        let n = word(&bytes, 4) as usize * word(&bytes, 6) as usize;
        let origin =
            [8, 12].map(|p| i32::from_le_bytes(bytes[p..p + 4].try_into().unwrap()) as f64);
        let blocks_at = 16 + (n + 1) * 4;
        let first_key = blocks_at + uint(&keys, 8) * 10;
        let mut tiles = Vec::with_capacity(n);
        for t in 0..n {
            let pos = uint(&bytes, 28 + t * 4);
            if pos == uint(&bytes, 32 + t * 4) {
                tiles.push(None);
                continue;
            }
            let count = uint(&bytes, pos);
            let polygons = pos + 12;
            let rings = polygons + (count + 1) * 4;
            let vertices = rings + (uint(&bytes, pos + 4) + 1) * 4;
            let heights = vertices + uint(&bytes, pos + 8) * 4;
            let mut footprints = Vec::with_capacity(count);
            for i in 0..count {
                let ring = uint(&bytes, polygons + 4 * i);
                let start = vertices + 4 * uint(&bytes, rings + 4 * ring);
                let end = vertices + 4 * uint(&bytes, rings + 4 * (ring + 1));
                let mut bounds = Bounds::new([word(&bytes, start), word(&bytes, start + 2)]);
                for v in (start + 4..end).step_by(4) {
                    bounds.include([word(&bytes, v), word(&bytes, v + 2)]);
                }
                footprints.push(bounds);
            }
            let mut bounds = footprints.first()?.to_owned();
            for f in &footprints {
                bounds.include(f.min);
                bounds.include(f.max);
            }
            tiles.push(Some(Tile {
                polygons,
                rings,
                vertices,
                heights,
                flags: heights + 2 * count,
                keys: first_key + 2 * uint(&keys, 16 + t * 4),
                bounds,
                footprints,
            }));
        }
        let mut centers = HashMap::new();
        for filename in ["blocks.csv", "jobs.csv"] {
            let mut csv = csv::Reader::from_path(dir.join(filename)).ok()?;
            for record in csv.records() {
                let record = record.ok()?;
                centers
                    .entry(record[0].to_owned())
                    .or_insert([record[2].parse().ok()?, record[3].parse().ok()?]);
            }
        }
        Some(Self {
            bytes,
            keys,
            blocks_at,
            origin,
            tiles,
            centers,
        })
    }

    fn shape(&self, t: &Tile, index: usize) -> Rings {
        let start = uint(&self.bytes, t.polygons + 4 * index);
        let end = uint(&self.bytes, t.polygons + 4 * (index + 1));
        (start..end)
            .map(|r| {
                let start = uint(&self.bytes, t.rings + 4 * r);
                let end = uint(&self.bytes, t.rings + 4 * (r + 1));
                let mut ring = (start..end)
                    .map(|v| {
                        let p = t.vertices + v * 4;
                        [
                            (self.origin[0] + f64::from(word(&self.bytes, p))) / 100_000.,
                            (self.origin[1] + f64::from(word(&self.bytes, p + 2))) / 100_000.,
                        ]
                    })
                    .collect::<Vec<_>>();
                ring.push(ring[0]);
                ring
            })
            .collect()
    }

    fn at_point(&self, point: Point) -> Vec<String> {
        let grid = [
            point[0] * 100_000. - self.origin[0],
            point[1] * 100_000. - self.origin[1],
        ];
        let mut matches = Vec::new();
        for (t, tile) in self.tiles.iter().enumerate() {
            let Some(tile) = tile else { continue };
            if !tile.bounds.contains_grid(grid) {
                continue;
            }
            for (i, b) in tile.footprints.iter().enumerate() {
                if b.contains_grid(grid) && contains(&self.shape(tile, i), point) {
                    matches.push(format!("mesh:{t}:{i}"));
                }
            }
        }
        matches
    }

    fn matching_outline(&self, outline: &Rings) -> Option<String> {
        let outer = bounds(outline.first()?)?;
        let grid = |point: Point| [point[0] * 100_000. - self.origin[0], point[1] * 100_000. - self.origin[1]];
        let [min, max] = [grid(outer.min), grid(outer.max)];
        let intersects = |b: &Bounds<u16>| {
            (0..2).all(|i| f64::from(b.min[i]) <= max[i] && f64::from(b.max[i]) >= min[i])
        };
        let polygon = |rings: &Rings| {
            let line = |ring: &Vec<Point>| LineString::from(
                ring.iter().map(|p| (p[0] - outer.min[0], p[1] - outer.min[1])).collect::<Vec<_>>()
            );
            Polygon::new(line(&rings[0]), rings[1..].iter().map(line).collect())
        };
        let source = polygon(outline);
        let area = source.unsigned_area();
        let mut best = None;
        let mut score = 0.5;
        for (t, tile) in self.tiles.iter().enumerate() {
            let Some(tile) = tile.as_ref().filter(|tile| intersects(&tile.bounds)) else { continue };
            for (i, b) in tile.footprints.iter().enumerate() {
                if !intersects(b) || self.bytes[tile.flags + i] & 4 != 0 { continue; }
                let candidate = polygon(&self.shape(tile, i));
                let intersection = source.intersection(&candidate).unsigned_area();
                let overlap = intersection / (area + candidate.unsigned_area() - intersection);
                if overlap > score {
                    best = Some(format!("mesh:{t}:{i}"));
                    score = overlap;
                }
            }
        }
        best
    }

    fn feature(&self, id: &str) -> Option<Value> {
        let (t, i) = id.strip_prefix("mesh:")?.split_once(':')?;
        let (t, i): (usize, usize) = (t.parse().ok()?, i.parse().ok()?);
        if format!("mesh:{t}:{i}") != id {
            return None;
        }
        let tile = self.tiles.get(t)?.as_ref()?;
        tile.footprints.get(i)?;
        let key = word(&self.keys, tile.keys + 2 * i);
        let (block, cut) = if key == u16::MAX {
            (None, None)
        } else {
            let pos = self.blocks_at + key as usize * 10;
            (
                Some(u64::from_le_bytes(self.keys[pos..pos + 8].try_into().unwrap()).to_string()),
                Some(word(&self.keys, pos + 8).to_string()),
            )
        };
        let center = block.as_ref().and_then(|id| self.centers.get(id));
        let flags = self.bytes[tile.flags + i];
        let height_source = if flags & 32 != 0 {
            "venue_operator"
        } else if flags & 16 != 0 {
            "osm_building_part"
        } else if flags & 8 != 0 {
            "satellite_estimate"
        } else {
            [
                "osm_height",
                "osm_levels",
                "census_density_estimate",
                "default",
            ][(flags & 3) as usize]
        };
        Some(
            json!({"type":"Feature","id":id,"geometry":{"type":"Polygon","coordinates":self.shape(tile,i)},
            "properties":{"height_m":f64::from(word(&self.bytes,tile.heights+2*i)) / 10.0, "height_source":height_source,
              "source":if flags & 4 != 0 {"census_generated"} else {"osm"},
              "block_id":block,"cut":cut,"block_longitude":center.map(|p|p[0]),"block_latitude":center.map(|p|p[1])}}),
        )
    }
}

fn contains(rings: &[Vec<Point>], point: Point) -> bool {
    fn ring_contains(ring: &[Point], [x, y]: Point) -> bool {
        let mut inside = false;
        for edge in ring.windows(2) {
            let ([x1, y1], [x2, y2]) = (edge[0], edge[1]);
            if (x2 - x1) * (y - y1) == (y2 - y1) * (x - x1)
                && (x1.min(x2)..=x1.max(x2)).contains(&x)
                && (y1.min(y2)..=y1.max(y2)).contains(&y)
            {
                return true;
            }
            if (y1 > y) != (y2 > y) && x < x1 + (x2 - x1) * (y - y1) / (y2 - y1) {
                inside = !inside;
            }
        }
        inside
    }
    rings.first().is_some_and(|r| ring_contains(r, point))
        && !rings.iter().skip(1).any(|r| ring_contains(r, point))
}

#[derive(Deserialize)]
struct Block {
    block_id: String,
    cut: String,
    longitude: f64,
    latitude: f64,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "coordinates")]
enum Boundary {
    Polygon(Rings),
    MultiPolygon(Vec<Rings>),
}

#[derive(Deserialize)]
struct BlockFeature {
    properties: Block,
    geometry: Boundary,
}

#[derive(Deserialize)]
struct BlockCollection {
    features: Vec<BlockFeature>,
}

struct BlockShape {
    bounds: Bounds<f64>,
    polygons: Vec<Rings>,
    info: Block,
}

fn bounds(ring: &[Point]) -> Option<Bounds<f64>> {
    let mut b = Bounds::new(*ring.first()?);
    for &p in ring {
        b.include(p);
    }
    Some(b)
}

// Strictly inside one of the horizontal intervals of a ring, including concave rings.
fn interior(ring: &[Point]) -> Option<Point> {
    let min = ring.iter().map(|p| p[1]).min_by(f64::total_cmp)?;
    let next = ring
        .iter()
        .map(|p| p[1])
        .filter(|y| *y > min)
        .min_by(f64::total_cmp)?;
    let y = (min + next) * 0.5;
    let mut xs: Vec<_> = ring
        .windows(2)
        .filter_map(|e| {
            let ([a, b], [c, d]) = (e[0], e[1]);
            ((b > y) != (d > y)).then(|| a + (c - a) * (y - b) / (d - b))
        })
        .collect();
    xs.sort_by(f64::total_cmp);
    xs.chunks_exact(2)
        .find(|p| p[0] < p[1])
        .map(|p| [(p[0] + p[1]) * 0.5, y])
}

fn edges_inside(ring: &[Point], polygon: &Rings, want_inside: bool) -> bool {
    // Check every interval between boundary intersections, including co-linear overlaps.
    // Testing vertices alone misses a street or courtyard between two corners.
    for e in ring.windows(2) {
        let ([ax, ay], [bx, by]) = (e[0], e[1]);
        let (dx, dy) = (bx - ax, by - ay);
        let mut ts = vec![0., 1.];
        for edge in polygon.iter().flat_map(|r| r.windows(2)) {
            let ([cx, cy], [ex, ey]) = (edge[0], edge[1]);
            let (ux, uy) = (ex - cx, ey - cy);
            let det = dx * uy - dy * ux;
            if det != 0. {
                let t = ((cx - ax) * uy - (cy - ay) * ux) / det;
                let u = ((cx - ax) * dy - (cy - ay) * dx) / det;
                if (0.0..=1.).contains(&t) && (0.0..=1.).contains(&u) {
                    ts.push(t);
                }
            } else if dx * (cy - ay) == dy * (cx - ax) {
                for [x, y] in [edge[0], edge[1]] {
                    let t = if dx.abs() > dy.abs() {
                        (x - ax) / dx
                    } else {
                        (y - ay) / dy
                    };
                    if (0.0..=1.).contains(&t) {
                        ts.push(t);
                    }
                }
            }
        }
        ts.sort_by(f64::total_cmp);
        for ts in ts.windows(2).filter(|p| p[1] > p[0]) {
            let t = (ts[0] + ts[1]) * 0.5;
            let p = [ax + t * dx, ay + t * dy];
            if contains(polygon, p) != want_inside {
                // A coincident hole edge does not put positive footprint area inside the hole.
                if !polygon.iter().flat_map(|r| r.windows(2)).any(|e| {
                    let (a, b) = (e[0], e[1]);
                    (b[0] - a[0]) * (p[1] - a[1]) == (b[1] - a[1]) * (p[0] - a[0])
                        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
                        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
                }) {
                    return false;
                }
            }
        }
    }
    true
}

fn covered(building: &Rings, block: &Rings) -> bool {
    edges_inside(&building[0], block, true)
        && block.iter().skip(1).all(|hole| {
            // A courtyard can lie wholly inside the proposed footprint, with no
            // boundary crossing. A building courtyard can also exclude it.
            !interior(hole).is_some_and(|p| contains(building, p))
                && edges_inside(hole, building, false)
        })
}

impl BlockShape {
    fn load(dir: &Path) -> Option<Vec<Self>> {
        let bytes = gunzip(&dir.join("building_blocks.geojson.gz"))?;
        let collection: BlockCollection = serde_json::from_slice(&bytes).ok()?;
        collection
            .features
            .into_iter()
            .map(|f| {
                let polygons = match f.geometry {
                    Boundary::Polygon(r) => vec![r],
                    Boundary::MultiPolygon(r) => r,
                };
                let mut b = bounds(polygons.first()?.first()?)?;
                for poly in &polygons {
                    for &p in poly.first()? {
                        b.include(p);
                    }
                }
                Some(Self {
                    bounds: b,
                    polygons,
                    info: f.properties,
                })
            })
            .collect()
    }
}

pub(super) struct Catalog {
    dir: PathBuf,
    mesh: OnceLock<Option<Mesh>>,
    blocks: OnceLock<Option<Vec<BlockShape>>>,
}

impl Catalog {
    pub(super) fn load(dir: &Path) -> Self {
        Self {
            dir: dir.to_owned(),
            mesh: OnceLock::new(),
            blocks: OnceLock::new(),
        }
    }

    fn mesh(&self) -> Option<&Mesh> {
        self.mesh.get_or_init(|| Mesh::load(&self.dir)).as_ref()
    }

    pub(super) fn at_point(&self, p: Point) -> Vec<String> {
        self.mesh().map_or_else(Vec::new, |m| m.at_point(p))
    }

    pub(super) fn matching_outline(&self, outline: &Value) -> Option<Value> {
        let rings: Rings = serde_json::from_value(outline["coordinates"].clone()).ok()?;
        self.feature(&self.mesh()?.matching_outline(&rings)?)
    }

    pub(super) fn feature(&self, id: &str) -> Option<Value> {
        if !id.starts_with("mesh:") {
            return None;
        }
        let mut feature = self.mesh()?.feature(id)?;
        if let Some(id) = feature["properties"]["block_id"].as_str()
            && feature["properties"]["block_longitude"].is_null()
            && let Ok(blocks) = self.boundaries()
            && let Some(block) = blocks.iter().find(|b| b.info.block_id == id)
        {
            feature["properties"]["block_longitude"] = json!(block.info.longitude);
            feature["properties"]["block_latitude"] = json!(block.info.latitude);
        }
        Some(feature)
    }

    fn boundaries(&self) -> Result<&[BlockShape], String> {
        self.blocks.get_or_init(||BlockShape::load(&self.dir)).as_deref().ok_or_else(||
            "The INE block boundary pack is unavailable. Cannot assign this building to a block.".into())
    }

    pub(super) fn block(&self, geometry: &Value) -> Result<Value, String> {
        let rings: Rings = serde_json::from_value(geometry["coordinates"].clone())
            .map_err(|_| "Invalid building coordinates.")?;
        let bbox = rings
            .first()
            .and_then(|r| bounds(r))
            .ok_or("Empty building footprint.")?;
        let mut selected = None;
        for block in self.boundaries()? {
            if !block.bounds.contains(bbox.min) || !block.bounds.contains(bbox.max) {
                continue;
            }
            if block.polygons.iter().any(|p| covered(&rings, p)) {
                if selected.is_some() {
                    return Err("The footprint matches multiple census blocks.".into());
                }
                selected = Some(&block.info);
            }
        }
        let row = |b: &Block, share: f64| {
            json!({"block_id":b.block_id,"cut":b.cut,
            "longitude":b.longitude,"latitude":b.latitude,"share":share})
        };
        let (shares, coverage) = if let Some(b) = selected {
            (vec![row(b, 1.0)], 1.0)
        } else {
            // Use one local plane for the footprint and every intersection. Holes remain holes.
            let origin = rings[0][0];
            let polygon = |rings: &Rings| {
                let line = |ring: &Vec<Point>| -> LineString<f64> {
                    ring.iter()
                        .map(|p| (p[0] - origin[0], p[1] - origin[1]))
                        .collect()
                };
                Polygon::new(line(&rings[0]), rings[1..].iter().map(line).collect())
            };
            let footprint = polygon(&rings);
            let mut areas = Vec::new();
            for block in self.boundaries()? {
                if (0..2).any(|i| {
                    block.bounds.max[i] <= bbox.min[i] || block.bounds.min[i] >= bbox.max[i]
                }) {
                    continue;
                }
                let area: f64 = block
                    .polygons
                    .iter()
                    .map(|p| footprint.intersection(&polygon(p)).unsigned_area())
                    .sum();
                if area > 0.0 {
                    areas.push((&block.info, area));
                }
            }
            let total: f64 = areas.iter().map(|(_, area)| area).sum();
            if areas.is_empty() {
                return Err("The footprint has no positive area inside the block pack.".into());
            }
            (
                areas
                    .into_iter()
                    .map(|(b, area)| row(b, area / total))
                    .collect(),
                total / footprint.unsigned_area(),
            )
        };
        let primary = shares
            .iter()
            .max_by(|a, b| {
                a["share"]
                    .as_f64()
                    .unwrap()
                    .total_cmp(&b["share"].as_f64().unwrap())
            })
            .unwrap();
        Ok(json!({"block_id":primary["block_id"],"cut":primary["cut"],
            "block_longitude":primary["longitude"],"block_latitude":primary["latitude"],
            "block_shares":shares,"block_covered_fraction":coverage}))
    }
}
