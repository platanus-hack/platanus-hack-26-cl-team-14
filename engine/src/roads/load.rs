use super::{ArcEdge, Road};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub(super) struct Loaded {
    pub nodes: Vec<[f64; 2]>,
    pub roads: BTreeMap<String, Road>,
    pub cuts: Vec<String>,
    pub anchors: Vec<usize>,
}

#[derive(Deserialize)]
struct Graph {
    nodes: BTreeMap<String, [f64; 2]>,
    ways: BTreeMap<String, Way>,
    anchors: HashMap<String, Option<String>>,
}

#[derive(Deserialize)]
struct Way {
    parts: Vec<Vec<String>>,
    edges: Vec<Edge>,
    #[serde(flatten)]
    properties: Map<String, Value>,
}

#[derive(Deserialize)]
struct Edge {
    id: String,
    from: String,
    to: String,
    length_m: f64,
    speed_kmh: f64,
    speed_source: String,
}

#[derive(Deserialize)]
struct Commune {
    cut: String,
    longitude: f64,
    latitude: f64,
}

fn geometry(mut lines: Vec<Value>) -> Value {
    if lines.len() == 1 {
        json!({"type":"LineString", "coordinates":lines.pop().unwrap()})
    } else {
        json!({"type":"MultiLineString", "coordinates":lines})
    }
}

fn display_roads(dir: &Path) -> BTreeMap<String, Road> {
    #[derive(Deserialize)]
    struct Collection {
        features: Vec<Value>,
    }
    let collection: Collection = serde_json::from_reader(BufReader::new(
        File::open(dir.join("roads.geojson")).unwrap(),
    ))
    .unwrap();
    let mut grouped = BTreeMap::new();
    for mut feature in collection.features {
        let props = &mut feature["properties"];
        let id = &props["id"];
        let key = id.as_str().map_or_else(|| id.to_string(), str::to_owned);
        props["id"] = json!(key);
        if props.get("class").is_none() {
            props["class"] = props["kind"].clone();
        }
        feature["id"] = json!(key);
        let line = feature["geometry"]["coordinates"].take();
        let (_, lines) = grouped.entry(key).or_insert_with(|| (feature, Vec::new()));
        lines.push(line);
    }
    grouped
        .into_iter()
        .map(|(id, (mut feature, lines))| {
            feature["geometry"] = geometry(lines);
            (
                id,
                Road {
                    feature,
                    edges: Vec::new(),
                },
            )
        })
        .collect()
}

pub(super) fn load(dir: &Path) -> Loaded {
    let graph: Graph = serde_json::from_reader(BufReader::new(
        File::open(dir.join("road_graph.json")).unwrap(),
    ))
    .unwrap();
    let mut indices = HashMap::with_capacity(graph.nodes.len());
    let mut nodes = Vec::with_capacity(graph.nodes.len());
    for (id, point) in graph.nodes {
        indices.insert(id, nodes.len());
        nodes.push(point);
    }

    let mut roads = display_roads(dir);
    for (id, way) in graph.ways {
        let road = roads.entry(id.clone()).or_insert_with(|| {
            let lines: Vec<_> = way.parts.iter().filter(|part| part.len() >= 2).map(|part| {
                json!(part.iter().map(|key| nodes[indices[key]]).collect::<Vec<_>>())
            }).collect();
            let feature = if lines.is_empty() {
                Value::Null
            } else {
                json!({"type":"Feature","properties":{"id":id,"kind":way.properties["class"]},"geometry":geometry(lines)})
            };
            Road { feature, edges: Vec::new() }
        });

        let mut speeds = Map::new();
        road.edges = way
            .edges
            .into_iter()
            .map(|edge| {
                let forward = edge.id.ends_with("/f");
                let direction = if forward { "forward" } else { "backward" };
                // The pack assigns one speed and source per OSM way and direction.
                speeds.entry(direction).or_insert_with(
                    || json!({"speed_kmh":edge.speed_kmh,"speed_source":edge.speed_source}),
                );
                ArcEdge {
                    from: indices[&edge.from],
                    to: indices[&edge.to],
                    metres: edge.length_m,
                    speed: edge.speed_kmh,
                    forward,
                    assumed: edge.speed_source == "class_assumption",
                }
            })
            .collect();
        if !road.feature.is_null() {
            let props = road.feature["properties"].as_object_mut().unwrap();
            props.extend(way.properties);
            if !speeds.is_empty() {
                props.insert("source_speed_by_direction".into(), Value::Object(speeds));
            }
        }
    }

    let mut cuts = Vec::new();
    let mut anchors = Vec::new();
    for row in csv::Reader::from_path(dir.join("communes.csv"))
        .unwrap()
        .deserialize::<Commune>()
    {
        let row = row.unwrap();
        let node = match graph.anchors.get(&row.cut).and_then(Option::as_ref) {
            Some(id) => indices[id],
            None => {
                // Each uncovered comuna has its own isolated node.
                nodes.push([row.longitude, row.latitude]);
                nodes.len() - 1
            }
        };
        cuts.push(row.cut);
        anchors.push(node);
    }
    Loaded {
        nodes,
        roads,
        cuts,
        anchors,
    }
}
