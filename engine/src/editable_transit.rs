//! Edits a directional transit pattern and reports changes in weekday 08:00 travel times.
mod routing;
mod timetable;

use routing::Routing;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::path::Path;
use std::sync::{Arc, OnceLock};
use timetable::{Calendar, EncodedSchedule, Exception, Schedule, Starts, active_services};

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
            && self.new_stops == other.new_stops
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

    /// Locate the unchanged calls of a terminal extension in their original order.
    fn retained_calls(&self, previous: &Self) -> Option<std::ops::Range<usize>> {
        if self.mode != "metro" || self.stop_ids.len() <= previous.stop_ids.len() {
            return None;
        }
        let n = previous.stop_ids.len();
        let start = self
            .stop_ids
            .windows(n)
            .position(|calls| calls == previous.stop_ids)?;
        let end = start + n;
        if self.segment_minutes[start..end - 1] != previous.segment_minutes
            || self.dwell_minutes[start..end] != previous.dwell_minutes
            || self.pickup_type[start..end] != previous.pickup_type
            || self.drop_off_type[start..end] != previous.drop_off_type
        {
            return None;
        }
        Some(start..end)
    }

    fn extend_schedules(&self, previous: &Self, retained: std::ops::Range<usize>) -> Vec<Schedule> {
        let proposed = self.call_times();
        previous
            .schedules
            .iter()
            .map(|schedule| {
                let mut calls = proposed.clone();
                let prefix = schedule.calls[0][0] - calls[retained.start][0];
                let suffix = schedule.calls.last().unwrap()[1] - calls[retained.end - 1][1];
                for call in &mut calls[..retained.start] {
                    for time in call {
                        *time += prefix;
                    }
                }
                calls[retained.clone()].copy_from_slice(&schedule.calls);
                for call in &mut calls[retained.end..] {
                    for time in call {
                        *time += suffix;
                    }
                }
                Schedule {
                    calls,
                    starts: schedule.starts.clone(),
                }
            })
            .collect()
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
        catalog
            .pack
            .stops
            .get(id)
            .map_or_else(|| self.new_stops[id], |stop| stop.coordinates)
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

const ASSUMPTIONS: [&str; 6] = [
    "Depart comuna points at 08:00 on Monday 13 April 2026 and count arrivals by 11:00. Fixed trips use scheduled boarding calls. Frequency trips use half a headway after arrival while the shifted window permits it.",
    "New or explicitly changed frequency runs at the first stop from 08:00 to 09:00. Changes to stops or ride times reuse the published service starts unless headway changes.",
    "Access and egress use straight-line walks up to 1 km. Stop transfers use straight-line walks up to 300 m.",
    "The graph measures travel between comuna population points. A route must include at least one transit segment. Null means no such route in this graph.",
    "A geometry-only move keeps service times and the stop order. For added stop calls, omitted dwell is zero and boarding and alighting are allowed. The shape follows the selected calls in order.",
    "Proposed Metro stops join the comuna graph using the standard walk limits. Custom stops on other modes use only the person-level rider model.",
];

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
            metro_projects: BTreeMap::new(),
        }
    }

    fn baseline(&self) -> &[Option<f64>] {
        self.base
            .get_or_init(|| self.routing.minutes(self.pack.lines.values()))
    }
}

#[derive(Clone)]
pub struct LineEdits<'a> {
    catalog: &'a LineCatalog,
    changes: BTreeMap<String, Option<Pattern>>,
    minutes: Option<Vec<Option<f64>>>,
    metro_projects: BTreeMap<String, Vec<(String, f64)>>,
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
    pub retained_calls: Option<std::ops::Range<usize>>,
}

