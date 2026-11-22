use super::{Approach, Path, Replay, STOP_DISTANCE, Site, ms};
use crate::movers::{CAR_LANE_OFFSET, Vehicle, right, road_water};
use bevy::prelude::*;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::BufReader,
    ops::RangeInclusive,
};

const MATCH_RADIUS: f32 = 5.;
const CELL: f32 = 40.;
const EDGE_PROBE: f32 = 3.;
const SHORT_LINK_LIMIT: f32 = EDGE_PROBE + 0.5;
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

struct DirectedEdge {
    nodes: [Node; 2],
    start: Vec2,
    end: Vec2,
    heading: Vec2,
    ground: bool,
}

pub(super) struct Topology {
    pub(super) nodes: Vec<(Node, Vec2)>,
    cells: HashMap<Cell, Vec<usize>>,
    edges: Vec<DirectedEdge>,
    edge_cells: HashMap<Cell, Vec<usize>>,
    short_links: HashMap<Node, [Vec<usize>; 2]>,
}

impl Topology {
    pub(super) fn load() -> Self {
        let file = File::open("web/data/local_car_graph.json.gz").unwrap();
        let pack: Pack =
            serde_json::from_reader(BufReader::new(flate2::read::GzDecoder::new(file))).unwrap();
        let positions: HashMap<Node, Vec2> = pack
            .nodes
            .into_iter()
            .filter_map(|(id, [lon, lat])| {
                Some((id.parse().ok()?, crate::movers::world(lon, lat).xz()))
            })
            .collect();
        let mut links: BTreeMap<Node, Vec<Node>> = BTreeMap::new();
        let mut edges = Vec::new();
        let mut short_links = HashMap::<_, [Vec<_>; 2]>::new();
        let mut edge_cells: HashMap<Cell, Vec<usize>> = HashMap::new();
        for way in pack.ways.into_values() {
            let ground = way.structure == "ground";
            for edge in way.edges {
                let (Ok(from), Ok(to)) = (edge.from.parse(), edge.to.parse()) else {
                    continue;
                };
                let (start, end) = (positions[&from], positions[&to]);
                let Some(heading) = (end - start).try_normalize() else {
                    continue;
                };
                if ground {
                    links.entry(from).or_default().push(to);
                    links.entry(to).or_default().push(from);
                    if start.distance_squared(end) <= SHORT_LINK_LIMIT * SHORT_LINK_LIMIT {
                        short_links.entry(from).or_default()[0].push(edges.len());
                        short_links.entry(to).or_default()[1].push(edges.len());
                    }
                }
                let (x0, z0) = cell(start.min(end), CELL);
                let (x1, z1) = cell(start.max(end), CELL);
                for x in x0..=x1 {
                    for z in z0..=z1 {
                        edge_cells.entry((x, z)).or_default().push(edges.len());
                    }
                }
                edges.push(DirectedEdge {
                    nodes: [from, to],
                    start,
                    end,
                    heading,
                    ground,
                });
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
            let p = positions[&node];
            cells.entry(cell(p, CELL)).or_default().push(nodes.len());
            nodes.push((node, p));
        }
        Self {
            nodes,
            cells,
            edges,
            edge_cells,
            short_links,
        }
    }

    fn route_edge(&self, point: Vec2, direction: Vec2) -> Option<&DirectedEdge> {
        let radius = Vec2::splat(MATCH_RADIUS);
        let (x0, z0) = cell(point - radius, CELL);
        let (x1, z1) = cell(point + radius, CELL);
        (x0..=x1)
            .flat_map(|x| (z0..=z1).map(move |z| (x, z)))
            .flat_map(|cell| self.edge_cells.get(&cell).into_iter().flatten())
            .filter_map(|&i| {
                let edge = &self.edges[i];
                let alignment = direction.dot(edge.heading);
                let fraction = project(point, edge.start, edge.end);
                let gap = point.distance_squared(edge.start.lerp(edge.end, fraction));
                (alignment >= 0.8 && gap <= MATCH_RADIUS * MATCH_RADIUS)
                    .then_some((edge, gap + 9. * (1. - alignment)))
            })
            .min_by(|(a, left), (b, right)| {
                left.total_cmp(right)
                    .then(a.ground.cmp(&b.ground))
                    .then(a.nodes.cmp(&b.nodes))
            })
            .map(|(edge, _)| edge)
    }

    fn connects(&self, path: &Path, distance: f32, node: Node) -> bool {
        // Sample both directed legs. Nearby decks can share the same map position.
        [(-EDGE_PROBE, 1), (EDGE_PROBE, 0)]
            .into_iter()
            .all(|(offset, side)| {
                let (probe, heading, _) = path.at(distance + offset, &path.metres).unwrap();
                let probe = probe.xz();
                let Some(edge) = self.route_edge(probe, heading.xz().normalize_or_zero()) else {
                    return false;
                };
                if !edge.ground {
                    return false;
                }
                let target = edge.nodes[side];
                if node == target {
                    return true;
                }
                let corridor = [0., offset / 2., offset]
                    .map(|offset| path.at(distance + offset, &path.metres).unwrap().0.xz());
                self.follows_short_links(node, target, side, corridor)
            })
    }

    fn follows_short_links(
        &self,
        node: Node,
        target: Node,
        side: usize,
        corridor: [Vec2; 3],
    ) -> bool {
        // A probe can pass a short OSM edge immediately before or after the node.
        let mut pending = vec![(node, SHORT_LINK_LIMIT)];
        let mut reached = HashMap::new();
        while let Some((node, length_left)) = pending.pop() {
            if node == target {
                return true;
            }
            if reached
                .get(&node)
                .is_some_and(|&length| length >= length_left)
            {
                continue;
            }
            reached.insert(node, length_left);
            for &index in self
                .short_links
                .get(&node)
                .into_iter()
                .flat_map(|links| &links[side])
            {
                let edge = &self.edges[index];
                let length = edge.start.distance(edge.end);
                let next = if side == 0 { edge.end } else { edge.start };
                let in_corridor = corridor.windows(2).any(|pair| {
                    let fraction = project(next, pair[0], pair[1]);
                    next.distance_squared(pair[0].lerp(pair[1], fraction)) <= 1.5_f32.powi(2)
                });
                if length <= length_left && in_corridor {
                    pending.push((edge.nodes[1 - side], length_left - length));
                }
            }
        }
        false
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
            let crossing = path.crossing_at(distance);
            if distance < 40.
                || total - distance < 25.
                || matches!(
                    crossing,
                    Some(road_water::Crossing::Bridge | road_water::Crossing::Tunnel)
                )
                || !self.connects(path, distance, self.nodes[junction].0)
            {
                continue;
            }
            let Some((p, dir, _)) = path.at(distance, &path.metres) else {
                continue;
            };
            if replay.road_edits.hides_on(p, dir, crossing) {
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
        let (p, dir) = path
            .road_pose(metres, Vehicle::Car, 0, path.points.len() - 1)
            .unwrap();
        (p + right(dir) * CAR_LANE_OFFSET).xz()
    };
    let entry = sample(first - 38.);
    let heading = (sample(first - 23.) - entry).normalize_or_zero();
    let stop = first - STOP_DISTANCE;
    let speed = Vehicle::Car.speed();
    let limit = last + 22.;
    let curve = (0..)
        .map(|i| stop + i as f32 * 3.)
        .take_while(|&metres| metres < limit)
        .chain([limit])
        .map(|metres| Sample {
            position: sample(metres),
            time: ms((metres - stop) / speed) as i32,
        })
        .collect();
    let movement = sites
        .entry(junction)
        .or_default()
        .register(entry, heading, curve);
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

#[derive(Clone, Copy)]
pub(super) struct Sample {
    pub(super) position: Vec2,
    pub(super) time: i32,
}

pub(super) type Window = RangeInclusive<u32>;

// The two outputs are the unsafe start offsets for a after b, and b after a.
pub(super) fn conflict_windows(a: &[Sample], b: &[Sample]) -> (Vec<Window>, Vec<Window>) {
    // Both cars' half lengths at the rendered scale, rounded up to one second of travel.
    const BODY_TIME_MS: i32 = 1_000;
    let mut a_after_b = Vec::new();
    let mut b_after_a = Vec::new();
    for a in a.windows(2) {
        for b in b.windows(2) {
            if !segments_conflict(a[0].position, a[1].position, b[0].position, b[1].position) {
                continue;
            }
            let start = b[0].time - a[1].time - BODY_TIME_MS;
            let end = b[1].time - a[0].time + BODY_TIME_MS;
            if end >= 0 {
                a_after_b.push(start.max(0) as u32..=end as u32);
            }
            if start <= 0 {
                b_after_a.push((-end).max(0) as u32..=(-start) as u32);
            }
        }
    }
    (merge_windows(a_after_b), merge_windows(b_after_a))
}

fn merge_windows(mut windows: Vec<Window>) -> Vec<Window> {
    windows.sort_unstable_by_key(|window| *window.start());
    let mut merged: Vec<Window> = Vec::new();
    for window in windows {
        if let Some(last) = merged.last_mut()
            && *window.start() <= *last.end() + 1
        {
            *last = *last.start()..=(*last.end()).max(*window.end());
        } else {
            merged.push(window);
        }
    }
    merged
}

fn segments_conflict(p: Vec2, q: Vec2, r: Vec2, s: Vec2) -> bool {
    let close = |p: Vec2, a: Vec2, b: Vec2| {
        p.distance_squared(a.lerp(b, project(p, a, b))) < 3.6_f32.powi(2)
    };
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
}
