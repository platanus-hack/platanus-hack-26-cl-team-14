use super::{Approach, CLEARANCE, Path, Replay, STOP_DISTANCE, Site, ms};
use crate::movers::{Vehicle, right, road_water};
use bevy::prelude::*;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::BufReader,
};

const MATCH_RADIUS: f32 = 5.;
const CELL: f32 = 40.;
type Cell = (i32, i32);
type Node = u64;

#[derive(Deserialize)]
struct Pack {
    nodes: HashMap<String, [f64; 2]>,
    ways: HashMap<String, Way>,
}

#[derive(Deserialize)]
struct Way {
    structure: String,
    edges: Vec<Edge>,
}

#[derive(Deserialize)]
struct Edge {
    from: String,
    to: String,
}

pub(super) struct Topology {
    pub(super) nodes: Vec<(Node, Vec2)>,
    cells: HashMap<Cell, Vec<usize>>,
}

impl Topology {
    pub(super) fn load() -> Self {
        let file = File::open("web/data/local_car_graph.json.gz").unwrap();
        let pack: Pack =
            serde_json::from_reader(BufReader::new(flate2::read::GzDecoder::new(file))).unwrap();
        let mut links: BTreeMap<Node, Vec<Node>> = BTreeMap::new();
        for way in pack.ways.values().filter(|way| way.structure == "ground") {
            for edge in &way.edges {
                let (Ok(from), Ok(to)) = (edge.from.parse(), edge.to.parse()) else {
                    continue;
                };
                links.entry(from).or_default().push(to);
                links.entry(to).or_default().push(from);
            }
        }
        let mut nodes = Vec::new();
        let mut cells: HashMap<Cell, Vec<usize>> = HashMap::new();
        for (node, mut neighbors) in links {
            neighbors.sort_unstable();
            neighbors.dedup();
            if neighbors.len() < 3 {
                continue;
            }
            let [lon, lat] = pack.nodes[&node.to_string()];
            let p = crate::movers::world(lon, lat).xz();
            cells.entry(cell(p, CELL)).or_default().push(nodes.len());
            nodes.push((node, p));
        }
        Self { nodes, cells }
    }

    pub(super) fn approaches(&self, path: &Path, replay: &Replay) -> Vec<(usize, f32)> {
        let total = *path.metres.last().unwrap();
        let mut hits: HashMap<usize, (f32, f32)> = HashMap::new();
        for (i, pair) in path.points.windows(2).enumerate() {
            let a = pair[0].xz();
            let b = pair[1].xz();
            let d = b - a;
            if d.length_squared() == 0. {
                continue;
            }
            let (x0, z0) = cell(a.min(b) - Vec2::splat(MATCH_RADIUS), CELL);
            let (x1, z1) = cell(a.max(b) + Vec2::splat(MATCH_RADIUS), CELL);
            for x in x0..=x1 {
                for z in z0..=z1 {
                    for &id in self.cells.get(&(x, z)).into_iter().flatten() {
                        let p = self.nodes[id].1;
                        let fraction = project(p, a, b);
                        let gap = p.distance_squared(a + d * fraction);
                        if gap > MATCH_RADIUS * MATCH_RADIUS {
                            continue;
                        }
                        let distance = path.metres[i].lerp(path.metres[i + 1], fraction);
                        let hit = hits.entry(id).or_insert((gap, distance));
                        if gap < hit.0 {
                            *hit = (gap, distance);
                        }
                    }
                }
            }
        }
        let mut hits: Vec<_> = hits.into_iter().collect();
        hits.sort_unstable_by(|a, b| a.1.1.total_cmp(&b.1.1).then(a.0.cmp(&b.0)));
        let mut approaches = Vec::new();
        for (junction, (_, distance)) in hits {
            // Require enough approach and exit to fit a car. Match only grade
            // crossings reached along the route, never a bridge or tunnel.
            if distance < 40.
                || total - distance < 25.
                || path.crossings.iter().any(|&(a, b, kind)| {
                    (a..=b).contains(&distance)
                        && matches!(
                            kind,
                            road_water::Crossing::Bridge | road_water::Crossing::Tunnel
                        )
                })
            {
                continue;
            }
            let Some((p, dir, _)) = path.at(distance, &path.metres) else {
                continue;
            };
            if replay.road_edits.hides(p, dir) {
                continue;
            }
            approaches.push((junction, distance));
        }
        approaches
    }
}

pub(super) fn register(
    path: &Path,
    junction: usize,
    first: f32,
    last: f32,
    sites: &mut HashMap<usize, Site>,
) -> Approach {
    let sample = |metres: f32| {
        let (p, dir, _) = path.at(metres, &path.metres).unwrap();
        (p + right(dir) * 1.6).xz()
    };
    let entry = sample(first - 38.);
    let heading = (sample(first - 23.) - entry).normalize_or_zero();
    let stop = first - STOP_DISTANCE;
    let mut curve: Vec<_> = (0..)
        .map(|i| stop + i as f32 * 7.)
        .take_while(|&metres| metres < last + 22.)
        .map(sample)
        .collect();
    curve.push(sample(last + 22.));
    let speed = Vehicle::Car.speed();
    let clearance = CLEARANCE + ms((last - first) / speed);
    let movement = sites
        .entry(junction)
        .or_default()
        .register(entry, heading, curve, clearance);
    Approach {
        junction,
        movement,
        stop_age: stop / speed,
    }
}

pub(super) fn cell(p: Vec2, size: f32) -> Cell {
    ((p.x / size).floor() as i32, (p.y / size).floor() as i32)
}

fn project(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let d = b - a;
    if d.length_squared() == 0. {
        return 0.;
    }
    ((p - a).dot(d) / d.length_squared()).clamp(0., 1.)
}

pub(super) fn conflict(a: &[Vec2], b: &[Vec2]) -> bool {
    let close = |p: Vec2, a: Vec2, b: Vec2| {
        p.distance_squared(a.lerp(b, project(p, a, b))) < 3.5_f32.powi(2)
    };
    a.windows(2).any(|a| {
        b.windows(2).any(|b| {
            let (p, q, r, s) = (a[0], a[1], b[0], b[1]);
            let ab = q - p;
            let cd = s - r;
            let cross = ab.perp_dot(cd);
            if cross.abs() > 0.001 {
                let alpha = (r - p).perp_dot(cd) / cross;
                let beta = (r - p).perp_dot(ab) / cross;
                if (0. ..=1.).contains(&alpha) && (0. ..=1.).contains(&beta) {
                    return true;
                }
            }
            close(p, r, s) || close(q, r, s) || close(r, p, q) || close(s, p, q)
        })
    })
}