impl LineEdits<'_> {
    /// Patterns that this request added (their IDs are not in the catalog).
    pub fn added(&self) -> Vec<AddedLine> {
        self.changes
            .iter()
            // Sections of a cut route are existing service, not added lines.
            .filter(|(id, _)| !self.catalog.pack.lines.contains_key(*id) && !id.contains("#cut"))
            .filter_map(|(id, pattern)| pattern.as_ref().map(|p| (id, p)))
            .filter(|(id, _)| {
                !self
                    .metro_projects
                    .values()
                    .flatten()
                    .any(|(chosen, _)| chosen == *id)
            })
            .map(|(id, p)| AddedLine {
                id: id.clone(),
                mode: p.mode.clone(),
                stop_ids: p.stop_ids.clone(),
                stops: p
                    .stop_ids
                    .iter()
                    .map(|s| p.stop_point(s, self.catalog))
                    .collect(),
                calls: p.call_times(),
                headway_minutes: p.headway_minutes.unwrap(),
                pickup_type: p.pickup_type.clone(),
                drop_off_type: p.drop_off_type.clone(),
                vehicle_capacity: p.vehicle_capacity,
                retained_calls: None,
            })
            .collect()
    }

    pub fn metro_projects(&self) -> BTreeMap<String, Vec<AddedLine>> {
        self.metro_projects
            .iter()
            .filter_map(|(project, selected)| {
                let lines = selected
                    .iter()
                    .map(|(id, headway)| {
                        let p = self
                            .changes
                            .get(id)?
                            .as_ref()
                            .filter(|pattern| pattern.mode == "metro")?;
                        Some(AddedLine {
                            id: id.clone(),
                            mode: p.mode.clone(),
                            stop_ids: p.stop_ids.clone(),
                            stops: p
                                .stop_ids
                                .iter()
                                .map(|stop| p.stop_point(stop, self.catalog))
                                .collect(),
                            calls: p.call_times(),
                            headway_minutes: *headway,
                            pickup_type: p.pickup_type.clone(),
                            drop_off_type: p.drop_off_type.clone(),
                            vehicle_capacity: p.vehicle_capacity,
                            retained_calls: self
                                .catalog
                                .pack
                                .lines
                                .get(id)
                                .and_then(|previous| p.retained_calls(previous)),
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some((project.clone(), lines))
            })
            .collect()
    }

    pub fn apply(&mut self, raw: &Value) -> Result<Value, String> {
        if raw["target"]["type"] == "metro_project" {
            return self.apply_metro_project(raw);
        }
        self.apply_edit(raw, true)
    }

    fn apply_metro_project(&mut self, raw: &Value) -> Result<Value, String> {
        let project = raw["target"]["id"]
            .as_str()
            .ok_or("A Metro project needs an ID.")?;
        if self.metro_projects.contains_key(project) {
            return Err("A Metro project must use one contiguous group of edits.".into());
        }
        let changes = raw["properties"]["edits"]
            .as_array()
            .ok_or("Supply Metro pattern edits.")?;
        let mut staged = self.clone();
        let mut parts = Vec::new();
        let mut selected = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for change in changes {
            let id = change["target"]["id"]
                .as_str()
                .ok_or("Each Metro pattern needs an ID.")?;
            if !matches!(change["op"].as_str(), Some("add" | "set")) || !seen.insert(id) {
                return Err("Use each Metro pattern ID once with add or set.".into());
            }
            let mut receipt = staged.apply_edit(change, false)?;
            let pattern = staged.changes[id]
                .as_ref()
                .ok_or("Expected a Metro service pattern.")?;
            if pattern.mode != "metro" {
                return Err("Every pattern in a Metro project must have mode metro.".into());
            }
            if let Some(previous) = self.catalog.pack.lines.get(id) {
                if pattern.retained_calls(previous).is_none()
                    || pattern.headway_minutes != previous.headway_minutes
                {
                    return Err(
                        "An extension must preserve all existing calls and timetables.".into(),
                    );
                }
            }
            if let Some(value) = change["properties"].get("demand_headway_minutes") {
                let headway = value
                    .as_f64()
                    .filter(|&m| m > 0.)
                    .ok_or("Use positive minutes for the demand headway.")?;
                selected.push((id.to_owned(), headway));
            }
            receipt["after"]["properties"]["metro_project_id"] = json!(project);
            parts.push(receipt);
        }
        if selected.len() != 2 {
            return Err("Mark one demand pattern per direction with a demand headway.".into());
        }
        let [(forward_id, _), (return_id, _)] = selected.as_slice() else {
            unreachable!()
        };
        let forward = staged.changes[forward_id].as_ref().unwrap();
        let reverse = staged.changes[return_id].as_ref().unwrap();
        let existing = (
            self.catalog.pack.lines.get(forward_id),
            self.catalog.pack.lines.get(return_id),
        );
        let opposite = match existing {
            (None, None) => forward.stop_ids.iter().eq(reverse.stop_ids.iter().rev()),
            (Some(a), Some(b)) => {
                a.route_id == b.route_id
                    && a.direction != b.direction
                    && forward
                        .stop_ids
                        .iter()
                        .filter(|id| !a.stop_ids.contains(id))
                        .eq(reverse
                            .stop_ids
                            .iter()
                            .rev()
                            .filter(|id| !b.stop_ids.contains(id)))
            }
            _ => false,
        };
        if !opposite {
            return Err(
                "Use opposite directions with the same proposed calls in reverse order.".into(),
            );
        }
        let pairs = staged.reroute();
        staged.metro_projects.insert(project.to_owned(), selected);
        *self = staged;
        Ok(json!({"kind":"line", "op":"metro_project", "id":project,
            "effects":{"pairs":pairs}, "parts":parts, "assumptions":ASSUMPTIONS}))
    }

    fn apply_edit(&mut self, raw: &Value, reroute: bool) -> Result<Value, String> {
        if raw["target"]["type"] == "route" {
            return self.cut_route(raw);
        }
        if raw["target"]["type"] == "mode" {
            return self.remove_mode(raw);
        }
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
        let pairs = if changed_service && reroute {
            self.reroute()
        } else {
            Vec::new()
        };
        Ok(json!({
            "kind": "line", "op": op, "id": id, "before": before, "after": after_feature,
            "effects": {"pairs": pairs},
            "assumptions": ASSUMPTIONS
        }))
    }

    /// Remove every pattern of one mode (for example all Metro service) with one reroute.
    fn remove_mode(&mut self, raw: &Value) -> Result<Value, String> {
        let mode = raw["target"]["id"].as_str().unwrap_or("");
        if raw["op"] != "remove" {
            return Err("A mode target supports only op remove.".into());
        }
        let ids: Vec<String> = self
            .catalog
            .pack
            .lines
            .iter()
            .filter(|(id, _)| !self.changes.contains_key(*id))
            .chain(
                self.changes
                    .iter()
                    .filter_map(|(id, p)| p.as_ref().map(|p| (id, p))),
            )
            .filter(|(_, p)| p.mode == mode)
            .map(|(id, _)| id.clone())
            .collect();
        if ids.is_empty() {
            return Err(format!("No active line has mode {mode}."));
        }
        for id in &ids {
            self.changes.insert(id.clone(), None);
        }
        let pairs = self.reroute();
        Ok(json!({
            "kind": "line", "op": "remove", "id": format!("mode:{mode}"), "mode": mode,
            "patterns_removed": ids.len(), "before": null, "after": null,
            "effects": {"pairs": pairs},
            "assumptions": ASSUMPTIONS
        }))
    }

    /// Split every active pattern of one route into the runs of calls that avoid the given stops (a line cut in
    /// sections, for example Line 1 between Baquedano and Escuela Militar). One reroute.
    fn cut_route(&mut self, raw: &Value) -> Result<Value, String> {
        let route = raw["target"]["id"].as_str().unwrap_or("");
        if raw["op"] != "cut" {
            return Err("A route target supports only op cut.".into());
        }
        let cut: Vec<&str> = raw["properties"]["stop_ids"]
            .as_array()
            .ok_or("A cut needs properties.stop_ids.")?
            .iter()
            .filter_map(Value::as_str)
            .collect();
        let active: Vec<(String, Pattern)> = self
            .catalog
            .pack
            .lines
            .iter()
            .filter(|(id, _)| !self.changes.contains_key(*id))
            .chain(
                self.changes
                    .iter()
                    .filter_map(|(id, p)| p.as_ref().map(|p| (id, p))),
            )
            .filter(|(_, p)| p.route_id.as_deref() == Some(route))
            .filter(|(_, p)| p.stop_ids.iter().any(|s| cut.contains(&s.as_str())))
            .map(|(id, p)| (id.clone(), p.clone()))
            .collect();
        if active.is_empty() {
            return Err(format!(
                "No active pattern of route {route} calls at the cut stops."
            ));
        }
        let mut sections = 0;
        for (id, pattern) in active {
            self.changes.insert(id.clone(), None);
            let mut start = 0;
            for end in 0..=pattern.stop_ids.len() {
                if end < pattern.stop_ids.len() && !cut.contains(&pattern.stop_ids[end].as_str()) {
                    continue;
                }
                if end - start >= 2 {
                    let mut part = pattern.clone();
                    part.stop_ids = pattern.stop_ids[start..end].to_vec();
                    part.segment_minutes = pattern.segment_minutes[start..end - 1].to_vec();
                    part.dwell_minutes = pattern.dwell_minutes[start..end].to_vec();
                    part.pickup_type = pattern.pickup_type[start..end].to_vec();
                    part.drop_off_type = pattern.drop_off_type[start..end].to_vec();
                    part.shape_id = None;
                    part.geometry = None;
                    // shortcut: the section reuses the pattern's start times at its own first stop.
                    part.schedules = Arc::new(vec![Schedule {
                        calls: part.call_times(),
                        starts: Starts::combine(&pattern.schedules),
                    }]);
                    self.changes
                        .insert(format!("{id}#cut{sections}"), Some(part));
                    sections += 1;
                }
                start = end + 1;
            }
        }
        let pairs = self.reroute();
        Ok(json!({
            "kind": "line", "op": "cut", "id": format!("route:{route}"), "stop_ids": cut,
            "sections": sections, "before": null, "after": null,
            "effects": {"pairs": pairs},
            "assumptions": ASSUMPTIONS.iter().copied().chain(["A cut section keeps the pattern's departures at its own first stop, so its timetable is shifted by the time from the original first stop."]).collect::<Vec<_>>()
        }))
    }

    /// Scheduled Metro minutes between every pair of stations on the operating network: ride time is arrival
    /// minus departure along one pattern; changing trains at a station costs TRANSFER_MINUTES. Unreachable pairs
    /// are absent.
    pub fn metro_minutes(&self) -> HashMap<String, HashMap<String, f64>> {
        use std::cmp::Reverse;
        // Assumed wait for another train (about half a peak headway). It removes free reboarding; a dwell longer
        // than this (2.083 min at PEG-PZE on three L3 patterns) can still make a transfer slightly cheaper.
        const TRANSFER_MINUTES: f64 = 2.0;
        let station = |id: &String| {
            self.catalog
                .pack
                .stops
                .get(id)
                .and_then(|s| s.parent_id.clone())
                .unwrap_or_else(|| id.clone())
        };
        // Node (station, false) = ready to board there; (station, true) = arrived there.
        let mut edges: HashMap<(String, bool), HashMap<(String, bool), f64>> = HashMap::new();
        let active = self
            .catalog
            .pack
            .lines
            .iter()
            .filter(|(id, _)| !self.changes.contains_key(*id))
            .map(|(_, p)| p)
            .chain(self.changes.values().filter_map(Option::as_ref))
            .filter(|p| p.mode == "metro");
        for p in active {
            let calls = p.call_times();
            for i in 0..p.stop_ids.len() {
                for j in i + 1..p.stop_ids.len() {
                    let best = edges
                        .entry((station(&p.stop_ids[i]), false))
                        .or_default()
                        .entry((station(&p.stop_ids[j]), true))
                        .or_insert(f64::INFINITY);
                    *best = best.min(calls[j][0] - calls[i][1]);
                }
            }
        }
        let stations: Vec<String> = edges
            .keys()
            .chain(edges.values().flat_map(|e| e.keys()))
            .map(|n| n.0.clone())
            .collect();
        for s in &stations {
            edges
                .entry((s.clone(), true))
                .or_default()
                .insert((s.clone(), false), TRANSFER_MINUTES);
            edges.entry((s.clone(), false)).or_default();
        }
        stations
            .iter()
            .map(|from| {
                let start = (from.clone(), false);
                let mut dist: HashMap<(String, bool), f64> = HashMap::from([(start.clone(), 0.0)]);
                let mut queue = std::collections::BinaryHeap::from([(Reverse(0u64), start)]);
                while let Some((Reverse(d), at)) = queue.pop() {
                    let d = f64::from_bits(d);
                    if d > dist[&at] {
                        continue;
                    }
                    for (next, m) in &edges[&at] {
                        let t = d + m;
                        if dist.get(next).is_none_or(|&old| t < old) {
                            dist.insert(next.clone(), t);
                            queue.push((Reverse(t.to_bits()), next.clone()));
                        }
                    }
                }
                (
                    from.clone(),
                    dist.into_iter()
                        .filter(|(n, _)| n.1)
                        .map(|(n, t)| (n.0, t))
                        .collect(),
                )
            })
            .collect()
    }

    /// Route the current edited network and return the comuna pairs whose minutes changed.
    fn reroute(&mut self) -> Vec<Value> {
        let mut pairs = Vec::new();
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
        pairs
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

        if pattern.mode == "metro" {
            for (id, &[lon, lat]) in &pattern.new_stops {
                if id.is_empty()
                    || self.catalog.pack.stops.contains_key(id)
                    || !(-180.0..=180.0).contains(&lon)
                    || !(-90.0..=90.0).contains(&lat)
                {
                    return Err(
                        "A proposed Metro stop needs a new ID and valid WGS84 coordinates.".into(),
                    );
                }
                if self
                    .changes
                    .iter()
                    .filter(|(other, _)| raw["target"]["id"].as_str() != Some(other.as_str()))
                    .filter_map(|(_, change)| change.as_ref())
                    .filter(|change| change.mode == "metro")
                    .filter_map(|change| change.new_stops.get(id))
                    .any(|point| *point != [lon, lat])
                {
                    return Err(format!(
                        "Proposed Metro stop {id} has conflicting locations."
                    ));
                }
            }
        }
        let extension = before.and_then(|previous| pattern.retained_calls(previous));
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
        } else if let (Some(previous), Some(retained)) = (before, &extension) {
            let old = previous.feature("", self.catalog);
            let middle: Vec<[f64; 2]> =
                serde_json::from_value(old["geometry"]["coordinates"].clone()).unwrap();
            let points: Vec<_> = pattern.stop_ids[..retained.start]
                .iter()
                .map(|id| pattern.stop_point(id, self.catalog))
                .chain(middle)
                .chain(
                    pattern.stop_ids[retained.end..]
                        .iter()
                        .map(|id| pattern.stop_point(id, self.catalog)),
                )
                .collect();
            pattern.geometry = Some(json!({"type":"LineString","coordinates": points}));
        } else if !stops_changed {
            pattern.geometry = before.and_then(|pattern| pattern.geometry.clone());
        }
        let matching = original
            .filter(|original| pattern.same_timing(original))
            .or_else(|| before.filter(|before| pattern.same_timing(before)));
        pattern.schedules = match (matching, before) {
            (Some(matching), _) => matching.schedules.clone(),
            (None, Some(previous)) if pattern.headway_minutes == previous.headway_minutes => {
                Arc::new(match extension {
                    Some(retained) => pattern.extend_schedules(previous, retained),
                    None => pattern.retime(previous),
                })
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
