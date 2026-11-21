mod city;
mod event;
mod site;
mod stations;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use city::{Coverage, Flow, Metadata, World, Zone, ZoneChange};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Arc};
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct Commune {
    cut: String,
    name: String,
    population: f64,
    households: f64,
    dwellings: f64,
    longitude: f64,
    latitude: f64,
}

#[derive(Deserialize)]
struct WorkOd {
    origin_cut: String,
    destination_cut: String,
    workers: f64,
}

#[derive(Deserialize)]
struct ModeShare {
    cut: String,
    mode: i32,
    commuters: f64,
}

#[derive(Serialize)]
struct ResidentMode {
    mode_code: i32,
    mode_name: &'static str,
    source_commuters: f64,
    share: f64,
    resident_commuters: f64,
    daily_home_departures: f64,
}

#[derive(Serialize)]
struct ModeSource {
    source_kind: &'static str,
    source: &'static str,
    comuna_cut: &'static str,
    source_total_commuters: f64,
    source_universe: &'static str,
    modeled_universe: &'static str,
    output_kind: &'static str,
    assumption: &'static str,
}

#[derive(Serialize)]
struct SiteHttpResponse {
    #[serde(flatten)]
    site: site::SiteResponse,
    resident_commuters_by_mode: Vec<ResidentMode>,
    resident_commuters_by_mode_source: ModeSource,
}

