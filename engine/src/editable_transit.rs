//! Edits a directional transit pattern and reports changes in weekday 08:00 travel times.
mod routing;

use routing::Routing;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Deserialize)]
struct Stop {
    coordinates: [f64; 2],
    parent_id: Option<String>,
}

#[derive(Clone, Deserialize)]
struct Pattern {
    mode: String,
    route_id: Option<String>,
    direction: Option<Value>,
    shape_id: Option<String>,
    stop_ids: Vec<String>,
    segment_minutes: Vec<f64>,
    dwell_minutes: Vec<f64>,
    pickup_type: Vec<u8>,
    drop_off_type: Vec<u8>,
    headway_minutes: Option<f64>,
    #[serde(skip)]
    geometry: Option<Value>,
}

impl Pattern {
    fn same_service(&self, other: &Self) -> bool {
        self.stop_ids == other.stop_ids
            && self.segment_minutes == other.segment_minutes
            && self.dwell_minutes == other.dwell_minutes
            && self.headway_minutes == other.headway_minutes
            && self.pickup_type == other.pickup_type
            && self.drop_off_type == other.drop_off_type
    }

    fn feature(&self, id: &str, catalog: &LineCatalog) -> Value {
        let geometry = self.geometry.clone().unwrap_or_else(|| {
            let coordinates = match self
                .shape_id
                .as_ref()
                .and_then(|id| catalog.pack.shapes.get(id))
            {
                Some(shape) => shape.clone(),
                None => self
                    .stop_ids
                    .iter()
                    .map(|stop| catalog.pack.stops[stop].coordinates)
                    .collect(),
            };
            json!({"type": "LineString", "coordinates": coordinates})
        });
        json!({"type": "Feature", "id": id, "geometry": geometry, "properties": {
            "id": id, "mode": self.mode, "route_id": self.route_id, "direction": self.direction,
            "stop_ids": self.stop_ids, "segment_minutes": self.segment_minutes,
            "dwell_minutes": self.dwell_minutes, "headway_minutes": self.headway_minutes,
            "pickup_type": self.pickup_type, "drop_off_type": self.drop_off_type
        }})
    }
}

#[derive(Deserialize)]
struct Pack {
    stops: BTreeMap<String, Stop>,
    lines: BTreeMap<String, Pattern>,
    shapes: BTreeMap<String, Vec<[f64; 2]>>,
}

pub struct LineCatalog {
    pack: Pack,
    cuts: Vec<String>,
    routing: Routing,
    base: OnceLock<Vec<Option<f64>>>,
}

impl LineCatalog {
    pub fn load(dir: &Path) -> Self {
        let pack: Pack = serde_json::from_reader(
            std::fs::File::open(dir.join("editable_transit.json")).unwrap(),
        )
        .unwrap();
        let (cuts, points): (Vec<_>, Vec<_>) = csv::Reader::from_path(dir.join("communes.csv"))
            .unwrap()
            .records()
            .map(|row| {
                let row = row.unwrap();
                (
                    row[0].to_owned(),
                    [row[5].parse().unwrap(), row[6].parse().unwrap()],
                )
            })
            .unzip();
        let routing = Routing::new(&pack.stops, &points);
        Self {
            pack,
            cuts,
            routing,
            base: OnceLock::new(),
        }
    }

    pub fn edits(&self) -> LineEdits<'_> {
        LineEdits {
            catalog: self,
            changes: BTreeMap::new(),
            minutes: None,
        }
    }

    fn baseline(&self) -> &[Option<f64>] {
        self.base
            .get_or_init(|| self.routing.minutes(self.pack.lines.values()))
    }
}

pub struct LineEdits<'a> {
    catalog: &'a LineCatalog,
    changes: BTreeMap<String, Option<Pattern>>,
    minutes: Option<Vec<Option<f64>>>,
}

