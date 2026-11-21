//! Scenario capacities. Source density assumptions are in context/research/land-use-blocks.md.
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize)]
struct Permit {
    cut: String,
    residential_m2_authorized: f64,
    dwellings_authorized: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct Housing {
    cut: String,
    occupied_share: f64,
    persons_per_occupied_dwelling: f64,
}

#[derive(Debug, Clone)]
pub(super) struct Occupancy {
    housing: HashMap<String, (Housing, f64)>,
}

impl Occupancy {
    pub(super) fn load(dir: &Path) -> Self {
        let mut totals = HashMap::<String, (f64, u64)>::new();
        for row in csv::Reader::from_path(dir.join("building_permits.csv"))
            .unwrap()
            .deserialize::<Permit>()
        {
            let permit = row.unwrap();
            let total = totals.entry(permit.cut).or_default();
            total.0 += permit.residential_m2_authorized;
            total.1 += permit.dwellings_authorized;
        }
        let mut housing = HashMap::new();
        for row in csv::Reader::from_path(dir.join("site_inputs.csv"))
            .unwrap()
            .deserialize::<Housing>()
        {
            let census = row.unwrap();
            if let Some(&(area, count)) = totals.get(&census.cut) {
                if area > 0.0 && count > 0 {
                    housing.insert(census.cut.clone(), (census, area / count as f64));
                }
            }
        }
        Self { housing }
    }

    /// Returns modeled jobs, residents, and the assumptions used. Unspecified use has no inferred capacity.
    pub(super) fn estimate(
        &self,
        cut: &str,
        properties: &Value,
        area_m2: f64,
    ) -> Result<(Option<f64>, Option<f64>, Vec<String>), String> {
        let number = |key: &str| -> Result<Option<f64>, String> {
            properties
                .get(key)
                .map(|v| {
                    v.as_f64()
                        .filter(|n| n.is_finite() && *n >= 0.0)
                        .ok_or_else(|| format!("{key} must be a finite, nonnegative number."))
                })
                .transpose()
        };
        let mut jobs = number("jobs")?;
        let mut residents = number("residents")?;
        let height = number("height_m")?.unwrap_or(3.5);
        if height <= 0.0 || !area_m2.is_finite() || area_m2 <= 0.0 {
            return Err("The building needs positive height and footprint area.".into());
        }
        let use_name = match properties.get("use") {
            Some(v) => v.as_str().ok_or("Building use must be a string.")?,
            None => "unknown",
        };
        let floor_area = area_m2 * (height / 3.5).max(1.0);
        let mut notes = Vec::new();
        if jobs.is_some() {
            notes.push("The scenario supplies the building job count.".into());
        }
        if residents.is_some() {
            notes.push("The scenario supplies the building resident count.".into());
        }
        let estimate_from_size = match use_name {
            "office" | "retail" | "industrial" => {
                let infer_jobs = jobs.is_none();
                if infer_jobs {
                    let gross_m2 = match use_name {
                        "office" => 1.175 * 12.0,
                        "retail" => 1.175 * 19.0,
                        _ => 36.0,
                    };
                    jobs = Some((floor_area / gross_m2).round());
                    notes.push(format!("Assumes {gross_m2:.3} gross m² per modeled {use_name} job, using UK full-time-equivalent employment densities as a proxy for Santiago."));
                }
                if residents.is_none() {
                    residents = Some(0.0);
                    notes.push(
                        "The model assigns zero residents to exclusively nonresidential use."
                            .into(),
                    );
                }
                infer_jobs
            }
            "residential" => {
                let infer_residents = residents.is_none();
                if infer_residents {
                    let (census, m2_per_dwelling) = self.housing.get(cut).ok_or_else(|| {
                        format!("No housing density for CUT {cut}. Supply residents.")
                    })?;
                    residents = Some(
                        (floor_area / m2_per_dwelling
                            * census.occupied_share
                            * census.persons_per_occupied_dwelling)
                            .round(),
                    );
                    notes.push(format!("CUT {cut}: {m2_per_dwelling:.2} authorized residential m² per dwelling from total MINVU permits for 2023–2025 (2025 preliminary). Census 2024 occupied share {:.4} and {:.4} persons per occupied dwelling. The estimate assumes these comuna averages apply to this building.", census.occupied_share, census.persons_per_occupied_dwelling));
                }
                if jobs.is_none() {
                    jobs = Some(0.0);
                    notes
                        .push("The model assigns zero jobs to exclusively residential use.".into());
                }
                infer_residents
            }
            "unknown" => {
                if jobs.is_none() || residents.is_none() {
                    notes.push(
                        "Building use is unknown. Unspecified capacity has no estimate.".into(),
                    );
                }
                false
            }
            _ => return Err(format!("Unsupported building use: {use_name}.")),
        };
        if estimate_from_size {
            notes.push("Gross floor area equals footprint area times height / 3.5 m, with at least one floor equivalent. The estimate assumes the same footprint on each floor and rounds capacity to the nearest person.".into());
            if properties.get("height_m").is_none() {
                notes.push("Missing height assumes one floor of 3.5 m.".into());
            }
        }
        Ok((jobs, residents, notes))
    }
}
