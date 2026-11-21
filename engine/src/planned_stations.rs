//! Select records from the design catalog, then project each new physical station once.
//! Current GTFS parents keep their original access point and open/closed state.
//! The catalog describes proposed infrastructure. It supplies no timetable or transfer costs.
use crate::stations::access::AddedStation;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub enum Selector<'a> {
    /// A station's line-specific design ID, such as L7-13.
    Id(&'a str),
    /// All corridors of the line. L6 selects both extensions.
    Line(&'a str),
    /// One corridor, such as L6-west.
    Corridor(&'a str),
}

pub struct PlannedStations {
    features: Vec<Value>,
    physical: HashMap<String, String>,
    // A new physical ID identifies its canonical design point in the full source catalog.
    new_points: HashMap<String, usize>,
}

pub struct Selection<'a> {
    /// Full source station and route features in catalog order. ID selection returns a station only.
    pub features: Vec<&'a Value>,
    /// One entry per selected line-specific station record, including existing interchanges.
    pub physical_ids: BTreeMap<&'a str, &'a str>,
    /// Unique new physical stations, ready for station access. Existing parents remain in its open set.
    pub additions: Vec<AddedStation>,
}

impl PlannedStations {
    pub fn load(data: &Path) -> Self {
        let mut source: Value = serde_json::from_reader(File::open(data.join("metro_planned.geojson")).unwrap()).unwrap();
        let features = std::mem::take(source["features"].as_array_mut().unwrap());
        let stations: Vec<_> = features.iter().enumerate()
            .filter(|(_, f)| f["properties"]["kind"] == "station")
            .map(|(i, f)| (i, &f["properties"]))
            .collect();
        let by_id: HashMap<_, _> = stations.iter().enumerate()
            .map(|(i, (_, p))| (p["id"].as_str().unwrap(), i)).collect();

        // A group can span several lines. Use all links before applying the user's selection.
        let mut group: Vec<_> = (0..stations.len()).collect();
        for (i, (_, p)) in stations.iter().enumerate() {
            if let Some(other) = p["planned_interchange_id"].as_str() {
                let old = group[by_id[other]];
                let new = group[i];
                for id in &mut group {
                    if *id == old { *id = new; }
                }
            }
        }

        let mut physical = HashMap::new();
        let mut new_points = HashMap::new();
        for (i, (_, p)) in stations.iter().enumerate() {
            let peers = stations.iter().enumerate().filter(|(j, _)| group[*j] == group[i]);
            let existing = peers.clone().find_map(|(_, (_, s))| s["gtfs_parent_station_id"].as_str());
            let id = p["id"].as_str().unwrap();
            if let Some(parent) = existing {
                physical.insert(id.to_owned(), parent.to_owned());
            } else {
                let (_, (index, canonical)) = peers.min_by(|(_, (_, left)), (_, (_, right))| {
                    left["id"].as_str().cmp(&right["id"].as_str())
                }).unwrap();
                let canonical = canonical["id"].as_str().unwrap();
                physical.insert(id.to_owned(), canonical.to_owned());
                new_points.insert(canonical.to_owned(), *index);
            }
        }
        Self { features, physical, new_points }
    }

    /// Resolve a design station ID to its physical ID. A closure of an existing interchange
    /// uses this ID in the usual GTFS open set. A closure of a new point removes its canonical
    /// ID from the additions. Omitting a corridor leaves a shared point selected by its other corridor.
    pub fn physical_id(&self, design_id: &str) -> Option<&str> {
        self.physical.get(design_id).map(String::as_str)
    }

    /// Select the union of all selectors. Repeating an ID, a line, or a corridor keeps one copy.
    /// Route geometry is included for a selected line or corridor. Selection never reopens a current station.
    pub fn select(&self, selectors: &[Selector<'_>]) -> Selection<'_> {
        let mut features = Vec::new();
        let mut physical_ids = BTreeMap::new();
        let mut new = BTreeSet::new();
        for feature in &self.features {
            let p = &feature["properties"];
            let station = p["kind"] == "station";
            if !selectors.iter().any(|selector| match selector {
                Selector::Id(id) => station && p["id"].as_str() == Some(*id),
                Selector::Line(line) => p["line"].as_str() == Some(*line),
                Selector::Corridor(corridor) => p["corridor"].as_str() == Some(*corridor),
            }) { continue; }
            features.push(feature);
            if station {
                let id = p["id"].as_str().unwrap();
                let physical = self.physical[id].as_str();
                physical_ids.insert(id, physical);
                if self.new_points.contains_key(physical) { new.insert(physical); }
            }
        }
        let additions = new.into_iter().map(|id| {
            let f = &self.features[self.new_points[id]];
            let xy = &f["geometry"]["coordinates"];
            AddedStation {
                id: id.to_owned(),
                name: f["properties"]["name"].as_str().unwrap().to_owned(),
                longitude: xy[0].as_f64().unwrap(),
                latitude: xy[1].as_f64().unwrap(),
            }
        }).collect();
        Selection { features, physical_ids, additions }
    }
}
