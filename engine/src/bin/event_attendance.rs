//! event_attendance <data dir> <request.json> [closed GTFS parent IDs...]
#[path = "../attendance.rs"]
mod attendance;
#[path = "../population.rs"]
mod population;
#[path = "../stations.rs"]
mod stations;
use std::path::Path;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let data = Path::new(&args[1]);
    let request = serde_json::from_reader(std::fs::File::open(&args[2]).unwrap()).unwrap();
    let pop = population::Population::build(data);
    let access = stations::access::StationAccess::load(data);
    let mut active = access.all_parent_ids();
    for id in &args[3..] { active.remove(id); }
    let result = attendance::project(&pop, &access, &active, request);
    serde_json::to_writer(std::io::stdout().lock(), &result).unwrap();
}