struct Core {
    latest: std::sync::Mutex<(u64, serde_json::Value)>,
    scenario_events: tokio::sync::watch::Sender<serde_json::Value>,
    baseline: World,
    pairs: Vec<(usize, usize)>,
    site: site::SiteModel,
    site_destinations: Vec<(String, f64)>,
    site_modes: Vec<ModeShare>,
    site_mode_total: f64,
    station_model: stations::StationModel,
    station_observed: HashMap<String, Vec<Option<f64>>>,
    station_observation_metadata: serde_json::Value,
    event_model: event::EventModel,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ScenarioRequest {
    Shared { changes: Vec<ZoneChange> },
    Legacy(ZoneChange),
}

#[derive(Serialize)]
struct Summary {
    zone_id: String,
    jobs_multiplier: f64,
    housing_multiplier: f64,
    total_population: f64,
    total_households: f64,
    total_workers: f64,
    selected_zone_population_delta: f64,
    selected_zone_households_delta: f64,
    selected_zone_dwellings_delta: f64,
    selected_zone_workers_delta: f64,
    selected_zone_jobs_delta: f64,
}

#[derive(Serialize)]
struct Simulation {
    #[serde(flatten)]
    world: World,
    summary: Summary,
    elapsed_ms: f64,
}

impl Core {
    fn load(directory: &Path) -> Self {
        let mut communes = csv::Reader::from_path(directory.join("communes.csv")).unwrap();
        let mut zones = Vec::new();
        for row in communes.deserialize::<Commune>() {
            let c = row.unwrap();
            zones.push(Zone {
                id: c.cut,
                name: c.name,
                longitude: c.longitude,
                latitude: c.latitude,
                population: c.population,
                households: c.households,
                dwellings: c.dwellings,
                workers: 0.0,
                jobs: 0.0,
            });
        }
        let ids: HashMap<_, _> = zones
            .iter()
            .enumerate()
            .map(|(i, z)| (z.id.clone(), i))
            .collect();
        let mut od = csv::Reader::from_path(directory.join("work_od.csv")).unwrap();
        let mut flows = Vec::new();
        let mut pairs = Vec::new();
        let mut coverage = Coverage {
            source_total_workers: 0.0,
            included_workers: 0.0,
            external_destination_workers: 0.0,
            external_destination_pairs: 0,
        };
        for row in od.deserialize::<WorkOd>() {
            let f = row.unwrap();
            let origin = ids[&f.origin_cut];
            coverage.source_total_workers += f.workers;
            // Valid CUT codes use the first two digits for region; RM is region 13.
            if !f.destination_cut.starts_with("13") {
                coverage.external_destination_workers += f.workers;
                coverage.external_destination_pairs += 1;
                continue;
            }
            let destination = ids[&f.destination_cut];
            coverage.included_workers += f.workers;
            zones[origin].workers += f.workers;
            zones[destination].jobs += f.workers;
            pairs.push((origin, destination));
            flows.push(Flow {
                origin: f.origin_cut,
                destination: f.destination_cut,
                workers: f.workers,
            });
        }
        let site_destinations = flows
            .iter()
            .filter(|flow| flow.origin == "13114")
            .map(|flow| (flow.destination.clone(), flow.workers))
            .collect();
        let observations: site::ObservationData = serde_json::from_reader(
            std::fs::File::open(directory.join("tobalaba_observations.json")).unwrap(),
        )
        .unwrap();
        let mut modes = csv::Reader::from_path(directory.join("mode_share.csv")).unwrap();
        let site_modes: Vec<ModeShare> = modes
            .deserialize::<ModeShare>()
            .map(Result::unwrap)
            .filter(|row| row.cut == "13114")
            .collect();
        let site_mode_total = site_modes.iter().map(|row| row.commuters).sum();
        let station_model = stations::StationModel::new(directory);
        let metro_observations: serde_json::Value = serde_json::from_reader(
            std::fs::File::open(directory.join("metro_observations.json")).unwrap(),
        )
        .unwrap();
        let station_observed = metro_observations["stations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                (
                    row["station"]["id"].as_str().unwrap().to_owned(),
                    serde_json::from_value(row["half_hour_bins"]["LABORAL"].clone()).unwrap(),
                )
            })
            .collect();
        let station_observation_metadata = serde_json::json!({
            "source_file": "metro_observations.json", "day_type": "LABORAL",
            "period": metro_observations["period"]["LABORAL"],
            "timezone": metro_observations["timezone"], "unit": metro_observations["unit"],
            "bin_definition": metro_observations["bin_definition"]
        });
        Self {
            baseline: World {
                metadata: Metadata {
                    model: "empirical-od-shared-relative-capacity-v2",
                    state: "observed_baseline",
                    data_vintages: HashMap::from([
                        ("communes", "INE Census 2024 commune aggregates".to_owned()),
                        (
                            "work_od",
                            "INE Census 2024 usual work location, valid comuna, work codes 2/3"
                                .to_owned(),
                        ),
                    ]),
                    coverage,
                    observed_fields: vec![
                        "baseline commune population, households and dwellings",
                        "baseline work OD counts",
                    ],
                    modeled_fields: vec![
                        "all scenario counts",
                        "workers = included outbound commuters",
                        "jobs = included inbound commuters, not all jobs",
                    ],
                    assumptions: vec![
                        "The baseline reconstructs input aggregates. It is not independent predictive validation.",
                        "Work OD includes work-location codes 2/3 only. Work-at-home, foreign, unknown and no-fixed destinations are excluded.",
                        "Only RM-to-RM pairs enter the model. Source CSV rows to destinations outside RM remain unchanged but are omitted from model flows; metadata.coverage reports them.",
                        "Workers and jobs count included internal commuters only, not all employed residents. External commuting is not modeled.",
                        "All housing changes jointly scale dwelling stocks and relative residential weights from the fixed baseline; they are not hard occupancy caps.",
                        "Population, households and outbound workers are separately renormalized to preserve their area totals.",
                        "All jobs changes jointly scale destination odds within each origin. Housing changes that origin's worker total; jobs redistribute it. Reduced job attraction does not destroy workers or commute totals.",
                        "Unobserved OD pairs stay zero. There is no new route, price, vacancy, travel-time or migration-rate model.",
                        "Scenario multipliers must be finite and greater than zero. Counts are fractional expected values, not individual agents.",
                    ],
                },
                zones,
                flows,
            },
            pairs,
            site: site::SiteModel::new(observations).unwrap(),
            site_destinations,
            site_modes,
            site_mode_total,
            station_model,
            station_observed,
            station_observation_metadata,
            event_model: event::EventModel::new(directory),
            latest: std::sync::Mutex::new((0, serde_json::Value::Null)),
            scenario_events: tokio::sync::watch::channel(serde_json::Value::Null).0,
        }
    }

    // Preserve the existing single-zone client shape while using the shared calculation.
    fn simulate_request(&self, request: ScenarioRequest) -> Result<serde_json::Value, String> {
        let (changes, legacy) = match request {
            ScenarioRequest::Shared { changes } => (changes, None),
            ScenarioRequest::Legacy(change) => (vec![change.clone()], Some(change)),
        };
        let result = city::simulate(&self.baseline, &self.pairs, changes)?;
        if let Some(change) = legacy {
            let d = result
                .zone_deltas
                .iter()
                .find(|d| d.id == change.zone_id)
                .unwrap();
            let summary = Summary {
                zone_id: change.zone_id,
                jobs_multiplier: change.jobs_multiplier,
                housing_multiplier: change.housing_multiplier,
                total_population: result.summary.total_population,
                total_households: result.summary.total_households,
                total_workers: result.summary.total_workers,
                selected_zone_population_delta: d.population,
                selected_zone_households_delta: d.households,
                selected_zone_dwellings_delta: d.dwellings,
                selected_zone_workers_delta: d.workers,
                selected_zone_jobs_delta: d.jobs,
            };
            Ok(serde_json::to_value(Simulation {
                world: result.scenario,
                summary,
                elapsed_ms: result.elapsed_ms,
            })
            .unwrap())
        } else {
            let mut latest = self.latest.lock().unwrap();
            let mut result = serde_json::to_value(result).unwrap();
            result["revision"] = serde_json::json!(latest.0 + 1);
            *latest = (latest.0 + 1, result.clone());
            self.scenario_events.send_replace(result.clone());
            Ok(result)
        }
    }
}

