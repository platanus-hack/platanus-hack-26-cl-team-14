//! Suppress ordinary car poses on the retired pieces of an accepted road edit.
use super::*;
use serde::Deserialize;
use std::{collections::BTreeMap, fs::File, io::BufReader};

#[derive(Clone, Copy, PartialEq)]
struct Segment(Vec2, Vec2);

impl Segment {
    fn projection(self, p: Vec2, heading: Vec2) -> Option<Vec2> {
        let d = self.1 - self.0;
        let length = d.length();
        if length == 0. {
            return None;
        }
        let axis = d / length;
        // A crossing road is a different movement. The absolute dot accepts both directions.
        if axis.dot(heading).abs() < std::f32::consts::FRAC_1_SQRT_2 {
            return None;
        }
        Some(self.0 + axis * (p - self.0).dot(axis).clamp(0., length))
    }

    fn cells(self, margin: f32) -> impl Iterator<Item = IVec2> {
        let lo = tile(self.0.min(self.1) - margin);
        let hi = tile(self.0.max(self.1) + margin);
        (lo.x..=hi.x).flat_map(move |x| (lo.y..=hi.y).map(move |y| IVec2::new(x, y)))
    }
}

fn tile(p: Vec2) -> IVec2 {
    (p / 100.).floor().as_ivec2()
}

fn segments(geometry: &Value) -> Vec<Segment> {
    let coords = &geometry["coordinates"];
    let parts = match geometry["type"].as_str() {
        Some("LineString") => std::slice::from_ref(coords),
        Some("MultiLineString") => coords.as_array().unwrap().as_slice(),
        _ => return Vec::new(),
    };
    parts
        .iter()
        .flat_map(|part| {
            part.as_array()
                .unwrap()
                .windows(2)
                .map(|pair| Segment(point(&pair[0]).xz(), point(&pair[1]).xz()))
        })
        .collect()
}

#[derive(Deserialize)]
struct Roads {
    features: Vec<Way>,
}
#[derive(Deserialize)]
struct Way {
    properties: WayProperties,
    geometry: Line,
}
#[derive(Deserialize)]
struct WayProperties {
    osm_way_id: u64,
    width_m: f32,
}
#[derive(Deserialize)]
struct Line {
    coordinates: Vec<[f64; 2]>,
}

// Baseline and property-only scenarios never open the extra centerlines file.
#[derive(Default)]
pub(super) struct Catalog(Option<Roads>);

#[derive(Clone, Copy, PartialEq)]
struct Lane {
    line: Segment,
    until: f32,
    changed: Option<usize>,
}

#[derive(Clone, Default, PartialEq)]
pub(super) struct TrafficEdits {
    cells: HashMap<IVec2, Vec<Lane>>,
    before: Vec<Vec<Segment>>,
    retained: Vec<Segment>,
}

impl TrafficEdits {
    pub(super) fn crosses_retired(&self, from: Vec3, to: Vec3) -> bool {
        let segment = Segment(from.xz(), to.xz());
        if !segment.cells(0.).any(|cell| self.cells.contains_key(&cell)) {
            return false;
        }
        let direction = (to - from).normalize_or_zero();
        if [from, to, (from + to) * 0.5]
            .into_iter()
            .any(|p| self.hides(p, direction))
        {
            return true;
        }
        // Split at geometry boundaries so a short closed interval is sampled.
        let mut boundaries = vec![0., from.distance(to)];
        for line in self.before.iter().flatten().chain(&self.retained) {
            for boundary in [line.0, line.1] {
                if let Some(p) = segment.projection(boundary, direction.xz()) {
                    boundaries.push((p - from.xz()).dot(direction.xz()));
                }
            }
        }
        boundaries.sort_by(f32::total_cmp);
        boundaries
            .windows(2)
            .any(|pair| self.hides(from + direction * ((pair[0] + pair[1]) * 0.5), direction))
    }

