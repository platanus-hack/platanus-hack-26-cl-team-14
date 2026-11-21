//! Weekday 08:00 impedance, with the same walk limits as tools/pack/travel_times.py.
use super::{Pattern, Stop};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap};

const TRANSFER_METRES: f64 = 300.0;
const ACCESS_METRES: f64 = 1_000.0;
const WALK_METRES_PER_MINUTE: f64 = 80.0;

type Edges = Vec<Vec<(usize, f64)>>;

fn metres(a: [f64; 2], b: [f64; 2]) -> f64 {
    let latitude = ((a[1] + b[1]) * 0.5).to_radians();
    ((a[0] - b[0]) * latitude.cos() * 111_320.0).hypot((a[1] - b[1]) * 110_574.0)
}

fn cell([lon, lat]: [f64; 2]) -> (i32, i32) {
    const CELL: f64 = TRANSFER_METRES / 111_320.0 * 2.0;
    ((lon / CELL).floor() as i32, (lat / CELL).floor() as i32)
}

pub(super) struct Routing {
    stops: BTreeMap<String, usize>,
    walk: Edges,
    access: Vec<Vec<(usize, f64)>>,
}

impl Routing {
    pub(super) fn new(stops: &BTreeMap<String, Stop>, communes: &[[f64; 2]]) -> Self {
        let ids: BTreeMap<_, _> = stops
            .keys()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();
        let coordinates: Vec<_> = stops.values().map(|stop| stop.coordinates).collect();
        let mut grid: HashMap<_, Vec<usize>> = HashMap::new();
        for (i, &point) in coordinates.iter().enumerate() {
            grid.entry(cell(point)).or_default().push(i);
        }
        let mut walk = vec![Vec::new(); stops.len()];
        for (i, &point) in coordinates.iter().enumerate() {
            let (x, y) = cell(point);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for &j in grid.get(&(x + dx, y + dy)).into_iter().flatten() {
                        if j > i {
                            let distance = metres(point, coordinates[j]);
                            if distance <= TRANSFER_METRES {
                                let minutes = distance / WALK_METRES_PER_MINUTE;
                                walk[i].push((j, minutes));
                                walk[j].push((i, minutes));
                            }
                        }
                    }
                }
            }
        }
        for (i, stop) in stops.values().enumerate() {
            if let Some(&parent) = stop.parent_id.as_ref().and_then(|id| ids.get(id)) {
                walk[i].push((parent, 0.0));
                walk[parent].push((i, 0.0));
            }
        }
        let access = communes
            .iter()
            .map(|&point| {
                coordinates
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &stop)| {
                        let distance = metres(point, stop);
                        (distance <= ACCESS_METRES)
                            .then_some((i, distance / WALK_METRES_PER_MINUTE))
                    })
                    .collect()
            })
            .collect();
        Self {
            stops: ids,
            walk,
            access,
        }
    }

    pub(super) fn minutes<'a>(
        &self,
        patterns: impl Iterator<Item = &'a Pattern>,
    ) -> Vec<Option<f64>> {
        let mut edges = self.walk.clone();
        for pattern in patterns {
            let Some(headway) = pattern.headway_minutes else {
                continue;
            };
            let first = edges.len();
            edges.resize_with(first + 2 * pattern.stop_ids.len(), Vec::new);
            for (i, id) in pattern.stop_ids.iter().enumerate() {
                let stop = self.stops[id];
                let arrival = first + 2 * i;
                let departure = arrival + 1;
                if pattern.pickup_type[i] == 0 {
                    edges[stop].push((departure, headway * 0.5));
                }
                if pattern.drop_off_type[i] == 0 {
                    edges[arrival].push((stop, 0.0));
                }
                edges[arrival].push((departure, pattern.dwell_minutes[i]));
                if let Some(&ride) = pattern.segment_minutes.get(i) {
                    edges[departure].push((arrival + 2, ride));
                }
            }
        }
        let mut matrix = Vec::with_capacity(self.access.len().pow(2));
        for (origin, sources) in self.access.iter().enumerate() {
            let distances = shortest(&edges, sources);
            for (destination, stops) in self.access.iter().enumerate() {
                let best = stops
                    .iter()
                    .map(|&(stop, egress)| distances[stop] + egress)
                    .fold(f64::INFINITY, f64::min);
                matrix.push((origin != destination && best.is_finite()).then_some(best));
            }
        }
        matrix
    }
}

#[derive(PartialEq)]
struct Label {
    minutes: f64,
    node: usize,
}

impl Eq for Label {}

impl Ord for Label {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .minutes
            .total_cmp(&self.minutes)
            .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for Label {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn shortest(edges: &Edges, sources: &[(usize, f64)]) -> Vec<f64> {
    let mut distance = vec![f64::INFINITY; edges.len()];
    let mut queue = BinaryHeap::new();
    for &(node, minutes) in sources {
        distance[node] = minutes;
        queue.push(Label { minutes, node });
    }
    while let Some(Label { minutes, node }) = queue.pop() {
        if minutes > distance[node] {
            continue;
        }
        for &(next, cost) in &edges[node] {
            let candidate = minutes + cost;
            if candidate < distance[next] {
                distance[next] = candidate;
                queue.push(Label {
                    minutes: candidate,
                    node: next,
                });
            }
        }
    }
    distance
}
