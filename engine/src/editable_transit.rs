//! Edits a directional transit pattern and reports changes in weekday 08:00 travel times.
mod routing;
mod timetable;

use routing::Routing;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use std::sync::{Arc, OnceLock};
use timetable::{Calendar, EncodedSchedule, Exception, Schedule, Starts, active_services};

#[derive(Deserialize)]
struct Stop {
    coordinates: [f64; 2],
    parent_id: Option<String>,
}

#[derive(Deserialize)]
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
    /// Stops that are not in the GTFS catalog, for example river piers: ID → WGS84 [longitude, latitude].
    #[serde(default)]
    new_stops: BTreeMap<String, [f64; 2]>,
    /// Passengers per vehicle. None means no capacity limit in the rider model.
    #[serde(default)]
    vehicle_capacity: Option<f64>,
    #[serde(flatten)]
    encoded: EncodedSchedule,
    #[serde(skip)]
    schedules: Arc<Vec<Schedule>>,
    #[serde(skip)]
    geometry: Option<Value>,
}

impl Pattern {
    fn same_service(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.schedules, &other.schedules)
            && self.pickup_type == other.pickup_type
            && self.drop_off_type == other.drop_off_type
    }

    fn same_timing(&self, other: &Self) -> bool {
        self.stop_ids == other.stop_ids
            && self.segment_minutes == other.segment_minutes
            && self.dwell_minutes == other.dwell_minutes
            && self.headway_minutes == other.headway_minutes
    }

    fn call_times(&self) -> Vec<[f64; 2]> {
        let mut departure = 0.0;
        let mut calls = vec![[-self.dwell_minutes[0], departure]];
        for (&segment, &dwell) in self.segment_minutes.iter().zip(&self.dwell_minutes[1..]) {
            let arrival = departure + segment;
            departure = arrival + dwell;
            calls.push([arrival, departure]);
        }
        calls
    }

    fn retime(&self, previous: &Self) -> Vec<Schedule> {
        let mut unchanged = 0;
        if self.stop_ids[0] == previous.stop_ids[0] {
            unchanged = 2;
            for i in 1..self.stop_ids.len().min(previous.stop_ids.len()) {
                if self.stop_ids[i] != previous.stop_ids[i]
                    || self.segment_minutes[i - 1] != previous.segment_minutes[i - 1]
                {
                    break;
                }
                unchanged += 1;
                if self.dwell_minutes[i] != previous.dwell_minutes[i] {
                    break;
                }
                unchanged += 1;
            }
        }

        let calls = self.call_times();
        if unchanged <= 2 {
            return vec![Schedule {
                calls,
                starts: Starts::combine(&previous.schedules),
            }];
        }

        previous
            .schedules
            .iter()
            .map(|schedule| {
                let mut calls = calls.clone();
                let times = calls.as_flattened_mut();
                let old_times = schedule.calls.as_flattened();
                // Anchor the edited suffix to this trip's last unchanged event.
                let shift = old_times[unchanged - 1] - times[unchanged - 1];
                times[..unchanged].copy_from_slice(&old_times[..unchanged]);
                if self.dwell_minutes[0] != previous.dwell_minutes[0] {
                    times[0] = -self.dwell_minutes[0];
                }
                for time in &mut times[unchanged..] {
                    *time += shift;
                }
                Schedule {
                    calls,
                    starts: schedule.starts.clone(),
                }
            })
            .collect()
    }

    fn feature(&self, id: &str, catalog: &LineCatalog) -> Value {
        let geometry = self.geometry.clone().unwrap_or_else(|| {
            let stops: Vec<_> = self
                .stop_ids
                .iter()
                .map(|id| self.stop_point(id, catalog))
                .collect();
            let coordinates = self
                .shape_id
                .as_ref()
                .and_then(|id| catalog.pack.shapes.get(id))
                .and_then(|shape| served_shape(shape, &stops))
                .unwrap_or(&stops);
            json!({"type": "LineString", "coordinates": coordinates})
        });
        json!({"type": "Feature", "id": id, "geometry": geometry, "properties": {
            "id": id, "mode": self.mode, "route_id": self.route_id, "direction": self.direction, "shape_id": self.shape_id,
            "stop_ids": self.stop_ids, "segment_minutes": self.segment_minutes,
            "dwell_minutes": self.dwell_minutes, "headway_minutes": self.headway_minutes,
            "pickup_type": self.pickup_type, "drop_off_type": self.drop_off_type,
            "new_stops": self.new_stops, "vehicle_capacity": self.vehicle_capacity
        }})
    }

    fn stop_point(&self, id: &str, catalog: &LineCatalog) -> [f64; 2] {
        catalog.pack.stops.get(id).map_or_else(|| self.new_stops[id], |stop| stop.coordinates)
    }
}

