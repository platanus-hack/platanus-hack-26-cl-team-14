//! Editable road geometry and directed, free-flow driving times between comuna anchors.
mod feature;
mod load;
mod routing;

use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Clone)]
struct ArcEdge {
    from: usize,
    to: usize,
    metres: f64,
    speed: f64,
    forward: bool,
    assumed: bool,
}

struct Road {
    feature: Value,
    edges: Vec<ArcEdge>,
}

pub struct RoadGraph {
    dir: PathBuf,
    // Requests with no road changes do not parse or route the road pack.
    baseline: OnceLock<Option<(load::Loaded, Vec<f64>)>>,
}

pub struct RoadEdits<'a> {
    graph: &'a RoadGraph,
    changed: BTreeMap<String, Option<Road>>,
    extra_nodes: Vec<[f64; 2]>,
    minutes: Option<Vec<f64>>,
}

impl RoadGraph {
    pub fn load(dir: &Path) -> Self {
        Self {
            dir: dir.into(),
            baseline: OnceLock::new(),
        }
    }

    pub fn edits(&self) -> RoadEdits<'_> {
        RoadEdits {
            graph: self,
            changed: BTreeMap::new(),
            extra_nodes: vec![],
            minutes: None,
        }
    }

    fn baseline(&self) -> Result<&(load::Loaded, Vec<f64>), String> {
        self.baseline
            .get_or_init(|| {
                if !self.dir.join("road_graph.json").is_file() {
                    return None;
                }
                let data = load::load(&self.dir);
                let minutes = route(data.nodes.len(), data.roads.values(), &data.anchors);
                Some((data, minutes))
            })
            .as_ref()
            .ok_or_else(|| "Road routing needs data/mvp/road_graph.json.".into())
    }
}

fn route<'a>(nodes: usize, roads: impl Iterator<Item = &'a Road>, anchors: &[usize]) -> Vec<f64> {
    let mut adjacency = vec![Vec::new(); nodes];
    for edge in roads.flat_map(|road| &road.edges) {
        adjacency[edge.from].push((edge.to, edge.metres * 3.6 / edge.speed));
    }
    routing::travel_minutes(&adjacency, anchors)
}

fn class_speed(class: &str) -> f64 {
    match class {
        "motorway" => 90.,
        "motorway_link" => 45.,
        "trunk" => 85.,
        "trunk_link" => 40.,
        "primary" => 65.,
        "primary_link" => 30.,
        "secondary" => 55.,
        "secondary_link" => 25.,
        _ => 30.,
    }
}

fn oneway(value: &Value) -> i8 {
    match value.as_str() {
        Some("-1") => -1,
        Some("yes" | "1" | "true") => 1,
        _ if value.as_bool() == Some(true) => 1,
        _ => 0,
    }
}

fn permitted(properties: &Value, forward: bool) -> bool {
    properties[if forward {
        "car_access_forward"
    } else {
        "car_access_backward"
    }]
    .as_bool()
    .unwrap_or(true)
}

fn patch_edges(road: &Road, after: &Value, patch: &Value) -> Vec<ArcEdge> {
    let mut edges = road.edges.clone();
    let props = &after["properties"];
    for edge in &mut edges {
        if let Some(speed) = patch["speed_kmh"].as_f64() {
            edge.speed = speed;
            edge.assumed = false;
        } else if patch.get("class").is_some() && props.get("speed_kmh").is_none() {
            let speed = class_speed(props["class"].as_str().unwrap());
            let cap = props[if edge.forward {
                "speed_cap_forward_kmh"
            } else {
                "speed_cap_backward_kmh"
            }]
            .as_f64();
            edge.speed = match cap {
                Some(limit) => speed.min(limit),
                None if edge.assumed => speed,
                None => edge.speed.min(speed),
            };
        }
    }
    if patch.get("oneway").is_some() {
        let direction = oneway(&props["oneway"]);
        let existing: std::collections::HashSet<_> = edges.iter().map(|e| (e.from, e.to)).collect();
        let mut reverse = Vec::new();
        for edge in &edges {
            if (direction == 0 || edge.forward != (direction == 1))
                && permitted(props, !edge.forward)
                && !existing.contains(&(edge.to, edge.from))
            {
                reverse.push(ArcEdge {
                    from: edge.to,
                    to: edge.from,
                    forward: !edge.forward,
                    ..edge.clone()
                });
            }
        }
        edges.retain(|e| direction == 0 || e.forward == (direction == 1));
        edges.extend(reverse);
    }
    edges
}

fn metres(a: [f64; 2], b: [f64; 2]) -> f64 {
    let x = (a[0] - b[0]) * 111_320. * (-33.45_f64).to_radians().cos();
    x.hypot((a[1] - b[1]) * 110_540.)
}

