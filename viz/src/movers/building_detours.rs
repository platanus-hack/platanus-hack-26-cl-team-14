//! Local display detours use connected, directed OSM streets outside placed buildings.
use super::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BinaryHeap, HashSet};

#[derive(Deserialize)]
struct Pack {
    nodes: BTreeMap<String, [f64; 2]>,
    ways: BTreeMap<String, Way>,
}
#[derive(Deserialize)]
struct Way {
    edges: Vec<ArcEdge>,
    structure: String,
}
#[derive(Deserialize)]
struct ArcEdge {
    from: String,
    to: String,
    length_m: f32,
}
#[derive(Clone, Copy)]
struct Edge {
    to: usize,
    metres: f32,
    crossing: Option<road_water::Crossing>,
}
struct Graph {
    nodes: Vec<Vec3>,
    cells: HashMap<(i32, i32), Vec<(usize, usize)>>,
    edges: Vec<Vec<Edge>>,
}

impl Graph {
    fn load() -> Self {
        let pack: Pack =
            serde_json::from_reader(std::io::BufReader::new(flate2::read::GzDecoder::new(
                fs::File::open("web/data/local_car_graph.json.gz").unwrap(),
            )))
            .unwrap();
        let mut indices = HashMap::new();
        let mut nodes = Vec::new();
        let mut cells = HashMap::<_, Vec<_>>::new();
        for (id, [lon, lat]) in pack.nodes {
            let index = nodes.len();
            let p = world(lon, lat);
            indices.insert(id, index);
            nodes.push(p);
        }
        let mut edges = vec![Vec::new(); nodes.len()];
        for way in pack.ways.into_values() {
            let crossing = match way.structure.as_str() {
                "bridge" => Some(road_water::Crossing::Bridge),
                "tunnel" => Some(road_water::Crossing::Tunnel),
                _ => None,
            };
            for edge in way.edges {
                edges[indices[&edge.from]].push(Edge {
                    to: indices[&edge.to],
                    metres: edge.length_m,
                    crossing,
                });
            }
        }
        for (from, arcs) in edges.iter().enumerate() {
            for (index, edge) in arcs.iter().enumerate() {
                let (a, b) = (cell(nodes[from]), cell(nodes[edge.to]));
                for x in a.0.min(b.0)..=a.0.max(b.0) {
                    for z in a.1.min(b.1)..=a.1.max(b.1) {
                        cells.entry((x, z)).or_default().push((from, index));
                    }
                }
            }
        }
        Self {
            nodes,
            cells,
            edges,
        }
    }

    // Packed routes round coordinates to five decimals and can end inside an edge.
    fn anchors(
        &self,
        p: Vec3,
        direction: Vec3,
        crossing: Option<road_water::Crossing>,
        terminal: bool,
    ) -> Vec<Anchor> {
        let (x, z) = cell(p);
        let mut anchors = Vec::new();
        for i in x - 1..=x + 1 {
            for j in z - 1..=z + 1 {
                for &(from, index) in self.cells.get(&(i, j)).into_iter().flatten() {
                    let edge = self.edges[from][index];
                    let a = self.nodes[from];
                    let delta = self.nodes[edge.to] - a;
                    let alignment = delta.normalize().dot(direction);
                    if edge.crossing != crossing
                        || (if terminal { alignment.abs() } else { alignment }) < 0.7
                    {
                        continue;
                    }
                    let t = ((p - a).dot(delta) / delta.length_squared()).clamp(0., 1.);
                    if (a + delta * t).xz().distance_squared(p.xz()) <= 1.5_f32.powi(2) {
                        anchors.push(Anchor {
                            from,
                            edge,
                            fraction: t,
                        });
                    }
                }
            }
        }
        anchors.sort_by_key(|a| (a.from, a.edge.to));
        anchors.dedup_by_key(|a| (a.from, a.edge.to));
        anchors
    }
}

#[derive(Clone, Copy)]
struct Anchor {
    from: usize,
    edge: Edge,
    fraction: f32,
}

// BinaryHeap is a max heap. Reverse the nonnegative distance to get Dijkstra order.
#[derive(PartialEq)]
struct Visit(f32, usize);
impl Eq for Visit {}
impl PartialOrd for Visit {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}
impl Ord for Visit {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        rhs.0.total_cmp(&self.0).then_with(|| self.1.cmp(&rhs.1))
    }
}

