//! Apply building changes in request order. The catalog is shared across requests.
#[path = "buildings/catalog.rs"]
mod catalog;
#[path = "buildings/geometry.rs"]
mod geometry;
#[path = "buildings/occupancy.rs"]
mod occupancy;

use catalog::Catalog;
use occupancy::Occupancy;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::path::Path;

pub struct Buildings {
    catalog: Catalog,
    occupancy: Occupancy,
}

#[derive(Clone)]
struct Building {
    feature: Value,
    // Keep scenario overrides separate from derived counts when height or use changes.
    inputs: Value,
    assumptions: Vec<String>,
    // A comparison stays unavailable if the source building had no count for this stock.
    comparable: [bool; 2],
}

pub struct BuildingEdits<'a> {
    base: &'a Buildings,
    // A missing value hides a catalog footprint removed earlier in this request.
    changed: BTreeMap<String, Option<Building>>,
}

impl Buildings {
    pub fn load(dir: &Path) -> Self {
        Self {
            catalog: Catalog::load(dir),
            occupancy: Occupancy::load(dir),
        }
    }

    pub fn edits(&self) -> BuildingEdits<'_> {
        BuildingEdits {
            base: self,
            changed: BTreeMap::new(),
        }
    }

    fn building(&self, mut feature: Value, inputs: Value) -> Result<Building, String> {
        let area = geometry::area_m2(&feature["geometry"])?;
        let cut = feature["properties"]["cut"].as_str().unwrap_or("");
        let (jobs, residents, mut assumptions) = self.occupancy.estimate(cut, &inputs, area)?;
        if inputs.get("height_m").is_none() {
            assumptions.push("Missing drawing height assumes one storey of 3.5 m.".into());
        }
        let properties = feature["properties"]
            .as_object_mut()
            .ok_or("Missing drawing properties.")?;
        properties.extend(inputs.as_object().unwrap().clone());
        properties.entry("height_m").or_insert(json!(3.5));
        properties.entry("use").or_insert(json!("unknown"));
        properties.insert("footprint_area_m2".into(), json!(area));
        properties.insert("jobs".into(), json!(jobs));
        properties.insert("residents".into(), json!(residents));
        Ok(Building {
            feature,
            inputs,
            assumptions,
            comparable: [true; 2],
        })
    }
}

fn input_fields(properties: &Value) -> Value {
    let mut inputs = Map::new();
    for field in ["use", "height_m", "jobs", "residents"] {
        if let Some(value) = properties.get(field).filter(|v| !v.is_null()) {
            inputs.insert(field.into(), value.clone());
        }
    }
    Value::Object(inputs)
}

