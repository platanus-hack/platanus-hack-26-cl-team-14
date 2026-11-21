//! Local Tobalaba accounting; observed baseline is never scaled as a whole.
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Clone, Deserialize, Serialize)]
pub struct Parameter {
    pub value: f64,
    pub source_kind: String,
    pub source: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Parameters {
    pub occupied_share: Parameter,
    pub persons_per_occupied_dwelling: Parameter,
    pub known_commuters_per_resident: Parameter,
    pub attendance_home: Parameter,
    pub attendance_work: Parameter,
    pub filled_job_share: Parameter,
    pub metro_entry_share_home: Parameter,
    pub metro_entry_share_work: Parameter,
    pub access_share_home: Parameter,
    pub access_share_work: Parameter,
    pub maximum_access_distance_m: Parameter,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SiteRequest {
    pub station_id: String,
    pub comuna_cut: String,
    pub longitude: f64,
    pub latitude: f64,
    pub baseline_jobs: Parameter,
    pub baseline_dwellings: Parameter,
    pub jobs_removed: Parameter,
    pub dwellings_added: Parameter,
    pub parameters: Parameters,
    pub time_profile: String,
    pub population_policy: String,
}

#[derive(Deserialize)]
pub struct ObservationData {
    pub station: Station,
    pub observations: Vec<Observation>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Deserialize)]
pub struct Observation {
    pub day_type: String,
    pub half_hour: String,
    pub boardings: f64,
}

pub struct SiteModel {
    station: Station,
    observed: Vec<Option<f64>>,
    home_profile: Vec<f64>,
    work_profile: Vec<f64>,
}

#[derive(Serialize)]
pub struct SiteFlow {
    pub destination_cut: String,
    pub baseline_commuters: f64,
    pub scenario_commuters: f64,
    pub delta_commuters: f64,
}

#[derive(Serialize)]
pub struct SiteResponse {
    pub station: Station,
    pub observed: Vec<Option<f64>>,
    pub base: Vec<Option<f64>>,
    pub scenario: Vec<Option<f64>>,
    pub delta: Vec<Option<f64>>,
    pub site_base: Vec<Option<f64>>,
    pub site_scenario: Vec<Option<f64>>,
    pub home_delta: Vec<Option<f64>>,
    pub work_delta: Vec<Option<f64>>,
    pub baseline_residents: f64,
    pub scenario_residents: f64,
    pub baseline_home_departures: f64,
    pub scenario_home_departures: f64,
    pub flows: Vec<SiteFlow>,
    pub entrance_id: &'static str,
    pub entrance_distance_m: f64,
    pub entrance_longitude: f64,
    pub entrance_latitude: f64,
    pub entrance_source: &'static str,
    pub model_boundary: &'static str,
    pub global_conservation_claim: bool,
    pub validation: &'static str,
    pub scenario_day: &'static str,
    pub source_period: &'static str,
    pub timezone: &'static str,
    pub assumptions: Vec<&'static str>,
    pub request: SiteRequest,
    pub elapsed_ms: f64,
}

fn parameter(name: &str, p: &Parameter, fraction: bool) -> Result<(), String> {
    if !p.value.is_finite() || p.value < 0.0 || (fraction && p.value > 1.0) {
        return Err(format!(
            "{name} must be finite and {}",
            if fraction {
                "between 0 and 1"
            } else {
                "nonnegative"
            }
        ));
    }
    if !["observed_aggregate", "historical_proxy", "user_assumption"]
        .contains(&p.source_kind.as_str())
        || p.source.trim().is_empty()
    {
        return Err(format!(
            "{name} needs source_kind (observed_aggregate, historical_proxy, user_assumption) and source"
        ));
    }
    Ok(())
}

impl SiteModel {
    pub fn new(data: ObservationData) -> Result<Self, String> {
        if data.station.id != "TB-TOB" {
            return Err("Only the reviewed TB-TOB station is supported".into());
        }
        let mut observed = vec![None; 48];
        for row in data
            .observations
            .into_iter()
            .filter(|r| r.day_type == "LABORAL")
        {
            let (h, m) = row
                .half_hour
                .split_once(':')
                .ok_or("Invalid observation time")?;
            let h: usize = h.parse().map_err(|_| "Invalid observation hour")?;
            let m: usize = m.parse().map_err(|_| "Invalid observation minute")?;
            if h > 23 || ![0, 30].contains(&m) || !row.boardings.is_finite() || row.boardings < 0.0
            {
                return Err("Invalid weekday observation".into());
            }
            let slot = &mut observed[h * 2 + m / 30];
            if slot.is_some() {
                return Err("Duplicate weekday observation".into());
            }
            *slot = Some(row.boardings);
        }
        let profile = |start: usize, end: usize| -> Result<Vec<f64>, String> {
            let total: f64 = observed[start..end].iter().flatten().sum();
            if total <= 0.0 {
                return Err("Time window has no positive observations".into());
            }
            Ok((0..48)
                .map(|i| {
                    if (start..end).contains(&i) {
                        observed[i].unwrap_or(0.0) / total
                    } else {
                        0.0
                    }
                })
                .collect())
        };
        let home_profile = profile(12, 20)?; // 06:00 inclusive, 10:00 exclusive.
        let work_profile = profile(32, 42)?; // 16:00 inclusive, 21:00 exclusive.
        Ok(Self {
            station: data.station,
            observed,
            home_profile,
            work_profile,
        })
    }