#[derive(Default, Resource)]
pub(super) struct CarDetours {
    graph: Option<Graph>,
    open: Vec<Vec<Edge>>,
    retired: HashSet<(usize, usize)>,
    mask: StreetMask,
    // Retain the source Arc so its address cannot be reused during this result.
    paths: HashMap<usize, (Arc<Path>, Option<Arc<Path>>)>,
}

impl CarDetours {
    pub(super) fn reset(&mut self, mask: &StreetMask, roads: &road_edits::TrafficEdits) {
        self.paths.clear();
        self.mask = mask.clone();
        self.open.clear();
        self.retired.clear();
        if mask.is_empty() {
            return;
        }
        let graph = self.graph.get_or_insert_with(Graph::load);
        for (from, edges) in graph.edges.iter().enumerate() {
            for edge in edges {
                if roads.crosses_retired(graph.nodes[from], graph.nodes[edge.to]) {
                    self.retired.insert((from, edge.to));
                }
            }
        }
        self.open = graph
            .edges
            .iter()
            .enumerate()
            .map(|(from, edges)| {
                edges
                    .iter()
                    .copied()
                    .filter(|edge| {
                        let (a, b) = (graph.nodes[from], graph.nodes[edge.to]);
                        let separated = edge.crossing.is_some();
                        (separated || !mask.blocks_segment(a, b, 4.))
                            && !self.retired.contains(&(from, edge.to))
                    })
                    .collect()
            })
            .collect();
    }

    pub(super) fn path(&mut self, original: Arc<Path>) -> Option<Arc<Path>> {
        if self.mask.is_empty()
            || original.legs.len() != 1
            || original.legs[0].vehicle != Vehicle::Car
        {
            return Some(original);
        }
        let key = Arc::as_ptr(&original) as usize;
        if let Some((_, changed)) = self.paths.get(&key) {
            return changed.clone();
        }
        let changed = match self.blocked_span(&original) {
            Some((first, last)) => self.detour(&original, first, last),
            None => Some(original.clone()),
        };
        self.paths.insert(key, (original, changed.clone()));
        changed
    }

    fn blocked_span(&self, path: &Path) -> Option<(usize, usize)> {
        let conflicts: Vec<_> = path
            .points
            .windows(2)
            .enumerate()
            .filter_map(|(i, pair)| {
                let metres = (path.metres[i] + path.metres[i + 1]) * 0.5;
                let separated = path.crossings.iter().any(|&(a, b, kind)| {
                    matches!(
                        kind,
                        road_water::Crossing::Bridge | road_water::Crossing::Tunnel
                    ) && (a..b).contains(&metres)
                });
                (!separated && self.mask.blocks_segment(pair[0], pair[1], 4.)).then_some(i)
            })
            .collect();
        Some((*conflicts.first()?, *conflicts.last()?))
    }