async fn scenario_events(
    State(core): State<Arc<Core>>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let mut receiver = core.scenario_events.subscribe();
    let initial = receiver.borrow_and_update().clone();
    let stream = futures_util::stream::unfold(
        (receiver, Some(initial)),
        |(mut receiver, initial)| async move {
            let result = match initial.filter(|result| !result.is_null()) {
                Some(result) => result,
                None => {
                    receiver.changed().await.ok()?;
                    receiver.borrow_and_update().clone()
                }
            };
            let event = Event::default()
                .event("scenario")
                .id(result["revision"].to_string())
                .json_data(result)
                .unwrap();
            Some((Ok(event), (receiver, None)))
        },
    );
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn event_scenario(
    State(core): State<Arc<Core>>,
    Json(request): Json<event::EventRequest>,
) -> Result<Json<event::EventResponse>, (StatusCode, Json<serde_json::Value>)> {
    core.event_model
        .scenario(request)
        .map(Json)
        .map_err(|error| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": error })),
            )
        })
}

async fn station_scenario(
    State(core): State<Arc<Core>>,
    Json(request): Json<stations::MutRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let prediction = core.station_model.mut_scenario(request).map_err(|error| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": error })),
        )
    })?;
    let mut response = serde_json::to_value(prediction).unwrap();
    for station in response["stations"].as_array_mut().unwrap() {
        let observed = &core.station_observed[station["id"].as_str().unwrap()];
        let (mut observed_daily, mut base_daily, mut scenario_daily, mut delta_daily, mut bins) =
            (0.0, 0.0, 0.0, 0.0, 0);
        for (t, observation) in observed.iter().enumerate() {
            if let Some(value) = observation {
                let base = station["predicted_base"][t].as_f64().unwrap();
                let scenario = station["predicted_scenario"][t].as_f64().unwrap();
                observed_daily += value;
                base_daily += base;
                scenario_daily += scenario;
                delta_daily += scenario - base;
                bins += 1;
            }
        }
        station["observed"] = serde_json::json!(observed);
        station["observed_daily"] = serde_json::json!(observed_daily);
        station["predicted_base_daily"] = serde_json::json!(base_daily);
        station["predicted_scenario_daily"] = serde_json::json!(scenario_daily);
        station["delta_daily"] = serde_json::json!(delta_daily);
        station["observed_bin_count"] = serde_json::json!(bins);
        station["daily_sum_scope"] = serde_json::json!("observed_bins_only");
    }
    response["observed_source"] = core.station_observation_metadata.clone();
    Ok(Json(response))
}