    pub fn simulate(
        &self,
        request: SiteRequest,
        destinations: &[(String, f64)],
    ) -> Result<SiteResponse, String> {
        let start = Instant::now();
        // This release supports the reviewed MUT point only, not arbitrary point-to-comuna assignment.
        if request.station_id != "TB-TOB"
            || request.comuna_cut != "13114"
            || (request.longitude - -70.6009).abs() > 0.000001
            || (request.latitude - -33.4180).abs() > 0.000001
            || !request.longitude.is_finite()
            || !request.latitude.is_finite()
        {
            return Err(
                "Supported site: MUT (-70.6009,-33.4180), Las Condes 13114, station TB-TOB".into(),
            );
        }
        if request.time_profile != "observed_am_06_10_pm_16_21"
            || request.population_policy != "local_increment_unbalanced"
        {
            return Err(
                "Required profile observed_am_06_10_pm_16_21 and policy local_increment_unbalanced"
                    .into(),
            );
        }
        for (name, p) in [
            ("baseline_jobs", &request.baseline_jobs),
            ("baseline_dwellings", &request.baseline_dwellings),
            ("jobs_removed", &request.jobs_removed),
            ("dwellings_added", &request.dwellings_added),
        ] {
            parameter(name, p, false)?;
        }
        if request.jobs_removed.value > request.baseline_jobs.value {
            return Err("jobs_removed exceeds baseline_jobs".into());
        }
        let p = &request.parameters;
        for (name, p) in [
            ("occupied_share", &p.occupied_share),
            (
                "known_commuters_per_resident",
                &p.known_commuters_per_resident,
            ),
            ("attendance_home", &p.attendance_home),
            ("attendance_work", &p.attendance_work),
            ("filled_job_share", &p.filled_job_share),
            ("metro_entry_share_home", &p.metro_entry_share_home),
            ("metro_entry_share_work", &p.metro_entry_share_work),
            ("access_share_home", &p.access_share_home),
            ("access_share_work", &p.access_share_work),
        ] {
            parameter(name, p, true)?;
        }
        parameter(
            "persons_per_occupied_dwelling",
            &p.persons_per_occupied_dwelling,
            false,
        )?;
        parameter(
            "maximum_access_distance_m",
            &p.maximum_access_distance_m,
            false,
        )?;
        // Exact entrance records from GTFS_20260321_v3, all parent_station=TB-TOB.
        let entrances = [
            ("TB-TOB:F", -70.60073_f64, -33.41769_f64),
            ("TB-TOB:E", -70.60187, -33.4171),
            ("TB-TOB:A", -70.602911, -33.418451),
            ("TB-TOB:D", -70.60279, -33.418868),
            ("TB-TOB:C", -70.601327, -33.418532),
            ("TB-TOB:B", -70.601944, -33.418078),
        ];
        let (entrance_id, entrance_longitude, entrance_latitude, distance) = entrances
            .iter()
            .map(|&(id, lon, lat)| {
                let a = ((lat - request.latitude).to_radians() / 2.0).sin().powi(2)
                    + request.latitude.to_radians().cos()
                        * lat.to_radians().cos()
                        * ((lon - request.longitude).to_radians() / 2.0).sin().powi(2);
                (id, lon, lat, 6371000.0 * 2.0 * a.sqrt().asin())
            })
            .min_by(|a, b| a.3.total_cmp(&b.3))
            .unwrap();
        if distance > p.maximum_access_distance_m.value {
            return Err(format!(
                "Nearest Tobalaba entrance is {distance:.1}m away, beyond maximum_access_distance_m"
            ));
        }
        let residents = |d: f64| d * p.occupied_share.value * p.persons_per_occupied_dwelling.value;
        let r0 = residents(request.baseline_dwellings.value);
        let r1 = residents(request.baseline_dwellings.value + request.dwellings_added.value);
        let depart = |r: f64| r * p.known_commuters_per_resident.value * p.attendance_home.value;
        let h0 = depart(r0);
        let h1 = depart(r1);
        let metro_home = p.metro_entry_share_home.value * p.access_share_home.value;
        let metro_work = p.filled_job_share.value
            * p.attendance_work.value
            * p.metro_entry_share_work.value
            * p.access_share_work.value;
        let w0 = request.baseline_jobs.value * metro_work;
        let w1 = (request.baseline_jobs.value - request.jobs_removed.value) * metro_work;
        if [r0, r1, h0, h1, w0, w1].iter().any(|v| !v.is_finite()) {
            return Err("Scenario arithmetic exceeds finite count range".into());
        }
        let mut scenario = vec![None; 48];
        let mut delta = vec![None; 48];
        let mut site_base = vec![None; 48];
        let mut site_scenario = vec![None; 48];
        let mut home_delta = vec![None; 48];
        let mut work_delta = vec![None; 48];
        for i in 0..48 {
            if let Some(b) = self.observed[i] {
                let c0 = h0 * metro_home * self.home_profile[i] + w0 * self.work_profile[i];
                let c1 = h1 * metro_home * self.home_profile[i] + w1 * self.work_profile[i];
                if c0 > b {
                    return Err(format!(
                        "Baseline site contribution {c0:.3} exceeds observed {b:.3} at bin {i}; revise explicit assumptions"
                    ));
                }
                let change = c1 - c0;
                scenario[i] = Some(b + change);
                delta[i] = Some(change);
                site_base[i] = Some(c0);
                site_scenario[i] = Some(c1);
                home_delta[i] = Some((h1 - h0) * metro_home * self.home_profile[i]);
                work_delta[i] = Some((w1 - w0) * self.work_profile[i]);
            }
        }
        if destinations.iter().any(|(_, w)| !w.is_finite() || *w < 0.0) {
            return Err("OD weights must be finite and nonnegative".into());
        }
        let total: f64 = destinations.iter().map(|(_, w)| w).sum();
        if !total.is_finite() || (h1 > 0.0 && total <= 0.0) {
            return Err("Positive home departures require actual Las Condes OD weights".into());
        }
        let flows = destinations
            .iter()
            .filter(|(_, w)| *w > 0.0)
            .map(|(cut, w)| SiteFlow {
                destination_cut: cut.clone(),
                baseline_commuters: h0 * w / total,
                scenario_commuters: h1 * w / total,
                delta_commuters: (h1 - h0) * w / total,
            })
            .collect();
        Ok(SiteResponse {
            station: self.station.clone(),
            observed: self.observed.clone(),
            base: self.observed.clone(),
            scenario,
            delta,
            site_base,
            site_scenario,
            home_delta,
            work_delta,
            baseline_residents: r0,
            scenario_residents: r1,
            baseline_home_departures: h0,
            scenario_home_departures: h1,
            flows,
            entrance_id,
            entrance_distance_m: distance,
            entrance_longitude,
            entrance_latitude,
            entrance_source: "GTFS_20260321_v3.zip stops.txt, location_type=2, parent_station=TB-TOB; SHA256 f10945e80beebc39ea2ac70de997d3d69714897b575de6fbf4e3ca643013b77b",
            model_boundary: "local_increment_unbalanced",
            global_conservation_claim: false,
            validation: "accounting_only",
            scenario_day: "LABORAL",
            source_period: "Average weekdays 2026-04-13 through 2026-04-17",
            timezone: "America/Santiago",
            assumptions: vec![
                "Required scalar parameters and their provenance are echoed; no hidden occupancy or Metro split.",
                "AM home departures 06:00–10:00 and PM job returns 16:00–21:00 use observed shape, not independent validation.",
                "Only the reviewed MUT point is supported; entrance distance is straight-line among Tobalaba entrances, not network routing.",
                "Comuna OD weights describe expected work destinations, not observed point OD or transit routes.",
                "Shopping, visitors, school, weekend and relocation effects are omitted; unrelated observed station activity stays fixed.",
                "Metro-entry shares must match validation coverage; combined public transport share is not Metro share.",
            ],
            request,
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }
}
