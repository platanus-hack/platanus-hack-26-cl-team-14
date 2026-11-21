//! Hypothetical additional residents from MINVU permits and Census 2024 housing.
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Deserialize)]
struct Permit {
    cut: String,
    year: u16,
    dwellings_authorized: u64,
    minvu_status: String,
}

#[derive(Deserialize)]
struct Housing {
    cut: String,
    occupied_share: f64,
    persons_per_occupied_dwelling: f64,
}

pub fn expand(
    dir: &Path,
    years: &str,
    built_share: f64,
) -> Result<(BTreeMap<String, u64>, Value), String> {
    let year_error = "Use one permit year or an inclusive range within 2023-2025.";
    let year = |s: &str| {
        s.parse::<u16>()
            .ok()
            .filter(|n| s.len() == 4 && (2023..=2025).contains(n))
            .ok_or(year_error)
    };
    let (first, last) = years.split_once('-').unwrap_or((years, years));
    let (first, last) = (year(first)?, year(last)?);
    if first > last {
        return Err(year_error.into());
    }
    if !(0.0..=1.0).contains(&built_share) {
        return Err(
            "The completed share additional to the Census baseline must be between 0 and 1.".into(),
        );
    }

    let read_error = |e: csv::Error| e.to_string();
    let mut dwellings = BTreeMap::<String, u64>::new();
    let mut preliminary_years = BTreeSet::new();
    let mut permits =
        csv::Reader::from_path(dir.join("building_permits.csv")).map_err(read_error)?;
    for row in permits.deserialize::<Permit>() {
        let row = row.map_err(read_error)?;
        if (first..=last).contains(&row.year) {
            *dwellings.entry(row.cut).or_default() += row.dwellings_authorized;
            if row.minvu_status == "preliminary" {
                preliminary_years.insert(row.year);
            }
        }
    }

    let authorized = dwellings.values().sum::<u64>();
    let mut residents = BTreeMap::new();
    let mut housing = csv::Reader::from_path(dir.join("site_inputs.csv")).map_err(read_error)?;
    for row in housing.deserialize::<Housing>() {
        let row = row.map_err(read_error)?;
        if let Some(count) = dwellings.remove(&row.cut).filter(|&n| n > 0) {
            let per_dwelling = row.occupied_share * row.persons_per_occupied_dwelling;
            residents.insert(
                row.cut,
                (count as f64 * built_share * per_dwelling).round() as u64,
            );
        }
    }
    if let Some((cut, _)) = dwellings.iter().find(|(_, count)| **count > 0) {
        return Err(format!(
            "No Census housing factors for permit comuna {cut}."
        ));
    }

    let note = json!({
        "years": years, "built_share": built_share, "dwellings_authorized": authorized,
        "residents_added": residents.values().sum::<u64>(), "comunas": residents.len(),
        "preliminary_years": preliminary_years,
        "assumptions": [
            "MINVU counts authorized dwellings. The user assumes the share completed and additional to the Census 2024 baseline.",
            "Each comuna keeps its Census 2024 occupied share and persons per occupied dwelling. The model rounds hypothetical additional residents once per comuna.",
            "Permits identify a comuna. The scenario distributes additional residents like its current residents."
        ]
    });
    Ok((residents, note))
}