    fn detour(&self, path: &Path, first: usize, last: usize) -> Option<Arc<Path>> {
        let graph = self.graph.as_ref().unwrap();
        let anchor = |i: usize, source: bool| {
            let p = path.points[i];
            if self.mask.blocks_segment(p, p + Vec3::X * 0.01, 4.) {
                return None;
            }
            let (a, b) = if i + 1 < path.points.len() {
                (i, i + 1)
            } else {
                (i - 1, i)
            };
            let direction = (path.points[b] - path.points[a]).normalize();
            let m = (path.metres[a] + path.metres[b]) * 0.5;
            let crossing = path
                .crossings
                .iter()
                .find(|&&(a, b, _)| (a..b).contains(&m))
                .map(|&(_, _, k)| k);
            let anchors: Vec<_> = graph
                .anchors(p, direction, crossing, i == 0 || i + 1 == path.points.len())
                .into_iter()
                .filter(|a| {
                    let join = if source {
                        graph.nodes[a.edge.to]
                    } else {
                        graph.nodes[a.from]
                    };
                    !self.retired.contains(&(a.from, a.edge.to))
                        && (a.edge.crossing.is_some() || !self.mask.blocks_segment(p, join, 4.))
                })
                .collect();
            (!anchors.is_empty()).then_some((i, anchors))
        };
        let starts: Vec<_> = (0..=first)
            .filter_map(|i| anchor(i, true))
            .flat_map(|(i, arcs)| arcs.into_iter().map(move |arc| (i, arc)))
            .collect();
        let goals: Vec<_> = (last + 1..path.points.len())
            .filter_map(|i| anchor(i, false))
            .flat_map(|(i, arcs)| arcs.into_iter().map(move |arc| (i, arc)))
            .collect();
        if starts.is_empty() || goals.is_empty() {
            return None;
        }
        let mut distance = vec![f32::INFINITY; graph.nodes.len()];
        let mut previous = vec![None; graph.nodes.len()];
        let mut seeds = HashMap::new();
        let mut queue = BinaryHeap::new();
        for (index, a) in starts {
            let metres = path.metres[index] + a.edge.metres * (1. - a.fraction);
            if metres < distance[a.edge.to] {
                distance[a.edge.to] = metres;
                seeds.insert(a.edge.to, (index, a));
                queue.push(Visit(metres, a.edge.to));
            }
        }
        let mut reached = None;
        let mut best = f32::INFINITY;
        while let Some(Visit(metres, node)) = queue.pop() {
            if metres > distance[node] {
                continue;
            }
            if metres > best {
                break;
            }
            for &(index, goal) in goals.iter().filter(|(_, a)| a.from == node) {
                let total = metres + goal.edge.metres * goal.fraction + path.metres.last().unwrap()
                    - path.metres[index];
                if total < best {
                    best = total;
                    reached = Some((index, goal));
                }
            }
            for edge in &self.open[node] {
                let next = metres + edge.metres;
                if next < distance[edge.to] {
                    distance[edge.to] = next;
                    previous[edge.to] = Some((node, edge.crossing));
                    queue.push(Visit(next, edge.to));
                }
            }
        }
        let (end, goal) = reached?;
        let mut steps = Vec::new();
        let mut node = goal.from;
        while let Some((before, crossing)) = previous[node] {
            steps.push((graph.nodes[node], crossing));
            node = before;
        }
        let &(start, seed) = seeds.get(&node)?;
        steps.push((graph.nodes[node], seed.edge.crossing));
        steps.reverse();
        steps.push((path.points[end], goal.edge.crossing));
        let mut points = path.points[..=start].to_vec();
        let mut metres = path.metres[..=start].to_vec();
        let mut crossings = Vec::new();
        let left = path.metres[start];
        for &(p, crossing) in &steps {
            if points.last().unwrap().distance_squared(p) < 0.0001 {
                continue;
            }
            if crossing.is_none() && self.mask.blocks_segment(*points.last().unwrap(), p, 4.) {
                return None;
            }
            let before = *metres.last().unwrap();
            let after = before + points.last().unwrap().distance(p);
            points.push(p);
            metres.push(after);
            if let Some(kind) = crossing {
                crossings.push((before, after, kind));
            }
        }
        let right = *metres.last().unwrap();
        let old_right = path.metres[end];
        let delta = right - old_right;
        for &(a, b, kind) in &path.crossings {
            if a < left {
                crossings.push((a, b.min(left), kind));
            }
            if b > old_right {
                crossings.push((a.max(old_right) + delta, b + delta, kind));
            }
        }
        crossings.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut seconds = path.seconds[..=start].to_vec();
        for &m in &metres[start + 1..] {
            seconds.push(
                path.seconds[start]
                    + (path.seconds[end] - path.seconds[start]) * (m - left) / (right - left),
            );
        }
        points.extend_from_slice(&path.points[end + 1..]);
        metres.extend(path.metres[end + 1..].iter().map(|m| m + delta));
        seconds.extend_from_slice(&path.seconds[end + 1..]);
        let end = points.len() - 1;
        Some(Arc::new(Path {
            points,
            metres,
            seconds,
            crossings,
            origin_preview: path.origin_preview,
            footway_spans: Vec::new(),
            legs: vec![Leg {
                end,
                vehicle: Vehicle::Car,
                scheduled: None,
                walk_path: None,
            }],
        }))
    }
}
