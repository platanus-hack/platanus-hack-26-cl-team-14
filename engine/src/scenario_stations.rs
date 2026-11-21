//! Per-request station state. The shared fitted model and design catalog are read-only.
use super::plain;
use crate::stations::access::{
    AddedStation, StationAccess,
    planned::{PlannedStations, Selector},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub(super) struct Network {
    pub open: HashSet<String>,
    pub additions: Vec<AddedStation>,
    designs: BTreeSet<String>,
    points: BTreeMap<String, (AddedStation, bool)>,
}

impl Network {
    pub fn new(access: &StationAccess) -> Self {
        Self {
            open: access.all_parent_ids(),
            additions: Vec::new(),
            designs: BTreeSet::new(),
            points: BTreeMap::new(),
        }
    }

    pub fn apply(
        &mut self,
        raw: &Value,
        stations: &[(String, String, f64, f64)],
        catalog: &PlannedStations,
    ) -> Option<Result<Value, String>> {
        (raw["kind"] == "station").then(|| self.set(raw, stations, catalog))
    }

    fn set(
        &mut self,
        raw: &Value,
        stations: &[(String, String, f64, f64)],
        catalog: &PlannedStations,
    ) -> Result<Value, String> {
        let open = match (raw["op"].as_str(), raw["value"].as_f64()) {
            (Some("set"), Some(1.0)) => true,
            (Some("set"), Some(0.0)) => false,
            _ => return Err("A station change needs op set and value 0 or 1.".into()),
        };
        let state = if open { "open" } else { "closed" };
        let target = &raw["target"];
        let id = target["id"].as_str().unwrap_or("");
        let kind = target["type"].as_str().unwrap_or("");
        if kind == "station" {
            let physical = catalog.physical_id(id).unwrap_or(id);
            if let Some((key, name, ..)) = stations
                .iter()
                .find(|s| s.0 == physical || plain(&s.1) == plain(id))
            {
                if open {
                    self.open.insert(key.clone());
                } else {
                    self.open.remove(key);
                }
                return Ok(json!({"station": name, "state": state}));
            }
        }
        let understood = match kind {
            "point" => {
                let (Some(x), Some(y)) =
                    (target["longitude"].as_f64(), target["latitude"].as_f64())
                else {
                    return Err("A proposed station needs longitude and latitude.".into());
                };
                if !(-180.0..=180.0).contains(&x) || !(-90.0..=90.0).contains(&y) {
                    return Err("Station coordinates must be in WGS84 degrees.".into());
                }
                let key = if id.is_empty() {
                    format!("point:{x},{y}")
                } else {
                    id.to_owned()
                };
                if stations.iter().any(|s| s.0 == key) || catalog.physical_id(&key).is_some() {
                    return Err(
                        "This ID already identifies a Metro station. Use a station target.".into(),
                    );
                }
                let name = target["name"].as_str().unwrap_or(&key).to_owned();
                self.points.insert(
                    key.clone(),
                    (
                        AddedStation {
                            id: key.clone(),
                            name: name.clone(),
                            longitude: x,
                            latitude: y,
                        },
                        open,
                    ),
                );
                json!({"id": key, "station": name, "state": state})
            }
            "station" if self.points.contains_key(id) => {
                let (point, active) = self.points.get_mut(id).unwrap();
                *active = open;
                json!({"id": point.id, "station": point.name, "state": state})
            }
            "station" | "line" | "corridor" => {
                let selector = match kind {
                    "line" => Selector::Line(id),
                    "corridor" => Selector::Corridor(id),
                    _ => Selector::Id(id),
                };
                let selected = catalog.select(&[selector]);
                if selected.physical_ids.is_empty() {
                    return Err(format!("Unknown Metro {kind} ID: {id}"));
                }
                if kind == "station" && !open {
                    let physical = catalog.physical_id(id);
                    self.designs.retain(|d| catalog.physical_id(d) != physical);
                } else {
                    for design in selected.physical_ids.keys() {
                        if open {
                            self.designs.insert((*design).to_owned());
                        } else {
                            self.designs.remove(*design);
                        }
                    }
                }
                json!({"type": kind, "id": id, "state": state, "design_station_count": selected.physical_ids.len(), "physical_station_ids": selected.physical_ids.values().collect::<BTreeSet<_>>()})
            }
            _ => {
                return Err(
                    "A station change needs a station, point, line or corridor target.".into(),
                );
            }
        };
        let selectors: Vec<_> = self.designs.iter().map(|id| Selector::Id(id)).collect();
        self.additions = catalog.select(&selectors).additions;
        self.additions.extend(
            self.points
                .values()
                .filter(|(_, active)| *active)
                .map(|(point, _)| point.clone()),
        );
        Ok(understood)
    }
}
