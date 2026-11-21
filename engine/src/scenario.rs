//! One entry point for any question about the city: a list of structured changes -> the full day of all persons.
//! `Scenario::run` starts from the baseline each time, applies all changes to one state, lets the workers choose
//! again once (rechoose.rs) and projects the station day once (station_access.rs, L9). The map and the MCP tool read the same result.
use crate::choice::Choice;
use crate::population::{Person, Point, Population, sample_trips};
use crate::rechoose::{Base, rechoose};
use crate::stations::StationModel;
use crate::stations::access::StationAccess;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::path::Path;

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

/// Workers by comuna pair, public-mode workers by home comuna, 48 half-hour boardings of each station.
struct State { od: Vec<f64>, public: Vec<f64>, day: Vec<Vec<f64>> }

pub struct Scenario {
    choice: Choice,
    pop: Population,
    model: StationModel,
    access: StationAccess,
    base: State,
    /// Display groups of the baseline workers (population/trips.rs), for the map.
    trips: Value,
    placed: Base,
    /// Per station and per comuna: id, name, longitude, latitude.
    stations: Vec<(String, String, f64, f64)>,
    comunas: Vec<(f64, f64)>,
    /// Baseline workers per resident of each comuna (persons / blocks.csv population).
    ratio: Vec<f64>,
}

const ASSUMPTIONS: [&str; 9] = [
    "Persons are synthetic and consistent with the Census 2024 margins. They are not microdata.",
    "The station model (L9) is fitted on the baseline. Its scale, classes and profiles do not change.",
    "Travel times are comuna to comuna and lower than survey times. The effect of a time change is an unvalidated sensitivity; the size and direction of its error are unknown.",
    "A mode change does not reach the stations yet: stations use the baseline public share of the home comuna.",
    "Open city for residents: more or fewer residents change the total of workers, with the baseline workers per resident of the comuna.",
    "Station access (L9): a walk within 1,000 m, or else one bus. A closed station does not change the travel minutes between comunas or the job choice.",
    "Students do not choose again.",
    "Trips are display groups of workers, not persons. Departure half hours come from the EOD 2012 survey. Students are not included.",
    "All values are model values, not measured values.",
];

fn r1(x: f64) -> f64 { (x * 10.0).round() / 10.0 }

impl Scenario {
    pub fn load(dir: &Path) -> Scenario {
        let choice = Choice::load(dir);
        let pop = Population::build(dir);
        let model = StationModel::new(dir);
        let access = StationAccess::load(dir);
        let st: Vec<Value> = serde_json::from_reader(std::fs::File::open(dir.join("metro_training.json")).unwrap()).unwrap();
        let s = |v: &Value| v.as_str().unwrap().to_string();
        let stations = st.iter().map(|v| (s(&v["id"]), s(&v["name"]), v["longitude"].as_f64().unwrap(), v["latitude"].as_f64().unwrap())).collect();
        let comunas = csv::Reader::from_path(dir.join("communes.csv")).unwrap().into_records()
            .map(|r| r.unwrap()).map(|r| (r[5].parse().unwrap(), r[6].parse().unwrap())).collect();
        let placed = Base::new(&choice, &pop);
        let mut sc = Scenario { placed, base: State { od: vec![], public: vec![], day: vec![] }, ratio: vec![], trips: json!(sample_trips(&pop, 300)), choice, pop, model, access, stations, comunas };
        sc.base = sc.state(&sc.pop, &sc.access.all_parent_ids());
        let n = sc.choice.n;
        let mut residents = vec![0.0; n];
        for p in &sc.pop.homes { residents[sc.ix(&p.cut)] += p.weight; }
        sc.ratio = (0..n).map(|o| sc.base.od[o * n..][..n].iter().sum::<f64>() / residents[o]).collect();
        sc
    }

    fn ix(&self, cut: &str) -> usize { self.choice.cuts.iter().position(|c| c == cut).unwrap() }

