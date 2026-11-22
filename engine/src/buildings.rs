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
    // Tracks whether the original stock was known, independently of later scenario counts.
    baseline_known: [bool; 2],
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

    pub fn mapped_footprint(&self, locator: &Value) -> Result<Value, String> {
        if locator["outline"]["type"] == "Polygon" {
            geometry::area_m2(&locator["outline"])?;
            return self.catalog.matching_outline(&locator["outline"])
                .ok_or_else(|| "No native building matches that mapped outline.".into());
        }
        let point: [f64; 2] = serde_json::from_value(locator["point"].clone())
            .map_err(|_| "Select a building using a WGS84 point or a mapped outline.")?;
        if !(-180.0..=180.0).contains(&point[0]) || !(-90.0..=90.0).contains(&point[1]) {
            return Err("Use WGS84 coordinates.".into());
        }
        let mut mapped = self.catalog.at_point(point).into_iter()
            .filter_map(|id| self.catalog.feature(&id))
            .filter(|feature| feature["properties"]["source"] == "osm");
        let footprint = mapped.next().ok_or("No mapped building contains that point.")?;
        if mapped.next().is_some() {
            return Err("Several mapped buildings contain that point. Select a more precise point.".into());
        }
        Ok(footprint)
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
            baseline_known: [true; 2],
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
        Ok(self.base.catalog.feature(id).map(|mut feature| {
            let inputs = input_fields(&feature["properties"]);
            let mut assumptions = vec![
                "The catalog maps the footprint but does not measure its jobs or residents.".into(),
            ];
            let area = geometry::area_m2(&feature["geometry"])
                .map_err(|reason| {
                    assumptions.push(format!(
                        "The catalog footprint area is unavailable: {reason}"
                    ));
                })
                .ok();
            let p = &mut feature["properties"];
            p["footprint_area_m2"] = json!(area);
            p["use"] = Value::Null;
            p["jobs"] = Value::Null;
            p["residents"] = Value::Null;
            Building {
                feature,
                inputs,
                assumptions,
                baseline_known: [false; 2],
            }
        }))
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
                let allocation = self.base.catalog.block(polygon)?;
                feature["geometry"] = polygon.clone();
                feature["properties"]
                    .as_object_mut()
                    .unwrap()
                    .extend(allocation.as_object().unwrap().clone());
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
                if change["properties"].get("use").is_none_or(|v| {
                    v.is_string() && v.as_str() == old.feature["properties"]["use"].as_str()
                }) && change["properties"].get("height_m").is_none_or(|v| {
                    v.is_number() && v.as_f64() == old.feature["properties"]["height_m"].as_f64()
                }) =>
            {
                let mut notes = old.assumptions.clone();
                for (field, value) in change["properties"].as_object().unwrap() {
                    if field == "height_m" || field == "use" {
                        continue;
                    }
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
                    baseline_known: old.baseline_known,
                })
            }
            (Some((feature, inputs)), prior) => {
                let mut building = self.base.building(feature, inputs)?;
                if let Some(old) = prior {
                    building.baseline_known = old.baseline_known;
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
            if building.feature["properties"]["block_shares"]
                .as_array()
                .is_some_and(|s| s.len() > 1)
                || building.feature["properties"]["block_covered_fraction"]
                    .as_f64()
                    .is_some_and(|f| f < 0.999999)
            {
                let note = "Known stocks use uniform density over the footprint's covered census-block area. Shares sum to one across covered area; street gaps receive no stock. Scalar block metadata and any occupancy rate use the largest-area block.".to_owned();
                if !assumptions.contains(&note) {
                    assumptions.push(note);
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

fn block_shares(properties: &Value) -> Vec<Value> {
    if let Some(shares) = properties["block_shares"].as_array() {
        return shares.clone();
    }
    if properties["block_id"].is_string()
        && properties["cut"].is_string()
        && properties["block_longitude"].is_number()
        && properties["block_latitude"].is_number()
    {
        vec![
            json!({"block_id":properties["block_id"],"cut":properties["cut"],
            "longitude":properties["block_longitude"],"latitude":properties["block_latitude"],"share":1.0}),
        ]
    } else {
        Vec::new()
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
            .any(|b| !b.baseline_known[index])
        {
            notes.push(format!("The catalog {field} count is unknown, so the net change from the catalog baseline is unavailable."));
            if before
                .iter()
                .chain(after.iter())
                .any(|b| !b.feature["properties"][field].is_number())
            {
                continue;
            }
            notes.push(format!(
                "The {field} delta compares consecutive scenario states."
            ));
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
        if before
            .iter()
            .chain(after.iter())
            .any(|b| block_shares(&b.feature["properties"]).is_empty())
        {
            notes.push(format!("The footprint lacks a mapped Census block. This edit does not apply a {field} delta."));
            continue;
        }
        for (p, contribution) in sides {
            for share in block_shares(p) {
                let id = share["block_id"].as_str().unwrap();
                let row = blocks.entry(id.to_owned()).or_insert_with(|| json!({"block_id":id,"cut":share["cut"],"longitude":share["longitude"],"latitude":share["latitude"]}));
                let value = row[delta].as_f64().unwrap_or(0.0)
                    + contribution * share["share"].as_f64().unwrap();
                row[delta] = json!(value);
            }
        }
    }
    blocks.into_values().collect()
}
