use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Instant};

#[derive(Clone, Serialize)]
pub(super) struct Zone {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) longitude: f64,
    pub(super) latitude: f64,
    pub(super) population: f64,
    pub(super) households: f64,
    pub(super) dwellings: f64,
    pub(super) workers: f64,
    pub(super) jobs: f64,
}

#[derive(Clone, Serialize)]
pub(super) struct Flow {
    pub(super) origin: String,
    pub(super) destination: String,
    pub(super) workers: f64,
}

#[derive(Clone, Serialize)]
pub(super) struct Coverage {
    pub(super) source_total_workers: f64,
    pub(super) included_workers: f64,
    pub(super) external_destination_workers: f64,
    pub(super) external_destination_pairs: usize,
}

#[derive(Clone, Serialize)]
pub(super) struct Metadata {
    pub(super) model: &'static str,
    pub(super) state: &'static str,
    pub(super) data_vintages: HashMap<&'static str, String>,
    pub(super) coverage: Coverage,
    pub(super) observed_fields: Vec<&'static str>,
    pub(super) modeled_fields: Vec<&'static str>,
    pub(super) assumptions: Vec<&'static str>,
}

#[derive(Clone, Serialize)]
pub(super) struct World {
    pub(super) metadata: Metadata,
    pub(super) zones: Vec<Zone>,
    pub(super) flows: Vec<Flow>,
}

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct ZoneChange {
    pub(super) zone_id: String,
    pub(super) jobs_multiplier: f64,
    pub(super) housing_multiplier: f64,
}

#[derive(Serialize)]
pub(super) struct ZoneDelta {
    pub(super) id: String,
    pub(super) population: f64,
    pub(super) households: f64,
    pub(super) dwellings: f64,
    pub(super) workers: f64,
    pub(super) jobs: f64,
}

#[derive(Serialize)]
pub(super) struct CitySummary {
    pub(super) changes: Vec<ZoneChange>,
    pub(super) total_population: f64,
    pub(super) total_households: f64,
    pub(super) total_workers: f64,
    pub(super) model_boundary: &'static str,
}

#[derive(Serialize)]
pub(super) struct CitySimulation {
    pub(super) baseline: World,
    pub(super) scenario: World,
    pub(super) zone_deltas: Vec<ZoneDelta>,
    pub(super) flow_deltas: Vec<Flow>,
    pub(super) summary: CitySummary,
    pub(super) elapsed_ms: f64,
}

pub(super) fn simulate(
    baseline: &World,
    pairs: &[(usize, usize)],
    changes: Vec<ZoneChange>,
) -> Result<CitySimulation, String> {
    let start = Instant::now();
    let mut world = baseline.clone();
    let n = world.zones.len();
    let (mut housing, mut jobs) = (vec![1.0; n], vec![1.0; n]);
    let mut changed = std::collections::HashSet::new();
    for change in &changes {
        let i = world
            .zones
            .iter()
            .position(|z| z.id == change.zone_id)
            .ok_or_else(|| format!("Unknown zone_id: {}", change.zone_id))?;
        if !changed.insert(i) {
            return Err("Each zone must appear only once in changes".into());
        }
        if ![change.housing_multiplier, change.jobs_multiplier]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0)
        {
            return Err("Housing and jobs multipliers must be finite and greater than zero".into());
        }
        housing[i] = change.housing_multiplier;
        jobs[i] = change.jobs_multiplier;
    }
    if housing.iter().chain(&jobs).any(|v| *v != 1.0) {
        world.metadata.state = "modeled_scenario";
        let scale = |field: fn(&Zone) -> f64| {
            let total: f64 = baseline.zones.iter().map(field).sum();
            total
                / baseline
                    .zones
                    .iter()
                    .enumerate()
                    .map(|(i, z)| field(z) * housing[i])
                    .sum::<f64>()
        };
        let (ps, hs, ws) = (
            scale(|z| z.population),
            scale(|z| z.households),
            scale(|z| z.workers),
        );
        for (i, z) in world.zones.iter_mut().enumerate() {
            z.population *= housing[i] * ps;
            z.households *= housing[i] * hs;
            z.workers *= housing[i] * ws;
            z.dwellings *= housing[i];
            z.jobs = 0.0;
        }
        // All destination weights compete in the same origin row. Housing changes its row mass.
        let mut row_weights = vec![0.0; n];
        for (flow, &(origin, destination)) in baseline.flows.iter().zip(pairs) {
            row_weights[origin] += flow.workers * jobs[destination];
        }
        for (flow, &(origin, destination)) in world.flows.iter_mut().zip(pairs) {
            flow.workers *= jobs[destination] * world.zones[origin].workers / row_weights[origin];
            world.zones[destination].jobs += flow.workers;
        }
    }
    let zone_deltas = world
        .zones
        .iter()
        .zip(&baseline.zones)
        .map(|(a, b)| ZoneDelta {
            id: a.id.clone(),
            population: a.population - b.population,
            households: a.households - b.households,
            dwellings: a.dwellings - b.dwellings,
            workers: a.workers - b.workers,
            jobs: a.jobs - b.jobs,
        })
        .collect();
    let flow_deltas = world
        .flows
        .iter()
        .zip(&baseline.flows)
        .map(|(a, b)| Flow {
            origin: a.origin.clone(),
            destination: a.destination.clone(),
            workers: a.workers - b.workers,
        })
        .collect();
    let summary = CitySummary {
        changes,
        total_population: world.zones.iter().map(|z| z.population).sum(),
        total_households: world.zones.iter().map(|z| z.households).sum(),
        total_workers: world.flows.iter().map(|f| f.workers).sum(),
        model_boundary: "closed_rm_redistribution",
    };
    Ok(CitySimulation {
        baseline: baseline.clone(),
        scenario: world,
        zone_deltas,
        flow_deltas,
        summary,
        elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}