    /// The comuna of a new point: the comuna of the nearest home block.
    fn cut_at(&self, x: f64, y: f64) -> String {
        let d = |p: &Point| (p.longitude - x).powi(2) + (p.latitude - y).powi(2);
        self.pop.homes.iter().min_by(|a, b| d(a).total_cmp(&d(b))).unwrap().cut.clone()
    }

    /// `open`: the GTFS parent ids of the open stations.
    fn state(&self, pop: &Population, open: &HashSet<String>) -> State {
        let n = self.choice.n;
        let home: Vec<usize> = pop.homes.iter().map(|p| self.ix(&p.cut)).collect();
        let work: Vec<usize> = pop.jobs.iter().map(|p| self.ix(&p.cut)).collect();
        let share: Vec<f64> = self.choice.cuts.iter().map(|c| self.model.public_share(c)).collect();
        let (mut od, mut public) = (vec![0.0; n * n], vec![0.0; n]);
        let (mut h, mut j) = (vec![0.0; pop.homes.len()], vec![0.0; pop.jobs.len()]);
        for p in pop.persons.iter().filter(|p| !p.student) {
            let o = home[p.home as usize];
            od[o * n + work[p.dest as usize]] += 1.0;
            public[o] += (p.mode == 2) as u8 as f64;
            h[p.home as usize] += share[o];
            j[p.dest as usize] += share[o];
        }
        let pts = |v: &[Point], m: &[f64]| v.iter().zip(m).map(|(p, &x)| (p.longitude, p.latitude, x)).collect::<Vec<_>>();
        let day = self.access.project_points(&pts(&pop.homes, &h), &pts(&pop.jobs, &j), open).into_iter().map(|x| x.2).collect();
        State { od, public, day }
    }