fn geometry_edges(
    previous: Option<&Road>,
    after: &Value,
    coords: &[[f64; 2]],
    extra: &[[f64; 2]],
    active: &[bool],
) -> Result<(Vec<[f64; 2]>, Vec<ArcEdge>, usize), String> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut snapped = 0;
    let props = &after["properties"];
    let direction = oneway(&props["oneway"]);
    let default_speed = class_speed(props["class"].as_str().unwrap_or("road"));
    let speed = |forward| {
        previous
            .and_then(|road| road.edges.iter().find(|e| e.forward == forward))
            .map(|e| (e.speed, e.assumed))
            .unwrap_or_else(|| {
                (
                    props["speed_kmh"].as_f64().unwrap_or(default_speed),
                    props["speed_kmh"].as_f64().is_none(),
                )
            })
    };
    let mut junctions = HashMap::new();
    for part in feature::lines(&after["geometry"])? {
        let mut chain = Vec::new();
        for (i, &p) in part.iter().enumerate() {
            let endpoint = i == 0 || i + 1 == part.len();
            let key = (p[0].to_bits(), p[1].to_bits());
            let near = if endpoint {
                coords
                    .iter()
                    .chain(extra)
                    .enumerate()
                    .filter(|(n, _)| active[*n])
                    .map(|(n, &q)| (n, q, metres(p, q)))
                    .filter(|x| x.2 <= 30.)
                    .min_by(|a, b| a.2.total_cmp(&b.2).then(a.0.cmp(&b.0)))
            } else {
                None
            };
            let (node, pos) = if let Some((n, q, _)) = near {
                snapped += 1;
                (n, q)
            } else if endpoint && junctions.contains_key(&key) {
                (junctions[&key], p)
            } else {
                let n = coords.len() + extra.len() + nodes.len();
                nodes.push(p);
                if endpoint {
                    junctions.insert(key, n);
                }
                (n, p)
            };
            chain.push((node, pos));
        }
        for w in chain.windows(2) {
            let [(a, p), (b, q)] = [w[0], w[1]];
            let length = metres(p, q);
            if a == b || length == 0. {
                continue;
            }
            for (forward, from, to) in [(true, a, b), (false, b, a)] {
                if (direction == 1 && !forward)
                    || (direction == -1 && forward)
                    || !permitted(props, forward)
                {
                    continue;
                }
                if let Some(road) = previous {
                    if !road.edges.iter().any(|e| e.forward == forward) {
                        continue;
                    }
                }
                let (speed, assumed) = speed(forward);
                edges.push(ArcEdge {
                    from,
                    to,
                    metres: length,
                    speed,
                    forward,
                    assumed,
                });
            }
        }
    }
    Ok((nodes, edges, snapped))
}

impl RoadEdits<'_> {
    pub fn apply(&mut self, change: &Value) -> Result<Value, String> {
        let id = change["target"]["id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or("A road change needs a string target.id.")?;
        let (data, baseline) = self.graph.baseline()?;
        let before = match self.changed.get(id) {
            Some(current) => current.as_ref(),
            None => data.roads.get(id),
        };
        if before.is_some_and(|r| r.feature.is_null()) {
            return Err(format!("Road {id} has no editable geometry."));
        }
        let after = feature::changed_feature(id, before.map(|r| &r.feature), change)?;
        let others = || {
            data.roads
                .iter()
                .filter(|(key, _)| key.as_str() != id && !self.changed.contains_key(*key))
                .map(|(_, v)| v)
                .chain(
                    self.changed
                        .iter()
                        .filter(|(key, _)| key.as_str() != id)
                        .filter_map(|(_, v)| v.as_ref()),
                )
        };
        let previous = self.minutes.as_deref().unwrap_or(baseline);
        let mut new_nodes = Vec::new();
        let mut assumptions = vec!["Minutes are shortest directed free-flow car times between fixed comuna anchors. Null means unreachable. The graph covers the mapped city rectangle. Traffic, turn restrictions and local access time are omitted. Road-induced mode changes are not modeled.".to_string()];
        let mut next = None;
        if let Some(f) = after {
            let edges = match (change["op"].as_str(), before) {
                (Some("set"), Some(road)) => patch_edges(road, &f, &change["properties"]),
                (Some("move"), Some(road)) if f["geometry"] == road.feature["geometry"] => {
                    road.edges.clone()
                }
                (Some("move"), Some(road)) if road.edges.is_empty() => Vec::new(),
                _ => {
                    let mut active = vec![false; data.nodes.len() + self.extra_nodes.len()];
                    for edge in others().flat_map(|r| &r.edges) {
                        active[edge.from] = true;
                        active[edge.to] = true;
                    }
                    let (added, edges, snapped) =
                        geometry_edges(before, &f, &data.nodes, &self.extra_nodes, &active)?;
                    new_nodes = added;
                    assumptions.push(format!("Edited geometry uses straight segments in local metres. {snapped} endpoints snapped to active road nodes within 30 m. Crossings between line interiors do not create junctions."));
                    edges
                }
            };
            if edges.iter().any(|e| e.assumed) {
                assumptions.push(format!("Car speeds use mapped limits where available and free-flow estimates by class. The class estimate for {} is {} km/h.", f["properties"]["class"], class_speed(f["properties"]["highway"].as_str().unwrap_or_else(|| f["properties"]["class"].as_str().unwrap_or("road")))));
            }
            next = Some(Road { feature: f, edges });
        }
        let minutes = route(
            data.nodes.len() + self.extra_nodes.len() + new_nodes.len(),
            others().chain(next.iter()),
            &data.anchors,
        );
        let n = data.cuts.len();
        let pairs: Vec<_> = previous.iter().zip(&minutes).enumerate().filter(|(_, (a,b))| a!=b)
            .map(|(i,(a,b))| json!({"origin_cut":data.cuts[i/n],"destination_cut":data.cuts[i%n],
                                     "before_minutes":a.is_finite().then_some(a),"after_minutes":b.is_finite().then_some(b)})).collect();
        let edit = json!({"kind":"road","op":change["op"],"id":id,"before":before.map(|r| &r.feature),
                    "after":next.as_ref().map(|r| &r.feature),"effects":{"pairs":pairs},"assumptions":assumptions});
        self.extra_nodes.extend(new_nodes);
        self.changed.insert(id.into(), next);
        self.minutes = Some(minutes);
        Ok(edit)
    }
}
