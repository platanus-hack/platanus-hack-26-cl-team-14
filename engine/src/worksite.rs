//! Keep scenario worksite jobs distinct from the census-block stock.
use crate::choice::Choice;
use crate::population::{Point, Population};
use santiago_scenario_core::roads::RoadEdits;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};

const ID_PREFIX: &str = "worksite:";

pub fn is_worksite(id: &str) -> bool {
    id.starts_with(ID_PREFIX)
}

pub struct Worksite {
    job: usize,
    building_id: String,
    name: String,
}

impl Worksite {
    pub fn add(raw: &Value, edits: &[Value], jobs: &mut Vec<Point>) -> Result<Self, &'static str> {
        let target = &raw["target"];
        let building_id = target["building_id"]
            .as_str()
            .ok_or("A worksite needs a placed building.")?;
        let placed = edits
            .iter()
            .rev()
            .find(|edit| edit["kind"] == "building" && edit["id"] == building_id)
            .filter(|edit| matches!(edit["op"].as_str(), Some("add" | "move")))
            .ok_or("The worksite building was not placed.")?;
        let cut = placed["after"]["properties"]["cut"]
            .as_str()
            .ok_or("The worksite has no destination comuna.")?;
        let longitude = target["longitude"]
            .as_f64()
            .filter(|x| (-180.0..=180.0).contains(x))
            .ok_or("Use a WGS84 longitude.")?;
        let latitude = target["latitude"]
            .as_f64()
            .filter(|x| (-90.0..=90.0).contains(x))
            .ok_or("Use a WGS84 latitude.")?;
        let value = raw["value"]
            .as_u64()
            .filter(|&x| x > 0 && x <= u32::MAX as u64)
            .ok_or("A worksite needs a positive whole number of jobs.")?;
        let name = target["name"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or("Name the worksite.")?;
        let id = format!("{ID_PREFIX}{building_id}");
        if raw["op"] != "add" || jobs.iter().any(|p| p.id == id) {
            return Err("Add the jobs for a placed worksite once.");
        }
        let job = jobs.len();
        jobs.push(Point {
            id,
            cut: cut.to_owned(),
            longitude,
            latitude,
            weight: value as f64,
        });
        Ok(Self {
            job,
            building_id: building_id.to_owned(),
            name: name.to_owned(),
        })
    }

    pub fn receipt(&self, pop: &Population) -> Value {
        let site = &pop.jobs[self.job];
        json!({"worksite_id":site.id, "building_id":self.building_id,
            "destination_cut":site.cut, "jobs_before":0, "jobs_after":site.weight})
    }
}

fn mode(code: i8) -> &'static str {
    match code {
        1 => "car",
        2 => "public",
        3 => "walk",
        4 => "bicycle",
        5 => "motorcycle",
        6 => "boat",
        7 => "other",
        _ => "unknown",
    }
}

const BREAKS: [f64; 5] = [15., 30., 45., 60., 90.];

pub fn results(
    worksites: &[Worksite],
    pop: &Population,
    choice: &Choice,
    roads: &RoadEdits<'_>,
    public_minutes: Option<(&[f64], &[bool])>,
) -> Vec<Value> {
    if worksites.is_empty() {
        return Vec::new();
    }
    let mut counts: HashMap<_, BTreeMap<(&str, i8), usize>> = worksites
        .iter()
        .map(|site| (site.job, BTreeMap::new()))
        .collect();
    for worker in pop.persons.iter().filter(|worker| !worker.student) {
        if let Some(site) = counts.get_mut(&(worker.dest as usize)) {
            *site
                .entry((&pop.homes[worker.home as usize].cut, worker.mode))
                .or_default() += 1;
        }
    }
    let car_times = counts
        .values()
        .any(|rows| rows.keys().any(|&(_, mode)| mode == 1))
        .then(|| roads.comuna_minutes())
        .flatten();
    worksites.iter().map(|site| {
        let job = &pop.jobs[site.job];
        let dest = choice.cuts.iter().position(|cut| *cut == job.cut).unwrap();
        let mut modes: BTreeMap<&str, (usize, usize, [usize; 6])> = BTreeMap::new();
        let mut outside = 0;
        let mut assigned = 0;
        let rows: Vec<_> = counts[&site.job].iter().map(|(&(origin, code), &workers)| {
            let outside_comuna = origin != job.cut;
            let mode = mode(code);
            let minutes = match (code, outside_comuna) {
                (1, true) => car_times.and_then(|(cuts, minutes)| {
                    let o = cuts.iter().position(|cut| cut == origin)?;
                    let d = cuts.iter().position(|cut| *cut == job.cut)?;
                    let value = minutes[o * cuts.len() + d];
                    value.is_finite().then_some(value)
                }),
                (2, true) => public_minutes.and_then(|(minutes, unreachable)| {
                    let o = choice.cuts.iter().position(|cut| cut == origin)?;
                    let pair = o * choice.n + dest;
                    (choice.gtfs_minutes[pair] && !unreachable[pair]).then_some(minutes[pair])
                }),
                _ => None,
            };
            assigned += workers;
            if outside_comuna {
                outside += workers;
                let (n, covered, bins) = modes.entry(mode).or_default();
                *n += workers;
                if let Some(minutes) = minutes {
                    *covered += workers;
                    bins[BREAKS.partition_point(|edge| minutes >= *edge)] += workers;
                }
            }
            json!({"origin_cut":origin,"mode":mode,"workers":workers,"minutes":minutes,
                "residence": if outside_comuna {"outside"} else {"local"}})
        }).collect();
        let by_mode: Vec<_> = modes.into_iter().map(|(mode, (workers, timed_workers, counts))| {
            let bins = counts.into_iter().enumerate().map(|(i, count)| json!({
                "min_inclusive": i.checked_sub(1).map(|j| BREAKS[j]),
                "max_exclusive": BREAKS.get(i), "workers": count,
            })).collect::<Vec<_>>();
            json!({"mode":mode,"workers":workers,"timed_workers":timed_workers,
                "missing_workers":workers-timed_workers,
                "share_percent":100.0 * workers as f64 / outside as f64,
                "bins": (timed_workers > 0).then_some(bins)})
        }).collect();
        json!({
            "id":job.id,"building_id":site.building_id,"name":site.name,"destination_cut":job.cut,
            "longitude":job.longitude,"latitude":job.latitude,"job_stock":job.weight,
            "assigned_workers":assigned,"outside_workers":outside,"outside_by_mode":by_mode,"rows":rows,
            "unit":"modeled workers",
            "scope":"A typical weekday. Counts follow synthetic workers assigned to this exact scenario worksite. Outside means the modeled home comuna differs from the placed building's primary census-block comuna.",
            "time_unit":"minutes",
            "time_scope":"Travel times are comuna-pair proxies for the same assigned workers. Car uses the scenario road graph at free flow between comuna anchors. Public uses the scenario's scheduled-transit pair minutes for the morning routing reference (GTFS baseline coverage only, no imputed distance-fit pairs or timed transit scenarios). The public mode includes the broad Census public category. Other modes and local journeys have no time in this result. These are not door-to-door times to the building.",
        })
    }).collect()
}
