//! Editable road geometry and directed, free-flow driving times between comuna anchors.
mod cost;
mod feature;
mod geometry;
mod load;
mod routing;

use serde_json::{Value, json};
use std::collections::BTreeMap;
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

#[derive(Clone)]
struct Segment {
    from: usize,
    to: usize,
    metres: f64,
}

struct Road {
    segments: Vec<Segment>,
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

fn metres(a: [f64; 2], b: [f64; 2]) -> f64 {
    let x = (a[0] - b[0]) * 111_320. * (-33.45_f64).to_radians().cos();
    x.hypot((a[1] - b[1]) * 110_540.)
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
        if let Some(mut f) = after {
            let op = change["op"].as_str().unwrap();
            let props = &mut f["properties"];
            if op == "add" && change["properties"].get("class").is_none() {
                props["class"] = json!("secondary");
                props["kind"] = json!("secondary");
                assumptions
                    .push("A new road without a class uses the secondary-road estimate.".into());
            }
            if op == "add"
                || (change["properties"].get("class").is_some()
                    && before
                        .is_some_and(|road| road.feature["properties"]["class"] != props["class"]))
            {
                // Keep the original OSM highway for provenance; routing_class describes this scenario.
                let class = props["class"].as_str().unwrap();
                let routing = props["highway"]
                    .as_str()
                    .filter(|source| source.strip_suffix("_link") == Some(class))
                    .unwrap_or(class);
                props["routing_class"] = json!(routing);
            }
            let class = props["routing_class"].as_str().unwrap_or("road");
            if cost::speed_for_class(class).is_none() && props["speed_kmh"].as_f64().is_none() {
                return Err(format!("Road class {class} needs an explicit speed_kmh."));
            }
            if op == "add" && props.get("oneway").is_none() {
                props["oneway"] = json!("no");
                assumptions
                    .push("A new road without a direction permits cars in both directions.".into());
            }
            let segments = match before {
                Some(road) if op == "set" || f["geometry"] == road.feature["geometry"] => {
                    road.segments.clone()
                }
                _ => {
                    let mut active = vec![false; data.nodes.len() + self.extra_nodes.len()];
                    for edge in others().flat_map(|r| &r.edges) {
                        active[edge.from] = true;
                        active[edge.to] = true;
                    }
                    let (added, segments, snapped) = geometry::segments(
                        before,
                        &f,
                        &data.nodes,
                        &self.extra_nodes,
                        &active,
                        &data.anchors,
                    )?;
                    new_nodes = added;
                    assumptions.push(format!("Edited geometry uses local metres. {snapped} endpoints snapped to active road nodes within 30 m. Retained junctions and comuna anchors keep their original identities. Crossing line interiors do not create junctions."));
                    segments
                }
            };
            let edges = cost::edges(&segments, &f["properties"]);
            if edges.iter().any(|e| e.assumed) {
                let class = f["properties"]["routing_class"].as_str().unwrap_or("road");
                if let Some(speed) = cost::speed_for_class(class) {
                    assumptions.push(format!("The free-flow estimate for {class} is {speed} km/h when no lower mapped limit applies."));
                }
            }
            next = Some(Road {
                feature: f,
                segments,
                edges,
            });
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
