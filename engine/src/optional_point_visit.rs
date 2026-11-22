//! Compare supplied point options for one worker with a fixed home and job.
use crate::population::Population;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::optional_visit::{VisitOption, choose_visit};

#[derive(Deserialize)]
struct Request {
    person_index: usize,
    home_block_id: String,
    job_block_id: String,
    baseline: Limits,
    scenario: Limits,
    skip_preference: i32,
    options: Vec<PointOption>,
}

#[derive(Deserialize, Serialize)]
struct Limits {
    available_minutes: u32,
    budget_clp: u32,
}

#[derive(Deserialize)]
struct PointOption {
    place_id: String,
    mode: String,
    preference: i32,
    baseline: Quote,
    scenario: Option<Quote>,
}

#[derive(Deserialize, Serialize)]
struct Quote {
    total_minutes: u32,
    cost_clp: u32,
    available: bool,
    route_source: String,
    cost_source: String,
    access_source: String,
}

impl Quote {
    fn has_sources(&self) -> bool {
        [&self.route_source, &self.cost_source, &self.access_source]
            .into_iter()
            .all(|source| !source.trim().is_empty())
    }
}

#[derive(Deserialize)]
struct ActivityPoint {
    osm_id: String,
    name: String,
}

fn anchor(pop: &Population, index: usize) -> Option<(&str, &str, i8)> {
    let worker = pop.persons.get(index).filter(|person| !person.student)?;
    let home = pop.homes.get(worker.home as usize)?;
    let job = pop.jobs.get(worker.dest as usize)?;
    Some((&home.id, &job.id, worker.mode))
}

fn point_names(data: &Path, options: &[PointOption]) -> Result<HashMap<String, String>, String> {
    let ids: HashSet<_> = options
        .iter()
        .map(|option| option.place_id.as_str())
        .collect();
    let mut catalog = csv::Reader::from_path(data.join("activity_points.csv"))
        .map_err(|error| format!("Cannot read packed activity points: {error}"))?;
    let mut names = HashMap::new();
    for point in catalog.deserialize::<ActivityPoint>() {
        let point =
            point.map_err(|error| format!("Cannot read a packed activity point: {error}"))?;
        if ids.contains(point.osm_id.as_str()) {
            names.insert(point.osm_id, point.name);
        }
    }
    if let Some(option) = options
        .iter()
        .find(|option| !names.contains_key(&option.place_id))
    {
        return Err(format!(
            "Unknown activity point OSM ID: {}.",
            option.place_id
        ));
    }
    Ok(names)
}

fn decide(
    request: &Request,
    names: &HashMap<String, String>,
    limits: &Limits,
    scenario: bool,
) -> Value {
    let options: Vec<_> = request
        .options
        .iter()
        .map(|option| {
            let quote = if scenario {
                option.scenario.as_ref().unwrap_or(&option.baseline)
            } else {
                &option.baseline
            };
            VisitOption {
                choice: (option, quote),
                total_minutes: quote.total_minutes,
                cost_clp: quote.cost_clp,
                available: quote.available,
                preference: option.preference,
            }
        })
        .collect();
    match choose_visit(
        &options,
        limits.available_minutes,
        limits.budget_clp,
        request.skip_preference,
    ) {
        Some((option, quote)) => json!({
            "place_id": option.place_id,
            "name": names[&option.place_id],
            "mode": option.mode,
            "quote": quote,
        }),
        None => Value::Null,
    }
}

pub(super) fn compare(
    data: &Path,
    before: &Population,
    after: &Population,
    resident_stock_changed: bool,
    raw: &Value,
) -> Result<Value, String> {
    if resident_stock_changed {
        return Err(
            "Point visits require the same worker index before and after resident edits.".into(),
        );
    }
    let request: Request = serde_json::from_value(raw.clone())
        .map_err(|error| format!("Invalid optional point visit: {error}"))?;
    if request.options.iter().any(|option| {
        !option.baseline.has_sources()
            || option
                .scenario
                .as_ref()
                .is_some_and(|quote| !quote.has_sources())
    }) {
        return Err(
            "Point visit quotes need route, cost and access sources or assumptions.".into(),
        );
    }
    let expected = (
        request.home_block_id.as_str(),
        request.job_block_id.as_str(),
    );
    let anchors = [before, after].map(|pop| anchor(pop, request.person_index));
    let [
        Some((home_before, job_before, mode_before)),
        Some((home_after, job_after, mode_after)),
    ] = anchors
    else {
        return Err(
            "Point visits require an indexed worker with a home and job in both cases.".into(),
        );
    };
    if (home_before, job_before) != expected || (home_after, job_after) != expected {
        return Err(
            "The worker's packed home or job does not match the requested fixed anchors.".into(),
        );
    }
    let names = point_names(data, &request.options)?;
    Ok(json!({
        "person_index": request.person_index,
        "home_block_id": request.home_block_id,
        "job_block_id": request.job_block_id,
        "baseline": {
            "limits": request.baseline,
            "work_mode": mode_before,
            "visit": decide(&request, &names, &request.baseline, false),
        },
        "scenario": {
            "limits": request.scenario,
            "work_mode": mode_after,
            "visit": decide(&request, &names, &request.scenario, true),
        },
        "basis": "Supplied visit feasibility, costs and preferences for a synthetic worker. Time and budget limits are scenario inputs."
    }))
}
