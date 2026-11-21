//! station_access <data dir> <all-open.json> <changed.json> [parent ID to close ...]
#[path = "../stations.rs"]
mod stations;
use stations::access::StationAccess;
use std::collections::BTreeMap;
use std::path::Path;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let data = Path::new(&args[1]);
    let started = std::time::Instant::now();
    let model = StationAccess::load(data);
    let load_ms = started.elapsed().as_millis();
    let (homes, jobs) = model.baseline_points(data);
    let mut active = model.all_parent_ids();
    for (i, output) in args[2..4].iter().enumerate() {
        if i == 1 {
            for id in &args[4..] { active.remove(id); }
        }
        let t = std::time::Instant::now();
        let predictions: BTreeMap<_, _> = model.project_points(&homes, &jobs, &active)
            .into_iter().map(|(id, _, bins)| (id, bins)).collect();
        let project_ms = t.elapsed().as_millis();
        serde_json::to_writer(std::fs::File::create(output).unwrap(), &predictions).unwrap();
        println!("active_parents {} project_ms {project_ms}", active.len());
    }
    println!("walk_scale {} bus_scale {} bus_mix {} training_cells {} load_ms {load_ms}",
        model.walk_scale(), model.bus_scale, model.bus_mix, model.training_cells);
}