async fn site_scenario(
    State(core): State<Arc<Core>>,
    Json(request): Json<site::SiteRequest>,
) -> Result<Json<SiteHttpResponse>, (StatusCode, Json<serde_json::Value>)> {
    core.site
        .simulate(request, &core.site_destinations)
        .map(|site| {
            let commuters = site.scenario_residents
                * site.request.parameters.known_commuters_per_resident.value;
            let resident_commuters_by_mode = core.site_modes.iter().map(|row| {
                let share = row.commuters / core.site_mode_total;
                ResidentMode {
                    mode_code: row.mode,
                    mode_name: match row.mode {
                        1 => "Auto particular",
                        2 => "Transporte público (bus, micro, metro, tren, taxi, colectivo)",
                        3 => "Caminando",
                        4 => "Bicicleta (incluye scooter)",
                        5 => "Motocicleta",
                        6 => "Caballo, lancha o bote",
                        7 => "Otro",
                        -99 => "No respuesta",
                        _ => unreachable!("Unknown Census mode code"),
                    },
                    source_commuters: row.commuters,
                    share,
                    resident_commuters: commuters * share,
                    daily_home_departures: site.scenario_home_departures * share,
                }
            }).collect();
            Json(SiteHttpResponse {
                site,
                resident_commuters_by_mode,
                resident_commuters_by_mode_source: ModeSource {
                    source_kind: "observed_aggregate",
                    source: "INE Census 2024 p45_medio_transporte; data/mvp/mode_share.csv",
                    comuna_cut: "13114",
                    source_total_commuters: core.site_mode_total,
                    source_universe: "Las Condes usual commuters, all dwelling kinds, all known national work destinations; work-location codes 2/3; no-response mode retained",
                    modeled_universe: "Scenario resident commuters under the supplied RM-internal known-commuter ratio; daily departures include supplied attendance",
                    output_kind: "transfer_assumption",
                    assumption: "Transfer all-dwelling, national-destination mode shares to site residents with RM-internal commuter assumptions. Not observed new-apartment modes. Code 2 combines public transport and cannot identify a Metro share.",
                },
            })
        })
        .map_err(|error| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": error })),
            )
        })
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let directory = args.next().unwrap_or_else(|| "data/mvp".to_owned());
    let port = args.next().unwrap_or_else(|| "8787".to_owned());
    let core = Arc::new(Core::load(Path::new(&directory)));
    let app = Router::new()
        .route(
            "/api/world",
            get(|State(core): State<Arc<Core>>| async move { Json(core.baseline.clone()) }),
        )
        .route(
            "/api/simulate",
            post(
                |State(core): State<Arc<Core>>, Json(scenario): Json<ScenarioRequest>| async move {
                    core.simulate_request(scenario).map(Json).map_err(|error| {
                        (
                            StatusCode::UNPROCESSABLE_ENTITY,
                            Json(serde_json::json!({"error": error})),
                        )
                    })
                },
            ),
        )
        .route(
            "/api/scenario-latest",
            get(|State(core): State<Arc<Core>>| async move {
                let result = core.latest.lock().unwrap().1.clone();
                ([("cache-control", "no-store")], Json(result))
            }),
        )
        .route("/api/site-scenario", post(site_scenario))
        .route("/api/station-scenario", post(station_scenario))
        .route("/api/event-scenario", post(event_scenario))
        .route("/api/scenario-events", get(scenario_events))
        .layer(CorsLayer::permissive())
        .with_state(core);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    eprintln!(
        "Scenario core listening on http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}
