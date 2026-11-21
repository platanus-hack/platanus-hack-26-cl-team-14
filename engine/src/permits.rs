//! Building permits (MINVU, building_permits.csv) -> one `residents` change per comuna (cut 5, link 2).
//! New residents = dwellings authorized x built share x Census 2024 occupied share x persons per occupied dwelling (site_inputs.csv).
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Rows of a CSV as maps from header to value.
fn rows(path: &Path) -> Vec<HashMap<String, String>> {
    csv::Reader::from_path(path).unwrap().deserialize().map(|r| r.unwrap()).collect()
}

/// Dwellings authorized by comuna and year, and residents per dwelling by comuna.
pub struct Permits { dwellings: Vec<(String, u32, f64)>, per_dwelling: HashMap<String, f64> }

impl Permits {
pub fn load(dir: &Path) -> Permits {
    let per_dwelling = rows(&dir.join("site_inputs.csv")).into_iter()
        .map(|r| (r["cut"].clone(), r["occupied_share"].parse::<f64>().unwrap() * r["persons_per_occupied_dwelling"].parse::<f64>().unwrap())).collect();
    let dwellings = rows(&dir.join("building_permits.csv")).into_iter()
        .map(|r| (r["cut"].clone(), r["year"].parse().unwrap(), r["dwellings_authorized"].parse().unwrap())).collect();
    Permits { dwellings, per_dwelling }
}

/// `id`: "2023", "2024", "2025" or a range such as "2023-2025". `built_share`: share of those dwellings that is built
/// and not in the Census 2024 base, 0 to 1. Ok: the residents changes and a note for `applied`. Err: the reason for `not_modeled`.
pub fn expand(&self, id: &str, built_share: f64) -> Result<(Vec<Value>, Value), String> {
    let (a, b) = id.split_once('-').unwrap_or((id, id));
    let (from, to) = (a.parse::<u32>().unwrap_or(0), b.parse::<u32>().unwrap_or(0));
    if !(2023 <= from && from <= to && to <= 2025) { return Err("Permit years are 2023 to 2025: id \"2023\", \"2024\", \"2025\" or a range such as \"2023-2025\".".into()); }
    if !(0.0..=1.0).contains(&built_share) { return Err("The value of a permits change is the built share, 0 to 1.".into()); }
    let mut dwellings: BTreeMap<String, f64> = BTreeMap::new();
    for (cut, _, d) in self.dwellings.iter().filter(|r| (from..=to).contains(&r.1)) { *dwellings.entry(cut.clone()).or_default() += d; }
    let changes: Vec<Value> = dwellings.iter().filter(|(_, d)| **d > 0.0)
        .map(|(cut, d)| json!({ "kind": "residents", "target": { "type": "comuna", "id": cut }, "op": "add", "value": (d * built_share * self.per_dwelling[cut]).round() }))
        .collect();
    let mut assumptions = vec![
        "MINVU permits authorize dwellings; they give no completion date. The built share is an assumption of the question.",
        "Hypothetical scenario: the built dwellings add to the Census 2024 base. A dwelling authorized in 2023 may already be in the Census.",
        "Residents per dwelling = occupied share x persons per occupied dwelling of the comuna in the Census 2024.",
        "A permit gives a comuna, not a building: the new residents spread over the comuna like its current residents.",
    ];
    if to == 2025 { assumptions.push("2025 permits are preliminary MINVU figures."); }
    let note = json!({
        "years": id, "built_share": built_share, "comunas": changes.len(),
        "dwellings_authorized": dwellings.values().sum::<f64>(),
        "residents_added": changes.iter().map(|c| c["value"].as_f64().unwrap()).sum::<f64>(),
        "assumptions": assumptions,
    });
    Ok((changes, note))
}
}
