//! Read a scenario result from stdin. Arguments: data directory, station parent ID, half-hour index (0–47).
use santiago_scenario_core::station_capacity::StationCapacity;

use serde_json::{Value, json};
use std::path::Path;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let capacity = StationCapacity::load(Path::new(&args[1]));
    let id = &args[2];
    let bin: usize = args[3].parse().unwrap();
    let scenario: Value = serde_json::from_reader(std::io::stdin().lock()).unwrap();
    let station = scenario["stations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == id.as_str())
        .unwrap();
    let open = scenario["applied"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|change| {
            change["kind"] == "station" && change["understood"]["station"] == station["name"]
        })
        .is_none_or(|change| change["understood"]["state"] == "open");
    let base = capacity.half_hour(
        id,
        bin,
        station["base_half_hour"][bin].as_f64().unwrap(),
        true,
    );
    let changed = capacity.half_hour(
        id,
        bin,
        station["scenario_half_hour"][bin].as_f64().unwrap(),
        open,
    );
    println!(
        "{}",
        json!({ "station_id": id, "half_hour": bin, "base": base, "scenario": changed })
    );
}
