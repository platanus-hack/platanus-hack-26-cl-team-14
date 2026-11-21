//! One entry point for any question about the city: a list of structured changes -> the full day of all persons.
//! `Scenario::run` starts from the baseline each time, applies all changes to one state, lets the workers choose
//! again once (rechoose.rs) and projects the station day once (station_access.rs, L9). The map and the MCP tool read the same result.
use crate::choice::Choice;
use crate::population::{FineModes, NEW_OTHER_MODE, Person, Point, Population, TripGroup, TripPlan, trip_deltas};
use crate::rechoose::{Base, rechoose};
use crate::stations::access::{StationAccess, planned::PlannedStations};
use santiago_scenario_core::{buildings::Buildings, editable_transit::LineCatalog, roads::RoadGraph};
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

#[path = "car_ownership.rs"]
mod car_ownership;
use car_ownership::CarOwnership;

#[path = "building_permits.rs"]
mod building_permits;
#[path = "attendance.rs"]
mod attendance;
#[path = "scenario_stations.rs"]
mod station_changes;
use station_changes::Network;

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Kind { Jobs, Residents, Minutes }

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Target {
    Block { id: String },
    Comuna { id: String },
    Point { longitude: f64, latitude: f64 },
    /// Minutes only. One direction: origin -> destination.
    Pair { origin: String, destination: String },
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Op { Add, Multiply, Set }

#[derive(Deserialize, Serialize, Clone)]
pub struct Change { pub kind: Kind, pub target: Target, pub value: f64, pub op: Op }

/// Workers by comuna pair, public/car/other workers by home comuna, 48 half-hour boardings of each station.
struct State { od: Vec<f64>, modes: Vec<[u32; 3]>, day: Vec<Vec<f64>> }

pub struct Scenario {
    data_dir: PathBuf,
    choice: Choice,
    car_ownership: Option<CarOwnership>,
    buildings: Option<Buildings>,
    roads: RoadGraph,
    lines: OnceLock<Option<LineCatalog>>,
    pop: Population,
    /// Public-transport share of the commuters of each home comuna (mode 2 of mode_share.csv), as the station fit (L2).
    share: Vec<f64>,
    access: StationAccess,
    planned: PlannedStations,
    base: State,
    /// Display groups of the baseline workers (population/trips.rs), for the map.
    trips: Vec<TripGroup>,
    plan: TripPlan,
    fine_modes: FineModes,
    placed: Base,
    /// Per station and per comuna: id, name, longitude, latitude.
    stations: Vec<(String, String, f64, f64)>,
    comunas: Vec<(f64, f64)>,
    /// Baseline workers per resident of each comuna (persons / blocks.csv population).
    ratio: Vec<f64>,
}

const ASSUMPTIONS: [&str; 10] = [
    "Persons are synthetic and consistent with the Census 2024 margins. They are not microdata.",
    "The station model (L9) is fitted on the baseline. Its scale, classes and profiles do not change.",
    "Travel times are comuna to comuna and lower than survey times. The effect of a time change is an unvalidated sensitivity; the size and direction of its error are unknown.",
    "A mode change does not reach the stations yet: stations use the baseline public share of the home comuna.",
    "Open city for residents: more or fewer residents change the total of workers, with the baseline workers per resident of the comuna.",
    "Station access (L9): a walk within 1,000 m, or else one bus. A closed station does not change the travel minutes between comunas or the job choice.",
    "Students do not choose again.",
    "Trips are display groups of workers, not persons. Departure half hours come from the EOD 2012 survey. Students are not included.",
    "The Census sets walking and cycling totals inside each comuna pair. The 2012 survey distance rule allocates workers within those totals. The engine has no measured response for cycling uptake after a new cycleway.",
    "All values are model values, not measured values.",
];

fn r1(x: f64) -> f64 { (x * 10.0).round() / 10.0 }
fn r6(x: f64) -> i64 { (x * 1e6).round() as i64 }
/// Metres from a point to (longitude, latitude), with the flat metric of the point-target rule.
fn dist_m(p: &Point, longitude: f64, latitude: f64) -> f64 { ((p.longitude - longitude) * 0.835).hypot(p.latitude - latitude) * 111_320.0 }
/// A station name without accents, in upper case: "Ñuñoa" = "NUNOA".
fn plain(s: &str) -> String {
    s.trim().to_uppercase().chars().map(|c| match c { 'Á' => 'A', 'É' => 'E', 'Í' => 'I', 'Ó' => 'O', 'Ú' | 'Ü' => 'U', 'Ñ' => 'N', c => c }).collect()
}

impl Scenario {
    pub fn load(dir: &Path) -> Scenario {
        let choice = Choice::load(dir);
        let car_ownership = dir.join("eod_car_mode.csv").is_file().then(|| CarOwnership::load(dir, &choice.cuts));
        let buildings = (dir.join("building_permits.csv").is_file() && dir.join("site_inputs.csv").is_file()).then(|| Buildings::load(dir));
        let pop = Population::build(dir);
        let mut modes = vec![(0.0, 0.0); choice.n];
        for r in csv::Reader::from_path(dir.join("mode_share.csv")).unwrap().into_records().map(|r| r.unwrap()) {
            if let Some(i) = choice.cuts.iter().position(|c| *c == r[0]) {
                let x: f64 = r[2].parse().unwrap();
                modes[i].1 += x;
                if &r[1] == "2" { modes[i].0 += x; }
            }
        }
        let share = modes.iter().map(|m| m.0 / m.1).collect();
        let plan = TripPlan::new(&pop, 2000);
        let layer = std::env::var("SANTIAGO_STATION_JOB_WEIGHTS");
        let access = StationAccess::load_layer(dir, layer.as_deref().unwrap_or("l9"));
        // Older data packs can still run the current-network and user-defined point scenarios.
        let planned = if dir.join("metro_planned.geojson").exists() { PlannedStations::load(dir) } else { PlannedStations::default() };
        let st: Vec<Value> = serde_json::from_reader(std::fs::File::open(dir.join("metro_training.json")).unwrap()).unwrap();
        let s = |v: &Value| v.as_str().unwrap().to_string();
        let stations = st.iter().map(|v| (s(&v["id"]), s(&v["name"]), v["longitude"].as_f64().unwrap(), v["latitude"].as_f64().unwrap())).collect();
        let comunas = csv::Reader::from_path(dir.join("communes.csv")).unwrap().into_records()
            .map(|r| r.unwrap()).map(|r| (r[5].parse().unwrap(), r[6].parse().unwrap())).collect();
        let placed = Base::new(&choice, &pop);
        let mut sc = Scenario { data_dir: dir.to_path_buf(), placed, base: State { od: vec![], modes: vec![], day: vec![] }, ratio: vec![], trips: plan.sample(&pop), plan, fine_modes: FineModes::from_baseline(&pop), roads: RoadGraph::load(dir), lines: OnceLock::new(), choice, car_ownership, buildings, pop, share, access, planned, stations, comunas };
        sc.base = sc.state(&sc.pop, &Network::new(&sc.access));
        let n = sc.choice.n;
        let mut residents = vec![0.0; n];
        for p in &sc.pop.homes { residents[sc.ix(&p.cut)] += p.weight; }
        sc.ratio = (0..n).map(|o| sc.base.od[o * n..][..n].iter().sum::<f64>() / residents[o]).collect();
        sc
    }

    // Allocate whole workers once from the final resident stocks. Carry fractional workers
    // between home blocks in the same comuna. Keep baseline workers when no stock changed.
    fn apply_net_residents(&self, pop: &mut Population) {
        // Baseline persons already reference the original indices. New homes have no workers yet.
        pop.homes[self.pop.homes.len()..].sort_by(|a, b| {
            a.longitude.total_cmp(&b.longitude).then(a.latitude.total_cmp(&b.latitude)).then(a.id.cmp(&b.id))
        });
        let mut cumulative = vec![0.0_f64; self.choice.n];
        let mut deltas = vec![0_i64; pop.homes.len()];
        let mut by_cut = vec![[Vec::new(), Vec::new()]; self.choice.n];
        for (home, point) in pop.homes.iter().enumerate() {
            let before = self.pop.homes.get(home).map_or(0.0, |p| p.weight);
            if point.weight == before { continue; }
            let cut = self.ix(&point.cut);
            by_cut[cut][usize::from(point.weight > before)].push(home);
            let previous = cumulative[cut];
            cumulative[cut] += (point.weight - before) * self.ratio[cut];
            deltas[home] = (cumulative[cut].round() - previous.round()) as i64;
        }
        let mut counts = vec![0_u64; deltas.len()];
        for person in pop.persons.iter().filter(|p| !p.student) {
            counts[person.home as usize] += 1;
        }
        // Allocate gains and losses on their respective blocks. A home can retain at
        // most one worker per final resident, and it can lose at most its current workers.
        for groups in by_cut {
            for (direction, homes) in groups.into_iter().enumerate() {
                let gaining = direction == 1;
                let sign = if gaining { 1 } else { -1 };
                let bounds = |h: usize| {
                    let residents = pop.homes[h].weight.floor() as u64;
                    if gaining { (0, residents.saturating_sub(counts[h])) }
                    else { (counts[h].saturating_sub(residents), counts[h]) }
                };
                let (mut target, mut assigned, mut slack_up, mut slack_down) = (0_u64, 0_u64, 0_u64, 0_u64);
                for &h in &homes {
                    let requested = deltas[h].unsigned_abs();
                    let (min, max) = bounds(h);
                    let allocated = requested.clamp(min, max);
                    target += requested;
                    assigned += allocated;
                    slack_up += max - allocated;
                    slack_down += allocated - min;
                    deltas[h] = sign * allocated as i64;
                }
                let increase = target > assigned;
                let capacity = if increase { slack_up } else { slack_down };
                let correction = target.abs_diff(assigned).min(capacity);
                if correction == 0 { continue; }
                let mut available = 0_u64;
                let mut allocated = 0_u64;
                for h in homes {
                    let current = deltas[h].unsigned_abs();
                    let (min, max) = bounds(h);
                    available += if increase { max - current } else { current - min };
                    let next = (u128::from(available) * u128::from(correction) / u128::from(capacity)) as u64;
                    let adjustment = sign * (next - allocated) as i64;
                    if increase { deltas[h] += adjustment; } else { deltas[h] -= adjustment; }
                    allocated = next;
                }
            }
        }
        if deltas.iter().any(|&d| d < 0) {
            let mut seen = vec![0_u64; deltas.len()];
            pop.persons.retain(|p| {
                let h = p.home as usize;
                if p.student || deltas[h] >= 0 { return true; }
                let remove = deltas[h].unsigned_abs().min(counts[h]);
                let i = seen[h];
                seen[h] += 1;
                // Spread removals evenly among the workers at this home block.
                (i * remove).div_ceil(counts[h]) == (seen[h] * remove).div_ceil(counts[h])
            });
        }
        for (home, delta) in deltas.into_iter().enumerate() {
            pop.persons.extend((0..delta).map(|_| Person {
                home: home as u32, dest: u32::MAX, student: false, mode: NEW_OTHER_MODE,
            }));
        }
    }

    fn ix(&self, cut: &str) -> usize { self.choice.cuts.iter().position(|c| c == cut).unwrap() }

    /// The comuna of a new point: the comuna of the nearest home block.
    fn cut_at(&self, x: f64, y: f64) -> String {
        let d = |p: &Point| (p.longitude - x).powi(2) + (p.latitude - y).powi(2);
        self.pop.homes.iter().min_by(|a, b| d(a).total_cmp(&d(b))).unwrap().cut.clone()
    }

    fn state(&self, pop: &Population, network: &Network) -> State {
        let n = self.choice.n;
        let home: Vec<usize> = pop.homes.iter().map(|p| self.ix(&p.cut)).collect();
        let work: Vec<usize> = pop.jobs.iter().map(|p| self.ix(&p.cut)).collect();
        let share = &self.share;
        let (mut od, mut modes) = (vec![0.0; n * n], vec![[0; 3]; n]);
        let (mut h, mut j) = (vec![0.0; pop.homes.len()], vec![0.0; pop.jobs.len()]);
        for p in pop.persons.iter().filter(|p| !p.student) {
            let o = home[p.home as usize];
            od[o * n + work[p.dest as usize]] += 1.0;
            modes[o][match p.mode { 2 => 0, 1 => 1, _ => 2 }] += 1;
            h[p.home as usize] += share[o];
            j[p.dest as usize] += share[o];
        }
        let pts = |v: &[Point], m: &[f64]| v.iter().zip(m).map(|(p, &x)| (p.longitude, p.latitude, x)).collect::<Vec<_>>();
        self.access.weight_job_masses(pop.jobs.iter().map(|p| (p.id.as_str(), p.cut.as_str())), &mut j);
        let day = self.access.project_with_additions(&pts(&pop.homes, &h), &pts(&pop.jobs, &j), &network.open, &network.additions)
            .into_iter().map(|x| x.2).collect();
        State { od, modes, day }
    }

    fn change_car_share(&self, raw: &Value, shares: &mut BTreeMap<usize, f64>) -> Result<Value, &'static str> {
        if raw["target"]["type"] != "comuna" {
            return Err("Use a comuna target and its home CUT for household access to a car.");
        }
        let cut = raw["target"]["id"].as_str().unwrap_or("");
        let origin = self.choice.cuts.iter().position(|id| id == cut).ok_or("Unknown home comuna CUT.")?;
        let cars = self.car_ownership.as_ref().ok_or("Household car access needs eod_car_mode.csv in the data pack.")?;
        let value = raw["value"].as_f64().filter(|v| v.is_finite()).ok_or("Use a finite numeric car share or multiplier.")?;
        let before = shares.get(&origin).copied().unwrap_or(cars.base[origin]);
        let after = match raw["op"].as_str() {
            Some("set") => value,
            Some("multiply") if value >= 0.0 => before * value,
            Some("multiply") => return Err("The household car share multiplier must be zero or greater."),
            _ => return Err("Use set or multiply for the household car share."),
        };
        if !(0.0..=1.0).contains(&after) {
            return Err("The resulting share of households with access to a car must be between 0 and 1.");
        }
        shares.insert(origin, after);
        let (base_source, rates_source) = cars.source[origin];
        Ok(json!({ "comuna": self.choice.names[origin], "household_car_share_before": before, "household_car_share_after": after,
            "base_share_source": base_source, "trip_rates_source": rates_source }))
    }

    /// Apply stock, time, station access, and event changes to the same city. Return each unsupported
    /// request in `not_modeled` and continue with the other changes.
    pub fn run(&self, changes: &[Value]) -> Value {
        let t = std::time::Instant::now();
        let (c, n) = (&self.choice, self.choice.n);
        let copy = |v: &[Point]| v.iter().map(|p| Point { id: p.id.clone(), cut: p.cut.clone(), ..*p }).collect::<Vec<_>>();
        let mut pop = Population { homes: copy(&self.pop.homes), jobs: copy(&self.pop.jobs), schools: vec![], persons: self.pop.persons.clone() };
        let mut minutes = c.minutes.clone();
        let (mut applied, mut not_modeled) = (vec![], vec![]);
        let mut network = Network::new(&self.access);
        let mut building_edits = self.buildings.as_ref().map(Buildings::edits);
        let mut road_edits = self.roads.edits();
        let mut line_edits = if changes.iter().any(|v| v["kind"] == "line") {
            self.lines.get_or_init(|| self.data_dir.join("editable_transit.json").is_file()
                .then(|| LineCatalog::load(&self.data_dir))).as_ref().map(LineCatalog::edits)
        } else { None };
        // For an outage keep the last finite graph time. None means this pair had no graph baseline.
        let mut transit_reference: HashMap<usize, Option<f64>> = HashMap::new();
        let mut resident_stock_edits = 0;
        let mut city_edits = Vec::new();
        let mut car_shares = BTreeMap::new();
        let mut requests = vec![];

        // Apply derived stock changes at the parent edit’s position in the request.
        let mut pending: Vec<_> = changes.iter().rev().cloned().map(|change| (change, false)).collect();
        while let Some((change, stock_child)) = pending.pop() {
            let raw = &change;
            if raw["kind"] == "road" || raw["kind"] == "line" {
                let is_line = raw["kind"] == "line";
                let edit = if is_line {
                    line_edits.as_mut().ok_or_else(|| "Line edits need editable_transit.json.".to_owned())
                        .and_then(|edits| edits.apply(raw))
                } else { road_edits.apply(raw) };
                let mut note = raw.clone();
                match edit {
                    Ok(mut edit) => {
                        if is_line {
                            for pair in edit["effects"]["pairs"].as_array_mut().unwrap() {
                                let index = self.ix(pair["origin_cut"].as_str().unwrap()) * n
                                    + self.ix(pair["destination_cut"].as_str().unwrap());
                                let reference = transit_reference.entry(index).or_insert(pair["before_minutes"].as_f64());
                                match (*reference, pair["after_minutes"].as_f64()) {
                                    (Some(last), Some(next)) => {
                                        let before = minutes[index].max(1.0);
                                        minutes[index] += next - last;
                                        pair["choice_minutes_delta"] = json!(minutes[index].max(1.0) - before);
                                        *reference = Some(next);
                                    }
                                    _ => {
                                        pair["choice_minutes_delta"] = Value::Null;
                                        pair["choice_effect_unavailable"] = json!(if reference.is_none() {
                                            "No finite baseline transit path for this comuna pair."
                                        } else { "The edited transit graph has no path for this comuna pair." });
                                    }
                                }
                            }
                        }
                        note["understood"] = json!({"id":edit["id"],"city_edit_index":city_edits.len()});
                        applied.push(note);
                        city_edits.push(edit);
                    }
                    Err(reason) => {
                        note["reason"] = json!(reason);
                        not_modeled.push(note);
                    }
                }
                continue;
            }
            if raw["kind"] == "building" {
                let mut note = raw.clone();
                let edit = building_edits.as_mut()
                    .ok_or_else(|| "Building edits need building_permits.csv and site_inputs.csv.".to_owned())
                    .and_then(|edits| edits.apply(raw));
                match edit {
                    Ok(edit) => {
                        let blocks = edit["effects"]["blocks"].as_array().unwrap();
                        // A prior stock edit may have removed the residents/jobs this footprint would debit.
                        // Reject the whole edit before changing either end of a move.
                        let conflict = blocks.iter().any(|b| {
                            [("jobs_delta", &pop.jobs), ("residents_delta", &pop.homes)].iter().any(|(field, points)| {
                                let delta = b[*field].as_f64().unwrap_or(0.0);
                                delta < 0.0 && points.iter().find(|p| b["block_id"] == p.id)
                                    .map_or(0.0, |p| p.weight) + delta < -1e-8
                            })
                        });
                        if conflict {
                            // Restore the footprint overlay from the earlier accepted edits in this request.
                            let mut restored = self.buildings.as_ref().unwrap().edits();
                            for previous in applied.iter().filter(|v: &&Value| v["kind"] == "building") {
                                restored.apply(previous).unwrap();
                            }
                            building_edits = Some(restored);
                            note["reason"] = json!("The building would withdraw more jobs or residents than remain on its block.");
                            not_modeled.push(note);
                            continue;
                        }
                        for b in blocks {
                            let id = b["block_id"].as_str().unwrap();
                            for (field, points) in [("jobs_delta", &mut pop.jobs), ("residents_delta", &mut pop.homes)] {
                                let Some(delta) = b[field].as_f64().filter(|d| *d != 0.0) else { continue };
                                let index = points.iter().position(|p| p.id == id).unwrap_or_else(|| {
                                    points.push(Point { id: id.to_owned(), cut: b["cut"].as_str().unwrap().to_owned(),
                                        longitude: b["longitude"].as_f64().unwrap(), latitude: b["latitude"].as_f64().unwrap(), weight: 0.0 });
                                    points.len() - 1
                                });
                                let stock = &mut points[index].weight;
                                if field == "residents_delta" {
                                    resident_stock_edits += 1;
                                }
                                *stock = (*stock + delta).max(0.0);
                            }
                        }
                        note["understood"] = json!({"id":edit["id"],"city_edit_index":city_edits.len()});
                        applied.push(note);
                        city_edits.push(edit);
                    }
                    Err(reason) => {
                        note["reason"] = json!(reason);
                        not_modeled.push(note);
                    }
                }
                continue;
            }
            if raw["kind"] == "cars" {
                let mut note = raw.clone();
                match self.change_car_share(raw, &mut car_shares) {
                    Ok(understood) => {
                        note["understood"] = understood;
                        applied.push(note);
                    }
                    Err(reason) => {
                        note["reason"] = json!(reason);
                        not_modeled.push(note);
                    }
                }
                continue;
            }
            if raw["target"]["type"] == "permits" {
                let mut note = change.clone();
                let result = match (raw["kind"].as_str(), raw["op"].as_str(), raw["target"]["id"].as_str(), raw["value"].as_f64()) {
                    (Some("residents"), Some("multiply"), Some(years), Some(share)) => building_permits::expand(&self.data_dir, years, share),
                    _ => Err("Use a residents permits target with a year or year range, op multiply, and a numeric completion share.".into()),
                };
                match result {
                    Ok((residents, summary)) => {
                        note["understood"] = summary;
                        applied.push(note);
                        for (cut, count) in residents.into_iter().rev() {
                            if count > 0 {
                                pending.push((json!({"kind":"residents","target":{"type":"comuna","id":cut},"op":"add","value":count}), true));
                            }
                        }
                    }
                    Err(reason) => {
                        note["reason"] = json!(reason);
                        not_modeled.push(note);
                    }
                }
                continue;
            }
            let kind = raw["kind"].as_str().unwrap_or("");
            let key = raw["target"]["id"].as_str().unwrap_or("");
            if let Some(result) = network.apply(raw, &self.stations, &self.planned) {
                let mut note = raw.clone();
                match result {
                    Ok(understood) => {
                        note["understood"] = understood;
                        applied.push(note);
                    }
                    Err(reason) => {
                        note["reason"] = json!(reason);
                        not_modeled.push(note);
                    }
                }
                continue;
            }
            if let (true, Some(longitude), Some(latitude), Some(attendance), Some(end)) = (kind == "event" && raw["target"]["type"] == "point" && raw["op"] == "set",
                raw["target"]["longitude"].as_f64(), raw["target"]["latitude"].as_f64(), raw["value"].as_f64().filter(|&v| v >= 0.0),
                raw["assumed_end_half_hour"].as_u64().filter(|&e| e < 48)) {
                let venue_cut = self.cut_at(longitude, latitude);
                let mut v = raw.clone();
                v["understood"] = json!({ "comuna": c.names[self.ix(&venue_cut)], "attendance": attendance });
                applied.push(v);
                requests.push(attendance::AttendanceRequest { longitude, latitude, venue_cut, attendance: attendance as u32, assumed_end_half_hour: end as u8 });
                continue;
            }
            if !["jobs", "residents", "minutes"].contains(&kind) {
                let mut v = raw.clone();
                v["reason"] = json!(match kind {
                    "event" => "An event change needs a point target, op set, the attendance (0 or more) as value and an integer assumed_end_half_hour (0 to 47).",
                    _ => "The engine does not know this kind of change.",
                });
                not_modeled.push(v);
                continue;
            }
            // A block that is not in the table of its kind (for example residents at a job-only block such as the MUT).
            if raw["target"]["type"] == "block" && !(if kind == "jobs" { &pop.jobs } else { &pop.homes }).iter().any(|p| p.id == key) {
                let mut v = raw.clone();
                v["reason"] = json!(format!("Block {key} has no {kind} in the engine data. Use a point target or another block."));
                not_modeled.push(v);
                continue;
            }
            let ch = &serde_json::from_value::<Change>(raw.clone()).unwrap();
            let op = |x: f64| match ch.op { Op::Add => x + ch.value, Op::Multiply => x * ch.value, Op::Set => ch.value }.max(0.0);
            let mut note = serde_json::to_value(ch).unwrap();
            // The points of the target (indices into pop.jobs or pop.homes) and its comuna. A point target is the point at these coordinates (6 decimals), the nearest point within 500 m for a change that gives 0 on an empty point, or else a new point.
            let points = |v: &mut Vec<Point>| -> (Vec<usize>, String) {
                match &ch.target {
                    Target::Block { id } => { let i = v.iter().position(|p| p.id == *id).unwrap(); (vec![i], v[i].cut.clone()) }
                    Target::Comuna { id } => ((0..v.len()).filter(|&i| v[i].cut == *id).collect(), id.clone()),
                    Target::Point { longitude, latitude } => {
                        if let Some(i) = v.iter().position(|p| r6(p.longitude) == r6(*longitude) && r6(p.latitude) == r6(*latitude)) { return (vec![i], v[i].cut.clone()); }
                        // A change that leaves an empty point empty (remove, reduce, scale) means an existing place: the nearest point within 500 m.
                        if op(0.0) == 0.0 {
                            let d = |p: &Point| ((p.longitude - longitude) * 0.835).powi(2) + (p.latitude - latitude).powi(2);
                            if let Some(i) = (0..v.len()).filter(|&i| d(&v[i]) < (500.0f64 / 111_320.0).powi(2)).min_by(|&a, &b| d(&v[a]).total_cmp(&d(&v[b]))) { return (vec![i], v[i].cut.clone()); }
                        }
                        let cut = self.cut_at(*longitude, *latitude);
                        v.push(Point { id: format!("new{}", v.len()), cut: cut.clone(), longitude: *longitude, latitude: *latitude, weight: 0.0 });
                        (vec![v.len() - 1], cut)
                    }
                    Target::Pair { .. } => unreachable!(),
                }
            };
            // Keep current proportions. If an earlier change emptied a comuna,
            // use its original block weights to distribute the next increase.
            let scale = |v: &mut [Point], ids: &[usize], baseline: &[Point]| {
                let before: f64 = ids.iter().map(|&i| v[i].weight).sum();
                let after = op(before);
                if before == after { return Some((before, after)); }
                if ids.len() == 1 {
                    v[ids[0]].weight = after;
                } else if before > 0.0 {
                    for &i in ids { v[i].weight = v[i].weight * after / before; }
                } else if after > 0.0 {
                    let weight = |i: usize| baseline.get(i).map_or(0.0, |p| p.weight);
                    let total: f64 = ids.iter().map(|&i| weight(i)).sum();
                    if total == 0.0 { return None; }
                    for &i in ids { v[i].weight = weight(i) * after / total; }
                }
                Some((before, after))
            };
            match ch.kind {
                Kind::Jobs => {
                    let (ids, cut) = points(&mut pop.jobs);
                    let Some((before, after)) = scale(&mut pop.jobs, &ids, &self.pop.jobs) else {
                        note["reason"] = json!("The comuna has no baseline block weights for this increase.");
                        not_modeled.push(note);
                        continue;
                    };
                    note["understood"] = json!({ "comuna": c.names[self.ix(&cut)], "jobs_before": r1(before), "jobs_after": r1(after) });
                    if let Target::Point { longitude, latitude } = &ch.target { let p = &pop.jobs[ids[0]]; let new = p.id.starts_with("new"); note["understood"]["block"] = if new { json!(null) } else { json!(p.id) }; note["understood"]["block_distance_m"] = if new { json!(null) } else { json!(r1(dist_m(p, *longitude, *latitude))) }; }
                }
                Kind::Residents => {
                    let (ids, cut) = points(&mut pop.homes);
                    let Some((before, after)) = scale(&mut pop.homes, &ids, &self.pop.homes) else {
                        note["reason"] = json!("The comuna has no baseline block weights for this increase.");
                        not_modeled.push(note);
                        continue;
                    };
                    let workers = if before == after { Some(0) } else {
                        resident_stock_edits += 1;
                        None
                    };
                    note["understood"] = json!({ "comuna": c.names[self.ix(&cut)], "residents_before": r1(before), "residents_after": r1(after), "workers_added": workers });
                    if let Target::Point { longitude, latitude } = &ch.target { let p = &pop.homes[ids[0]]; let new = p.id.starts_with("new"); note["understood"]["block"] = if new { json!(null) } else { json!(p.id) }; note["understood"]["block_distance_m"] = if new { json!(null) } else { json!(r1(dist_m(p, *longitude, *latitude))) }; }
                }
                Kind::Minutes => {
                    let cells: Vec<usize> = match &ch.target {
                        Target::Pair { origin, destination } => vec![self.ix(origin) * n + self.ix(destination)],
                        Target::Comuna { id } => { let q = self.ix(id); (0..n).filter(|&o| o != q).flat_map(|o| [o * n + q, q * n + o]).collect() }
                        _ => unreachable!(),
                    };
                    let before: f64 = cells.iter().map(|&i| if transit_reference.contains_key(&i) { minutes[i].max(1.0) } else { minutes[i] }).sum();
                    for &i in &cells { minutes[i] = op(if transit_reference.contains_key(&i) { minutes[i].max(1.0) } else { minutes[i] }).max(1.0); } // a trip is 1 minute or more
                    let after: f64 = cells.iter().map(|&i| minutes[i]).sum();
                    note["understood"] = json!({ "pairs": cells.len(), "mean_minutes_before": r1(before / cells.len() as f64), "mean_minutes_after": r1(after / cells.len() as f64) });
                }
            }
            if !stock_child { applied.push(note); }
        }

        if resident_stock_edits > 0 {
            self.apply_net_residents(&mut pop);
            if resident_stock_edits == 1 {
                let net = pop.persons.len() as i64 - self.pop.persons.len() as i64;
                for note in &mut applied {
                    if let Some(workers) = note["understood"].get_mut("workers_added").filter(|v| v.is_null()) {
                        *workers = json!(net);
                    }
                }
            }
        }
        for index in transit_reference.keys() {
            minutes[*index] = minutes[*index].max(1.0);
        }
        let weights: Vec<f64> = pop.jobs.iter().map(|p| p.weight).collect();
        let mut car_choice = None;
        if let Some(cars) = &self.car_ownership {
            for (&origin, &share) in &car_shares {
                if share == cars.base[origin] { continue; }
                let choice = car_choice.get_or_insert_with(|| self.choice.clone());
                let factors = cars.factors(origin, share);
                for pair in &mut choice.mode_k[origin * n..][..n] {
                    for (weight, factor) in pair.iter_mut().zip(factors) { *weight *= factor; }
                }
            }
        }
        let moved = rechoose(car_choice.as_ref().unwrap_or(c), &mut pop, &weights, &minutes, &self.placed);
        self.fine_modes.apply(&mut pop);
        let events: Vec<Value> = requests.into_iter().map(|r| {
            json!(attendance::project_with_additions(&pop, &self.access, &network.open, r, &network.additions))
        }).collect();
        let mut assumptions = ASSUMPTIONS.map(Value::from).to_vec();
        if resident_stock_edits > 1 {
            assumptions.push(json!("Worker changes use final resident stocks per home block and the Census worker ratio for its comuna. Summary workers_new gives the net change across the request."));
        }
        if city_edits.iter().any(|edit| edit["kind"] == "line") {
            assumptions.push(json!("Exploratory station boardings after a transit line edit. The station model uses worker origins, job destinations and baseline public transport shares. Ridership on the edited service is unknown."));
        }
        if !car_shares.is_empty() {
            assumptions.push(json!("The 2012 Santiago travel survey links household access to a car with the mode used to travel to work. The model uses this survey association to estimate commuter counts by mode and home comuna."));
        }
        if !network.additions.is_empty() {
            assumptions.push(json!("New stations model walking access only. The model assigns each new station a fitted profile from its job and home catchment. New stations do not change train schedules, transfers, travel times, or job choice."));
        }
        if let Some(e) = events.first() { assumptions.extend(e["assumptions"].as_array().unwrap().iter().cloned()); }
        // The points whose jobs or residents changed, for the map: one row per block id. A new point has no id; its key is its coordinates.
        let (mut rows, mut at): (Vec<(Value, [f64; 2])>, HashMap<String, usize>) = (vec![], HashMap::new());
        for (k, v, old) in [(0, &pop.jobs, &self.pop.jobs), (1, &pop.homes, &self.pop.homes)] {
            for (i, p) in v.iter().enumerate() {
                let d = p.weight - old.get(i).map_or(0.0, |q| q.weight);
                if d.abs() <= 1e-9 { continue; }
                let id = (!p.id.starts_with("new")).then_some(&p.id);
                let key = id.cloned().unwrap_or_else(|| format!("{} {}", r6(p.longitude), r6(p.latitude)));
                let j = *at.entry(key).or_insert_with(|| {
                    rows.push((json!({ "id": id, "cut": p.cut, "longitude": p.longitude, "latitude": p.latitude }), [0.0; 2]));
                    rows.len() - 1
                });
                rows[j].1[k] += d;
            }
        }
        rows.sort_by(|a, b| (b.1[0].abs() + b.1[1].abs()).total_cmp(&(a.1[0].abs() + a.1[1].abs())));
        let blocks_total = rows.len();
        let blocks: Vec<Value> = rows.into_iter().take(300).map(|(mut v, d)| { v["jobs_delta"] = json!(d[0]); v["residents_delta"] = json!(d[1]); v }).collect();
        let (b, s) = (&self.base, &self.state(&pop, &network));

        let points: Vec<_> = self.stations.iter().map(|(id, name, x, y)| (id, name, *x, *y))
            .chain(network.additions.iter().map(|p| (&p.id, &p.name, p.longitude, p.latitude))).collect();
        let zero = [0.0; 48];
        let base_bins = |i: usize| b.day.get(i).map(Vec::as_slice).unwrap_or(&zero);
        let daily: Vec<_> = s.day.iter().enumerate().map(|(i, bins)| {
            let base: f64 = base_bins(i).iter().sum();
            let scenario: f64 = bins.iter().sum();
            (base, scenario, scenario - base)
        }).collect();
        let mut order: Vec<usize> = (0..points.len()).collect();
        order.sort_by(|&x, &y| daily[y].2.abs().total_cmp(&daily[x].2.abs()));
        let stations: Vec<Value> = points.iter().enumerate().map(|(i, (id, name, x, y))| {
            let (base, scenario, delta) = daily[i];
            let mut v = json!({ "id": id, "name": name, "longitude": x, "latitude": y,
                "base_daily": r1(base), "scenario_daily": r1(scenario), "delta_daily": r1(delta),
                "base_half_hour": base_bins(i).iter().map(|&x| r1(x)).collect::<Vec<_>>(), "scenario_half_hour": s.day[i].iter().map(|&x| r1(x)).collect::<Vec<_>>() });
            if order[..12].contains(&i) { v["delta"] = s.day[i].iter().zip(base_bins(i)).map(|(after, before)| r1(after - before)).collect(); }
            v
        }).collect();
        let top = |ids: &mut dyn Iterator<Item = &usize>, sign: f64| -> Vec<Value> {
            ids.map(|&i| (i, r1(daily[i].2))).filter(|x| x.1 * sign > 0.0).take(5)
                .map(|(i, d)| json!({ "name": points[i].1, "delta_daily": d })).collect()
        };
        order.sort_by(|&x, &y| daily[y].2.total_cmp(&daily[x].2));
        let (gains, losses) = (top(&mut order.iter(), 1.0), top(&mut order.iter().rev(), -1.0));

        let zones: Vec<Value> = (0..n).map(|z| {
            let out = |d: &State| d.od[z * n..][..n].iter().sum::<f64>();
            let inb = |d: &State| (0..n).map(|o| d.od[o * n + z]).sum::<f64>();
            let share = |d: &State| (f64::from(d.modes[z][0]) / out(d) * 1000.0).round() / 1000.0;
            json!({ "id": c.cuts[z], "name": c.names[z], "longitude": self.comunas[z].0, "latitude": self.comunas[z].1,
                "residents_working_base": out(b), "residents_working_scenario": out(s), "jobs_base": inb(b), "jobs_scenario": inb(s),
                "public_share_base": share(b), "public_share_scenario": share(s) })
        }).collect();
        let mut flows: Vec<usize> = (0..n * n).filter(|&i| (s.od[i] - b.od[i]).abs() >= 1.0).collect();
        flows.sort_by(|&x, &y| (s.od[y] - b.od[y]).abs().total_cmp(&(s.od[x] - b.od[x]).abs()));
        let flows: Vec<Value> = flows.iter().take(200)
            .map(|&i| json!({ "origin": c.cuts[i / n], "destination": c.cuts[i % n], "base": b.od[i], "scenario": s.od[i] })).collect();

        let total = |d: &State| d.day.iter().map(|bins| bins.iter().sum::<f64>()).sum::<f64>();
        let trips = self.plan.sample(&pop);
        let mut result = json!({
            "applied": applied,
            "city_edits": city_edits,
            "not_modeled": not_modeled,
            "summary": {
                "station_job_weights_file": self.access.job_weights_file(),
                "workers_base": b.od.iter().sum::<f64>(), "workers_scenario": s.od.iter().sum::<f64>(), "workers_moved": moved,
                "workers_new": s.od.iter().sum::<f64>() - b.od.iter().sum::<f64>(),
                "boardings_base": r1(total(b)), "boardings_scenario": r1(total(s)),
                "top_station_gains": gains, "top_station_losses": losses,
            },
            "stations": stations, "zones": zones, "flows": flows,
            "blocks_changed": blocks, "blocks_changed_total": blocks_total,
            "trips_delta": trip_deltas(&self.trips, &trips), "trips_base": self.trips, "trips_scenario": trips,
            "events": events,
            "assumptions": assumptions,
            "elapsed_ms": t.elapsed().as_millis() as u64,
        });
        if !car_shares.is_empty() {
            let cars = self.car_ownership.as_ref().unwrap();
            let counts = |[public, car, other]: [u32; 3]| json!({
                "public": public, "car": car, "other": other, "total": public + car + other
            });
            result["car_mode_changes"] = car_shares.iter().map(|(&origin, &share)| json!({
                "id": c.cuts[origin], "name": c.names[origin],
                "household_car_share_base": cars.base[origin], "household_car_share_scenario": share,
                "base_share_source": cars.source[origin].0, "trip_rates_source": cars.source[origin].1,
                "workers_base": counts(b.modes[origin]), "workers_scenario": counts(s.modes[origin]),
            })).collect();
        }
        result
    }
}
