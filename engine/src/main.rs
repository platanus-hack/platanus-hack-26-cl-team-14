use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Arc, time::Instant};
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

#[derive(Clone, Serialize)]
struct Zone {
    id: String,
    name: String,
    longitude: f64,
    latitude: f64,
    population: f64,
    households: f64,
    dwellings: f64,
    workers: f64,
    jobs: f64,
}

#[derive(Clone, Serialize)]
struct Flow {
    origin: String,
    destination: String,
    workers: f64,
}

#[derive(Clone, Serialize)]
struct Metadata {
    model: &'static str,
    state: &'static str,
    data_vintages: HashMap<&'static str, String>,
    observed_fields: Vec<&'static str>,
    modeled_fields: Vec<&'static str>,
    assumptions: Vec<&'static str>,
}

#[derive(Clone, Serialize)]
struct World {
    metadata: Metadata,
    zones: Vec<Zone>,
    flows: Vec<Flow>,
}

struct Core {
    baseline: World,
    pairs: Vec<(usize, usize)>,
}

#[derive(Deserialize)]
struct Scenario {
    zone_id: String,
    jobs_multiplier: f64,
    housing_multiplier: f64,
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
        for row in od.deserialize::<WorkOd>() {
            let f = row.unwrap();
            let origin = ids[&f.origin_cut];
            let destination = ids[&f.destination_cut];
            zones[origin].workers += f.workers;
            zones[destination].jobs += f.workers;
            pairs.push((origin, destination));
            flows.push(Flow {
                origin: f.origin_cut,
                destination: f.destination_cut,
                workers: f.workers,
            });
        }
        Self {
            baseline: World {
                metadata: Metadata {
                    model: "empirical-od-relative-capacity-v1",
                    state: "observed_baseline",
                    data_vintages: HashMap::from([
                        (
                            "communes",
                            "Unconfirmed: awaiting extract provenance".to_owned(),
                        ),
                        (
                            "work_od",
                            "Unconfirmed: awaiting extract provenance".to_owned(),
                        ),
                    ]),
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
                        "Only included origins and destinations define the modeled area. There is no external inflow or outflow.",
                        "Housing scales selected-zone dwelling stock and its relative residential weight. It is not a hard occupancy cap.",
                        "Population, households and outbound workers are separately renormalized to preserve their area totals.",
                        "Jobs scale the selected destination's observed odds within each origin. Each origin's scenario worker total is preserved.",
                        "Unobserved OD pairs stay zero. There is no new route, price, vacancy, travel-time or migration-rate model.",
                        "Scenario multipliers must be finite and greater than zero. Counts are fractional expected values, not individual agents.",
                    ],
                },
                zones,
                flows,
            },
            pairs,
        }
    }

    fn simulate(&self, scenario: Scenario) -> Simulation {
        let start = Instant::now();
        let mut world = self.baseline.clone();
        let selected = world
            .zones
            .iter()
            .position(|z| z.id == scenario.zone_id)
            .unwrap();
        let before = &self.baseline.zones[selected];
        let h = scenario.housing_multiplier;
        let j = scenario.jobs_multiplier;
        let population: f64 = world.zones.iter().map(|z| z.population).sum();
        let households: f64 = world.zones.iter().map(|z| z.households).sum();
        let workers: f64 = world.zones.iter().map(|z| z.workers).sum();
        if h != 1.0 || j != 1.0 {
            world.metadata.state = "modeled_scenario";
            let population_scale = population / (population + before.population * (h - 1.0));
            let households_scale = households / (households + before.households * (h - 1.0));
            let workers_scale = workers / (workers + before.workers * (h - 1.0));
            for (i, zone) in world.zones.iter_mut().enumerate() {
                let weight = if i == selected { h } else { 1.0 };
                zone.population *= weight * population_scale;
                zone.households *= weight * households_scale;
                zone.workers *= weight * workers_scale;
                zone.jobs = 0.0;
            }
            world.zones[selected].dwellings *= h;

            // The empirical row sum is the normalizer; only the selected destination changes its odds.
            let mut row_weights: Vec<f64> = self.baseline.zones.iter().map(|z| z.workers).collect();
            for (flow, &(origin, destination)) in self.baseline.flows.iter().zip(&self.pairs) {
                if destination == selected {
                    row_weights[origin] += flow.workers * (j - 1.0);
                }
            }
            for (flow, &(origin, destination)) in world.flows.iter_mut().zip(&self.pairs) {
                let weight = if destination == selected { j } else { 1.0 };
                flow.workers *= weight * world.zones[origin].workers / row_weights[origin];
                world.zones[destination].jobs += flow.workers;
            }
        }
        let after = &world.zones[selected];
        let summary = Summary {
            zone_id: scenario.zone_id,
            jobs_multiplier: j,
            housing_multiplier: h,
            total_population: world.zones.iter().map(|z| z.population).sum(),
            total_households: world.zones.iter().map(|z| z.households).sum(),
            total_workers: world.flows.iter().map(|f| f.workers).sum(),
            selected_zone_population_delta: after.population - before.population,
            selected_zone_households_delta: after.households - before.households,
            selected_zone_dwellings_delta: after.dwellings - before.dwellings,
            selected_zone_workers_delta: after.workers - before.workers,
            selected_zone_jobs_delta: after.jobs - before.jobs,
        };
        Simulation {
            world,
            summary,
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }
}

#[tokio::main]
async fn main() {
    let directory = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "data/mvp".to_owned());
    let core = Arc::new(Core::load(Path::new(&directory)));
    let app = Router::new()
        .route(
            "/api/world",
            get(|State(core): State<Arc<Core>>| async move { Json(core.baseline.clone()) }),
        )
        .route(
            "/api/simulate",
            post(
                |State(core): State<Arc<Core>>, Json(scenario): Json<Scenario>| async move {
                    Json(core.simulate(scenario))
                },
            ),
        )
        .layer(CorsLayer::permissive())
        .with_state(core);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787")
        .await
        .unwrap();
    eprintln!("Scenario core listening on http://127.0.0.1:8787");
    axum::serve(listener, app).await.unwrap();
}