    pub(super) fn from_result(result: &Value, catalog: &mut Catalog) -> Self {
        let mut state = BTreeMap::new();
        for edit in result["city_edits"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|e| e["kind"] == "road")
        {
            let id = edit["id"].as_str().unwrap();
            let entry = state
                .entry(id)
                .or_insert((&edit["before"]["geometry"], &Value::Null));
            entry.1 = &edit["after"]["geometry"];
        }
        let mut cut = Self::default();
        let mut by_id = HashMap::new();
        for (id, (before, after)) in state {
            if before == after {
                continue;
            }
            cut.retained.extend(segments(after));
            if before.is_null() {
                continue;
            }
            let lines = segments(before);
            for line in &lines {
                // The comuna paths simplify geometry at 20 m. Limit the catalog search
                // to cells around the edited geometry; use mapped widths for the actual match.
                for cell in line.cells(30.) {
                    cut.cells.entry(cell).or_default();
                }
            }
            if let Ok(osm_id) = id.parse::<u64>() {
                by_id.insert(osm_id, cut.before.len());
            }
            cut.before.push(lines);
        }
        if cut.cells.is_empty() {
            return cut;
        }
        let roads = catalog.0.get_or_insert_with(|| {
            let file = File::open("web/data/road_centerlines.geojson.gz").unwrap();
            serde_json::from_reader(BufReader::new(flate2::read::GzDecoder::new(file))).unwrap()
        });
        let mut mapped = vec![false; cut.before.len()];
        for way in &roads.features {
            let changed = by_id.get(&way.properties.osm_way_id).copied();
            if let Some(index) = changed {
                mapped[index] = true;
            }
            let until = way.properties.width_m * 0.5 + 3.8;
            for pair in way.geometry.coordinates.windows(2) {
                let [a, b] = [pair[0], pair[1]].map(|[x, y]| world(x, y).xz());
                let line = Segment(a, b);
                for cell in line.cells(until) {
                    if let Some(lanes) = cut.cells.get_mut(&cell) {
                        lanes.push(Lane {
                            line,
                            until,
                            changed,
                        });
                    }
                }
            }
        }
        // IDs outside the native centerline export still have engine-supplied drawing geometry.
        for (index, lines) in cut.before.iter().enumerate().filter(|(i, _)| !mapped[*i]) {
            for &line in lines {
                for cell in line.cells(3.8) {
                    if let Some(lanes) = cut.cells.get_mut(&cell) {
                        lanes.push(Lane {
                            line,
                            until: 3.8,
                            changed: Some(index),
                        });
                    }
                }
            }
        }
        cut
    }

    pub(super) fn hides(&self, position: Vec3, direction: Vec3) -> bool {
        let p = position.xz();
        let Some(lanes) = self.cells.get(&tile(p)) else {
            return false;
        };
        let heading = direction.xz().normalize_or_zero();
        let (mut closed, mut open) = (f32::INFINITY, f32::INFINITY);
        for lane in lanes {
            let Some(projection) = lane.line.projection(p, heading) else {
                continue;
            };
            let distance = p.distance(projection);
            if distance > lane.until {
                continue;
            }
            let retired = lane.changed.is_some_and(|index| {
                // Engine edits clip the drawn chord. Project onto that chord first, so
                // the more detailed OSM centerline can keep its actual curves on both sides.
                let nearest = self.before[index]
                    .iter()
                    .filter_map(|line| line.projection(projection, heading))
                    .min_by(|a, b| {
                        a.distance_squared(projection)
                            .total_cmp(&b.distance_squared(projection))
                    });
                nearest.is_some_and(|old| {
                    !self.retained.iter().any(|line| {
                        line.projection(old, heading)
                            .is_some_and(|point| point.distance_squared(old) < 0.25)
                    })
                })
            });
            if retired {
                closed = closed.min(distance);
            } else {
                open = open.min(distance);
            }
        }
        // Packed trip geometry contains no traffic-way ID. Keep a pose when two
        // aligned carriageways match within the rounding precision of that geometry.
        closed + 0.8 < open
    }
}