    /// `changes`: JSON objects of the shape of `Change`, or `{"kind":"station","target":{"type":"station","id":<GTFS parent id or name>},
    /// "op":"set","value":0 or 1}` (0 = closed, 1 = open). A kind that the engine does not model (for example "event", a new
    /// station at a point) goes to `not_modeled`, unchanged, and the other changes run.
    pub fn run(&self, changes: &[Value]) -> Value {
        let t = std::time::Instant::now();
        let (c, n) = (&self.choice, self.choice.n);
        let copy = |v: &[Point]| v.iter().map(|p| Point { id: p.id.clone(), cut: p.cut.clone(), ..*p }).collect::<Vec<_>>();
        let mut pop = Population { homes: copy(&self.pop.homes), jobs: copy(&self.pop.jobs), schools: vec![], persons: self.pop.persons.clone() };
        let mut minutes = c.minutes.clone();
        let (mut applied, mut not_modeled) = (vec![], vec![]);
        let mut open = self.access.all_parent_ids();

        for raw in changes {
            let kind = raw["kind"].as_str().unwrap_or("");
            let key = raw["target"]["id"].as_str().unwrap_or("");
            if let Some((id, name, ..)) = self.stations.iter().find(|s| kind == "station" && (s.0 == key || s.1 == key.to_uppercase())) {
                let closed = raw["value"].as_f64() == Some(0.0);
                if closed { open.remove(id); } else { open.insert(id.clone()); }
                let mut v = raw.clone();
                v["understood"] = json!({ "station": name, "state": if closed { "closed" } else { "open" } });
                applied.push(v);
                continue;
            }
            if !["jobs", "residents", "minutes"].contains(&kind) {
                let mut v = raw.clone();
                v["reason"] = json!(match kind {
                    "station" => "Only existing stations can be closed or reopened. The engine does not model new stations or lines.",
                    "event" => "The engine does not model events yet. It models one usual work day.",
                    _ => "The engine does not know this kind of change.",
                });
                not_modeled.push(v);
                continue;
            }
            let ch = &serde_json::from_value::<Change>(raw.clone()).unwrap();
            let op = |x: f64| match ch.op { Op::Add => x + ch.value, Op::Multiply => x * ch.value, Op::Set => ch.value }.max(0.0);
            let mut note = serde_json::to_value(ch).unwrap();
            // The points of the target (indices into pop.jobs or pop.homes) and its comuna. A point target is a new point.
            let points = |v: &mut Vec<Point>| -> (Vec<usize>, String) {
                match &ch.target {
                    Target::Block { id } => { let i = v.iter().position(|p| p.id == *id).unwrap(); (vec![i], v[i].cut.clone()) }
                    Target::Comuna { id } => ((0..v.len()).filter(|&i| v[i].cut == *id).collect(), id.clone()),
                    Target::Point { longitude, latitude } => {
                        let cut = self.cut_at(*longitude, *latitude);
                        v.push(Point { id: format!("new{}", v.len()), cut: cut.clone(), longitude: *longitude, latitude: *latitude, weight: 0.0 });
                        (vec![v.len() - 1], cut)
                    }
                    Target::Pair { .. } => unreachable!(),
                }
            };
            // New weights of the points: a comuna keeps the proportions of its points.
            let scale = |v: &mut [Point], ids: &[usize]| -> (f64, f64) {
                let before: f64 = ids.iter().map(|&i| v[i].weight).sum();
                let after = op(before);
                for &i in ids { v[i].weight = if ids.len() == 1 { after } else { v[i].weight * after / before }; }
                (before, after)
            };
            match ch.kind {
                Kind::Jobs => {
                    let (ids, cut) = points(&mut pop.jobs);
                    let (before, after) = scale(&mut pop.jobs, &ids);
                    note["understood"] = json!({ "comuna": c.names[self.ix(&cut)], "jobs_before": r1(before), "jobs_after": r1(after) });
                }
                Kind::Residents => {
                    let (ids, cut) = points(&mut pop.homes);
                    let (before, after) = scale(&mut pop.homes, &ids);
                    let k = ((after - before) * self.ratio[self.ix(&cut)]).round();
                    let workers = if k > 0.0 {
                        // New workers on the home points in proportion to the new weights. u32::MAX = no job yet (rechoose.rs).
                        let (mut cum, mut done) = (0.0, 0.0);
                        for &i in &ids {
                            cum += k * pop.homes[i].weight / after;
                            let q = cum.round() - done;
                            done += q;
                            pop.persons.extend((0..q as u32).map(|_| Person { home: i as u32, dest: u32::MAX, student: false, mode: 7 }));
                        }
                        k
                    } else {
                        // Workers leave, evenly over the workers that live on the target.
                        let mut hit = vec![false; pop.homes.len()];
                        for &i in &ids { hit[i] = true; }
                        let live: Vec<usize> = (0..pop.persons.len()).filter(|&i| !pop.persons[i].student && hit[pop.persons[i].home as usize]).collect();
                        let gone = (-k as usize).min(live.len());
                        for j in 0..gone { pop.persons[live[j * live.len() / gone]].home = u32::MAX; }
                        pop.persons.retain(|p| p.home != u32::MAX);
                        -(gone as f64)
                    };
                    note["understood"] = json!({ "comuna": c.names[self.ix(&cut)], "residents_before": r1(before), "residents_after": r1(after), "workers_added": workers });
                }
                Kind::Minutes => {
                    let cells: Vec<usize> = match &ch.target {
                        Target::Pair { origin, destination } => vec![self.ix(origin) * n + self.ix(destination)],
                        Target::Comuna { id } => { let q = self.ix(id); (0..n).filter(|&o| o != q).flat_map(|o| [o * n + q, q * n + o]).collect() }
                        _ => unreachable!(),
                    };
                    let before: f64 = cells.iter().map(|&i| minutes[i]).sum();
                    for &i in &cells { minutes[i] = op(minutes[i]).max(1.0); } // a trip is 1 minute or more
                    let after: f64 = cells.iter().map(|&i| minutes[i]).sum();
                    note["understood"] = json!({ "pairs": cells.len(), "mean_minutes_before": r1(before / cells.len() as f64), "mean_minutes_after": r1(after / cells.len() as f64) });
                }
            }
            applied.push(note);
        }

        let weights: Vec<f64> = pop.jobs.iter().map(|p| p.weight).collect();
        let moved = rechoose(c, &mut pop, &weights, &minutes, &self.placed);
        let (b, s) = (&self.base, &self.state(&pop, &open));

        let daily = |d: &State, i: usize| d.day[i].iter().sum::<f64>();
        let mut order: Vec<usize> = (0..self.stations.len()).collect();
        order.sort_by(|&x, &y| (daily(s, y) - daily(b, y)).abs().total_cmp(&(daily(s, x) - daily(b, x)).abs()));
        let stations: Vec<Value> = self.stations.iter().enumerate().map(|(i, (id, name, x, y))| {
            let mut v = json!({ "id": id, "name": name, "longitude": x, "latitude": y,
                "base_daily": r1(daily(b, i)), "scenario_daily": r1(daily(s, i)), "delta_daily": r1(daily(s, i) - daily(b, i)) });
            if order[..12].contains(&i) { v["delta"] = (0..48).map(|t| r1(s.day[i][t] - b.day[i][t])).collect(); }
            v
        }).collect();
        let top = |ids: &mut dyn Iterator<Item = &usize>, sign: f64| -> Vec<Value> {
            ids.map(|&i| (i, r1(daily(s, i) - daily(b, i)))).filter(|x| x.1 * sign > 0.0).take(5)
                .map(|(i, d)| json!({ "name": self.stations[i].1, "delta_daily": d })).collect()
        };
        order.sort_by(|&x, &y| (daily(s, y) - daily(b, y)).total_cmp(&(daily(s, x) - daily(b, x))));
        let (gains, losses) = (top(&mut order.iter(), 1.0), top(&mut order.iter().rev(), -1.0));

        let zones: Vec<Value> = (0..n).map(|z| {
            let out = |d: &State| d.od[z * n..][..n].iter().sum::<f64>();
            let inb = |d: &State| (0..n).map(|o| d.od[o * n + z]).sum::<f64>();
            let share = |d: &State| (d.public[z] / out(d) * 1000.0).round() / 1000.0;
            json!({ "id": c.cuts[z], "name": c.names[z], "longitude": self.comunas[z].0, "latitude": self.comunas[z].1,
                "residents_working_base": out(b), "residents_working_scenario": out(s), "jobs_base": inb(b), "jobs_scenario": inb(s),
                "public_share_base": share(b), "public_share_scenario": share(s) })
        }).collect();
        let mut flows: Vec<usize> = (0..n * n).filter(|&i| (s.od[i] - b.od[i]).abs() >= 1.0).collect();
        flows.sort_by(|&x, &y| (s.od[y] - b.od[y]).abs().total_cmp(&(s.od[x] - b.od[x]).abs()));
        let flows: Vec<Value> = flows.iter().take(200)
            .map(|&i| json!({ "origin": c.cuts[i / n], "destination": c.cuts[i % n], "base": b.od[i], "scenario": s.od[i] })).collect();

        let total = |d: &State| (0..self.stations.len()).map(|i| daily(d, i)).sum::<f64>();
        json!({
            "applied": applied,
            "not_modeled": not_modeled,
            "summary": {
                "workers_base": b.od.iter().sum::<f64>(), "workers_scenario": s.od.iter().sum::<f64>(), "workers_moved": moved,
                "workers_new": s.od.iter().sum::<f64>() - b.od.iter().sum::<f64>(),
                "boardings_base": r1(total(b)), "boardings_scenario": r1(total(s)),
                "top_station_gains": gains, "top_station_losses": losses,
            },
            "stations": stations, "zones": zones, "flows": flows,
            "trips_base": self.trips, "trips_scenario": sample_trips(&pop, 300),
            "assumptions": ASSUMPTIONS,
            "elapsed_ms": t.elapsed().as_millis() as u64,
        })
    }
}
