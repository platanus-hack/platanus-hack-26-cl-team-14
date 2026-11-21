//! Manual run: cargo run --release --manifest-path engine/Cargo.toml --bin station_flows -- data/mvp <flows.csv> <out.json> ['<BlockStock json>']
//! flows.csv has the columns of work_od.csv (origin_cut, destination_cut, workers). Output: {station name: [48]}.
#[path = "../stations.rs"]
mod stations;
use std::collections::BTreeMap;

#[derive(serde::Deserialize)]
struct Od { origin_cut: String, destination_cut: String, workers: f64 }

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let model = stations::StationModel::new(std::path::Path::new(&a[1]));
    let flows: Vec<(String, String, f64)> = csv::Reader::from_path(&a[2]).unwrap().deserialize::<Od>()
        .map(|r| { let r = r.unwrap(); (r.origin_cut, r.destination_cut, r.workers) }).collect();
    let out: BTreeMap<String, Vec<f64>> = model.project(&flows, &a.get(4).map_or_else(stations::BlockStock::default, |j| serde_json::from_str(j).unwrap())).into_iter().map(|(_, name, v)| (name, v)).collect();
    serde_json::to_writer(std::fs::File::create(&a[3]).unwrap(), &out).unwrap();
}