// Follow the calls in order so a repeated terminal can select the end of a loop.
fn served_shape<'a>(shape: &'a [[f64; 2]], stops: &[[f64; 2]]) -> Option<&'a [[f64; 2]]> {
    let mut calls = stops.iter().scan(0, |cursor, stop| {
        let distance = |point: &[f64; 2]| {
            ((point[0] - stop[0]) * stop[1].to_radians().cos()).powi(2)
                + (point[1] - stop[1]).powi(2)
        };
        let index = (*cursor..shape.len())
            .min_by(|&a, &b| distance(&shape[a]).total_cmp(&distance(&shape[b])))?;
        *cursor = index;
        Some(index)
    });
    let first = calls.next()?;
    let last = calls.last()?;
    (last > first).then(|| &shape[first..=last])
}

#[derive(Deserialize)]
struct Pack {
    stops: BTreeMap<String, Stop>,
    lines: BTreeMap<String, Pattern>,
    shapes: BTreeMap<String, Vec<[f64; 2]>>,
    calendar: Vec<Calendar>,
    calendar_dates: Vec<Exception>,
}

pub struct LineCatalog {
    pack: Pack,
    cuts: Vec<String>,
    routing: Routing,
    base: OnceLock<Vec<Option<f64>>>,
}

impl LineCatalog {
    pub fn load(dir: &Path) -> Self {
        let mut pack: Pack = serde_json::from_reader(
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
        let active = active_services(&pack.calendar, &pack.calendar_dates);
        for pattern in pack.lines.values_mut() {
            pattern.schedules = Arc::new(std::mem::take(&mut pattern.encoded).compile(&active));
        }
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

/// An added service pattern, as the person-level rider model reads it.
pub struct AddedLine {
    pub id: String,
    pub mode: String,
    pub stop_ids: Vec<String>,
    pub stops: Vec<[f64; 2]>,
    /// [arrival, departure] minutes of each call from the first departure.
    pub calls: Vec<[f64; 2]>,
    pub headway_minutes: f64,
    pub pickup_type: Vec<u8>,
    pub drop_off_type: Vec<u8>,
    pub vehicle_capacity: Option<f64>,
}

impl LineEdits<'_> {
    /// Patterns that this request added (their IDs are not in the catalog).
    pub fn added(&self) -> Vec<AddedLine> {
        self.changes
            .iter()
            .filter(|(id, _)| !self.catalog.pack.lines.contains_key(*id))
            .filter_map(|(id, pattern)| pattern.as_ref().map(|p| (id, p)))
            .map(|(id, p)| AddedLine {
                id: id.clone(),
                mode: p.mode.clone(),
                stop_ids: p.stop_ids.clone(),
                stops: p.stop_ids.iter().map(|s| p.stop_point(s, self.catalog)).collect(),
                calls: p.call_times(),
                headway_minutes: p.headway_minutes.unwrap(),
                pickup_type: p.pickup_type.clone(),
                drop_off_type: p.drop_off_type.clone(),
                vehicle_capacity: p.vehicle_capacity,
            })
            .collect()
    }

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
            ("add", None) | ("set" | "move", Some(_)) => {
                let original = self.catalog.pack.lines.get(id);
                Some(self.updated(raw, before.or(original), original)?)
            }
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
                "Depart comuna points at 08:00 on Monday 13 April 2026 and count arrivals by 11:00. Fixed trips use scheduled boarding calls. Frequency trips use half a headway after arrival while the shifted window permits it.",
                "New or explicitly changed frequency runs at the first stop from 08:00 to 09:00. Changes to stops or ride times reuse the published service starts unless headway changes.",
                "Access and egress use straight-line walks up to 1 km. Stop transfers use straight-line walks up to 300 m.",
                "The graph measures travel between comuna population points. A route must include at least one transit segment. Null means no such route in this graph.",
                "A geometry-only move keeps service times and the stop order. For added stop calls, omitted dwell is zero and boarding and alighting are allowed. The shape follows the selected calls in order.",
                "A line that calls at new stops (outside the GTFS catalog) is not in the comuna graph and changes no comuna-pair minutes. Its riders come from the person-level line rider model."
            ]
        }))
    }

    fn updated(
        &self,
        raw: &Value,
        before: Option<&Pattern>,
        original: Option<&Pattern>,
    ) -> Result<Pattern, String> {
        let properties = &raw["properties"];
        let mut fields = match before {
            Some(pattern) => json!({
                "mode": pattern.mode, "route_id": pattern.route_id, "direction": pattern.direction,
                "stop_ids": pattern.stop_ids, "shape_id": pattern.shape_id,
                "segment_minutes": pattern.segment_minutes,
                "dwell_minutes": pattern.dwell_minutes, "headway_minutes": pattern.headway_minutes,
                "pickup_type": pattern.pickup_type, "drop_off_type": pattern.drop_off_type,
                "new_stops": pattern.new_stops, "vehicle_capacity": pattern.vehicle_capacity
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
            let stops = properties["stop_ids"]
                .as_array()
                .ok_or("Supply an ordered list of stop IDs.")?;
            let mut original_calls: BTreeMap<_, VecDeque<_>> = BTreeMap::new();
            if let Some(pattern) = before {
                for (i, id) in pattern.stop_ids.iter().enumerate() {
                    original_calls.entry(id.as_str()).or_default().push_back(i);
                }
            }
            let mut next_shape_call = 0;
            let mut dwell = Vec::with_capacity(stops.len());
            let mut pickup = Vec::with_capacity(stops.len());
            let mut drop_off = Vec::with_capacity(stops.len());
            for stop in stops {
                let retained = before.and_then(|pattern| {
                    let index = original_calls.get_mut(stop.as_str()?)?.pop_front()?;
                    if index < next_shape_call {
                        fields["shape_id"] = Value::Null;
                    }
                    next_shape_call = index + 1;
                    Some((
                        pattern.dwell_minutes[index],
                        pattern.pickup_type[index],
                        pattern.drop_off_type[index],
                    ))
                });
                if retained.is_none() {
                    fields["shape_id"] = Value::Null;
                }
                let (wait, on, off) = retained.unwrap_or((0.0, 0, 0));
                dwell.push(wait);
                pickup.push(on);
                drop_off.push(off);
            }
            fields["dwell_minutes"] = json!(dwell);
            fields["pickup_type"] = json!(pickup);
            fields["drop_off_type"] = json!(drop_off);
        }
        for key in [
            "mode",
            "stop_ids",
            "segment_minutes",
            "dwell_minutes",
            "headway_minutes",
            "pickup_type",
            "drop_off_type",
            "new_stops",
            "vehicle_capacity",
        ] {
            if let Some(value) = properties.get(key) {
                fields[key] = value.clone();
            }
        }
        let mut pattern: Pattern = serde_json::from_value(fields)
            .map_err(|_| "Supply a mode, known or new stop IDs, and travel minutes per segment.")?;
        if !matches!(pattern.mode.as_str(), "metro" | "bus" | "boat")
            || pattern.stop_ids.len() < 2
            || pattern.stop_ids.iter().any(|id| {
                !self.catalog.pack.stops.contains_key(id) && !pattern.new_stops.contains_key(id)
            })
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
            || pattern
                .pickup_type
                .iter()
                .chain(&pattern.drop_off_type)
                .any(|&code| code > 3)
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
        let matching = original
            .filter(|original| pattern.same_timing(original))
            .or_else(|| before.filter(|before| pattern.same_timing(before)));
        pattern.schedules = match (matching, before) {
            (Some(matching), _) => matching.schedules.clone(),
            (None, Some(previous)) if pattern.headway_minutes == previous.headway_minutes => {
                Arc::new(pattern.retime(previous))
            }
            _ => Arc::new(
                pattern
                    .headway_minutes
                    .map(|interval| Schedule {
                        calls: pattern.call_times(),
                        starts: Starts::assumed(interval),
                    })
                    .into_iter()
                    .collect(),
            ),
        };
        Ok(pattern)
    }
}