impl BuildingEdits<'_> {
    fn select(&self, target: &Value) -> Result<String, String> {
        if let Some(id) = target["id"].as_str().filter(|id| !id.trim().is_empty()) {
            if target.get("point").is_some() {
                return Err("Select a building by ID or point.".into());
            }
            return Ok(id.to_owned());
        }
        let point: [f64; 2] = serde_json::from_value(target["point"].clone())
            .map_err(|_| "Select a building by ID or a WGS84 point.")?;
        let mut ids = self.base.catalog.at_point(point);
        ids.retain(|id| !self.changed.contains_key(id));
        for (id, building) in &self.changed {
            if building
                .as_ref()
                .is_some_and(|b| geometry::contains(&b.feature["geometry"], point))
            {
                ids.push(id.clone());
            }
        }
        match ids.as_slice() {
            [id] => Ok(id.clone()),
            [] => Err("The point is outside all available building footprints.".into()),
            _ => {
                Err("The point selects multiple building footprints. Supply a building ID.".into())
            }
        }
    }

    fn current(&self, id: &str) -> Result<Option<Building>, String> {
        if let Some(entry) = self.changed.get(id) {
            return Ok(entry.clone());
        }
        self.base
            .catalog
            .feature(id)
            .map(|f| {
                let inputs = input_fields(&f["properties"]);
                let mut b = self.base.building(f, inputs)?;
                b.comparable =
                    ["jobs", "residents"].map(|k| b.feature["properties"][k].is_number());
                Ok::<_, String>(b)
            })
            .transpose()
    }

    pub fn apply(&mut self, change: &Value) -> Result<Value, String> {
        let op = change["op"]
            .as_str()
            .ok_or("Supply a building operation.")?;
        if op == "add"
            && change["target"]["id"]
                .as_str()
                .is_none_or(|s| s.trim().is_empty())
        {
            return Err("A new building needs an ID.".into());
        }
        let id = self.select(&change["target"])?;
        if op == "add" && id.starts_with("mesh:") {
            return Err("IDs starting with mesh: are reserved for catalog footprints.".into());
        }
        let before = self.current(&id)?;
        let mut after = match (op, before.as_ref()) {
            ("add", Some(_)) => return Err(format!("Building {id} already exists.")),
            ("add", None) => Some((
                json!({"type":"Feature", "id":id, "properties":{}}),
                json!({}),
            )),
            ("set" | "move", Some(old)) => Some((old.feature.clone(), old.inputs.clone())),
            ("remove", Some(_)) => None,
            ("remove" | "set" | "move", None) => return Err(format!("Unknown building {id}.")),
            _ => return Err("Use add, set, move or remove for a building.".into()),
        };
        if matches!(op, "remove" | "move") && change.get("properties").is_some() {
            return Err("Use set to change building properties.".into());
        }
        if matches!(op, "remove" | "set") && change.get("geometry").is_some() {
            return Err("Use move to replace building geometry.".into());
        }
        if let Some((feature, inputs)) = &mut after {
            if matches!(op, "set" | "add") {
                let patch = change["properties"]
                    .as_object()
                    .ok_or("Supply building properties.")?;
                if patch.is_empty()
                    || patch
                        .keys()
                        .any(|k| !matches!(k.as_str(), "use" | "height_m" | "jobs" | "residents"))
                {
                    return Err("Use building properties use, height_m, jobs or residents.".into());
                }
                inputs.as_object_mut().unwrap().extend(patch.clone());
                if patch.contains_key("height_m") {
                    feature["properties"]["height_source"] = json!("scenario");
                } else if op == "add" {
                    feature["properties"]["height_source"] = json!("default");
                }
                if op == "add" {
                    feature["properties"]["source"] = json!("scenario");
                }
            }
            if matches!(op, "move" | "add") {
                let polygon = &change["geometry"];
                geometry::area_m2(polygon)?;
                let (block, cut, longitude, latitude) = self.base.catalog.block(polygon)?;
                feature["geometry"] = polygon.clone();
                feature["properties"]["block_id"] = json!(block);
                feature["properties"]["cut"] = json!(cut);
                feature["properties"]["block_longitude"] = json!(longitude);
                feature["properties"]["block_latitude"] = json!(latitude);
            }
        }
        let after = match (after, before.as_ref()) {
            (None, _) => None,
            (Some((feature, _)), Some(old)) if op == "move" => {
                let mut moved = old.clone();
                moved.feature = feature;
                moved.feature["properties"]["footprint_area_m2"] =
                    json!(geometry::area_m2(&moved.feature["geometry"])?);
                moved
                    .assumptions
                    .push("Moving the building preserves its modeled jobs and residents.".into());
                Some(moved)
            }
            (Some((mut feature, inputs)), Some(old))
                if change["properties"].get("use").is_none()
                    && change["properties"].get("height_m").is_none() =>
            {
                let mut notes = old.assumptions.clone();
                for (field, value) in change["properties"].as_object().unwrap() {
                    let count = value
                        .as_f64()
                        .filter(|n| n.is_finite() && *n >= 0.0)
                        .ok_or_else(|| format!("{field} must be a finite, nonnegative number."))?;
                    feature["properties"][field] = json!(count);
                    notes.push(format!(
                        "This edit explicitly sets the building {field} count."
                    ));
                }
                Some(Building {
                    feature,
                    inputs,
                    assumptions: notes,
                    comparable: old.comparable,
                })
            }
            (Some((feature, inputs)), prior) => {
                let mut building = self.base.building(feature, inputs)?;
                if let Some(old) = prior {
                    building.comparable = old.comparable;
                }
                Some(building)
            }
        };
        let mut assumptions = Vec::new();
        for building in before.iter().chain(after.iter()) {
            for note in &building.assumptions {
                if !assumptions.contains(note) {
                    assumptions.push(note.clone());
                }
            }
        }
        let effects = block_effects(before.as_ref(), after.as_ref(), &mut assumptions);
        let edit = json!({"kind":"building","op":op,"id":id,
            "before":before.as_ref().map(|b| &b.feature),"after":after.as_ref().map(|b| &b.feature),
            "effects":{"blocks":effects},"assumptions":assumptions});
        self.changed.insert(id, after);
        Ok(edit)
    }
}

fn block_effects(
    before: Option<&Building>,
    after: Option<&Building>,
    notes: &mut Vec<String>,
) -> Vec<Value> {
    let mut blocks = BTreeMap::new();
    for (index, (field, delta)) in [("jobs", "jobs_delta"), ("residents", "residents_delta")]
        .into_iter()
        .enumerate()
    {
        if before
            .iter()
            .chain(after.iter())
            .any(|b| !b.comparable[index])
        {
            notes.push(format!("The source or edited building has no comparable {field} count. This edit does not apply a {field} delta."));
            continue;
        }
        let mut sides = [(before, -1.0), (after, 1.0)]
            .into_iter()
            .filter_map(|(b, sign)| {
                let p = &b?.feature["properties"];
                Some((p, sign * p[field].as_f64()?))
            })
            .peekable();
        if sides.peek().is_none() {
            notes.push(format!("No building {field} count is known for this edit."));
            continue;
        }
        if before
            .iter()
            .chain(after.iter())
            .any(|b| b.feature["properties"][field].is_null())
        {
            notes.push(format!(
                "Only the known scenario contribution to block {field} is applied or withdrawn."
            ));
        }
        if before.iter().chain(after.iter()).any(|b| {
            let p = &b.feature["properties"];
            !p["block_id"].is_string()
                || !p["cut"].is_string()
                || !p["block_longitude"].is_number()
                || !p["block_latitude"].is_number()
        }) {
            notes.push(format!("The footprint lacks a mapped Census block. This edit does not apply a {field} delta."));
            continue;
        }
        for (p, contribution) in sides {
            let id = p["block_id"].as_str().unwrap();
            let row = blocks.entry(id.to_owned()).or_insert_with(|| json!({"block_id":id,"cut":p["cut"],"longitude":p["block_longitude"],"latitude":p["block_latitude"]}));
            let value = row[delta].as_f64().unwrap_or(0.0) + contribution;
            row[delta] = json!(value);
        }
    }
    blocks.into_values().collect()
}