impl LineEdits<'_> {
    pub fn apply(&mut self, raw: &Value) -> Result<Value, String> {
        let id = raw["target"]["id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or("A line edit needs a target ID.")?;
        let before = match self.changes.get(id) {
            Some(pattern) => pattern.as_ref(),
            None => self.catalog.pack.lines.get(id),
        };
        let op = raw["op"].as_str().unwrap_or("");
        let after = match (op, before) {
            ("remove", Some(_)) => None,
            ("add", None) | ("set" | "move", Some(_)) => Some(self.updated(raw, before)?),
            ("add", Some(_)) => return Err(format!("Line {id} already exists.")),
            ("remove" | "set" | "move", None) => return Err(format!("Unknown line ID: {id}.")),
            _ => return Err("Use add, remove, set or move for a line.".into()),
        };

        let changed_service = match (before, after.as_ref()) {
            (Some(before), Some(after)) => !before.same_service(after),
            _ => true,
        };
        let before = before.map(|pattern| pattern.feature(id, self.catalog));
        let after_feature = after
            .as_ref()
            .map(|pattern| pattern.feature(id, self.catalog));
        self.changes.insert(id.to_owned(), after);
        let mut pairs = Vec::new();
        if changed_service {
            let patterns = self
                .catalog
                .pack
                .lines
                .iter()
                .filter(|(id, _)| !self.changes.contains_key(*id))
                .map(|(_, pattern)| pattern)
                .chain(self.changes.values().filter_map(Option::as_ref));
            let next = self.catalog.routing.minutes(patterns);
            let previous = self
                .minutes
                .as_deref()
                .unwrap_or_else(|| self.catalog.baseline());
            let cuts = &self.catalog.cuts;
            for (i, (&before, &after)) in previous.iter().zip(&next).enumerate() {
                if match (before, after) {
                    (Some(before), Some(after)) => (before - after).abs() >= 0.000001,
                    _ => before != after,
                } {
                    pairs.push(json!({
                        "origin_cut": cuts[i / cuts.len()], "destination_cut": cuts[i % cuts.len()],
                        "before_minutes": before, "after_minutes": after
                    }));
                }
            }
            self.minutes = Some(next);
        }

        Ok(json!({
            "kind": "line", "op": op, "id": id, "before": before, "after": after_feature,
            "effects": {"pairs": pairs},
            "assumptions": [
                "The weekday graph uses half the first-stop headway estimated from starts from 08:00 to 09:00 on 13 April 2026. It omits patterns without starts in that window and does not estimate stop-level service from earlier trips.",
                "Access and egress use straight-line walks up to 1 km. Stop transfers use straight-line walks up to 300 m.",
                "The graph measures travel between comuna population points. A route must include at least one transit segment. Null means no such route in this graph.",
                "A geometry-only move keeps service times and the stop order. If an edited pattern omits dwell times for new stops, it uses zero dwell."
            ]
        }))
    }

    fn updated(&self, raw: &Value, before: Option<&Pattern>) -> Result<Pattern, String> {
        let properties = &raw["properties"];
        let mut fields = match before {
            Some(pattern) => json!({
                "mode": pattern.mode, "route_id": pattern.route_id, "direction": pattern.direction,
                "stop_ids": pattern.stop_ids, "shape_id": pattern.shape_id,
                "segment_minutes": pattern.segment_minutes,
                "dwell_minutes": pattern.dwell_minutes, "headway_minutes": pattern.headway_minutes,
                "pickup_type": pattern.pickup_type, "drop_off_type": pattern.drop_off_type
            }),
            None => json!({}),
        };
        if properties.get("stop_ids").is_some() && properties.get("segment_minutes").is_none() {
            return Err("Supply segment minutes when you change the stop list.".into());
        }
        if raw["op"] == "move" && raw.get("geometry").is_none() {
            return Err("A line move needs a replacement geometry.".into());
        }
        let stops_changed = properties
            .get("stop_ids")
            .is_some_and(|stops| before.is_none_or(|pattern| *stops != json!(pattern.stop_ids)));
        if stops_changed {
            fields["shape_id"] = Value::Null;
            let stops = properties["stop_ids"].as_array().map_or(0, Vec::len);
            fields["dwell_minutes"] = json!(vec![0.0; stops]);
            fields["pickup_type"] = json!(vec![0; stops]);
            fields["drop_off_type"] = json!(vec![0; stops]);
        }
        for key in [
            "mode",
            "stop_ids",
            "segment_minutes",
            "dwell_minutes",
            "headway_minutes",
            "pickup_type",
            "drop_off_type",
        ] {
            if let Some(value) = properties.get(key) {
                fields[key] = value.clone();
            }
        }
        let mut pattern: Pattern = serde_json::from_value(fields)
            .map_err(|_| "Supply a mode, known stop IDs, and travel minutes per segment.")?;
        if !matches!(pattern.mode.as_str(), "metro" | "bus")
            || pattern.stop_ids.len() < 2
            || pattern
                .stop_ids
                .iter()
                .any(|id| !self.catalog.pack.stops.contains_key(id))
            || pattern.segment_minutes.len() + 1 != pattern.stop_ids.len()
            || pattern
                .segment_minutes
                .iter()
                .any(|&minutes| minutes <= 0.0)
            || pattern.dwell_minutes.len() != pattern.stop_ids.len()
            || pattern.dwell_minutes.iter().any(|&minutes| minutes < 0.0)
            || pattern
                .headway_minutes
                .is_some_and(|minutes| minutes <= 0.0)
            || pattern.pickup_type.len() != pattern.stop_ids.len()
            || pattern.drop_off_type.len() != pattern.stop_ids.len()
            || (before.is_none() && pattern.headway_minutes.is_none())
        {
            return Err("A line needs a supported mode, at least two known stops, positive segment times and headway for new service, nonnegative dwell times, and one pickup and drop-off rule per stop.".into());
        }

        if let Some(geometry) = raw.get("geometry") {
            let coords =
                serde_json::from_value::<Vec<[f64; 2]>>(geometry["coordinates"].clone()).ok();
            if geometry["type"] != "LineString"
                || !coords.is_some_and(|coords| {
                    coords.len() >= 2
                        && coords.iter().all(|&[x, y]| {
                            (-180.0..=180.0).contains(&x) && (-90.0..=90.0).contains(&y)
                        })
                })
            {
                return Err("Line geometry needs at least two WGS84 positions.".into());
            }
            pattern.geometry = Some(geometry.clone());
        } else if !stops_changed {
            pattern.geometry = before.and_then(|pattern| pattern.geometry.clone());
        }
        Ok(pattern)
    }
}
