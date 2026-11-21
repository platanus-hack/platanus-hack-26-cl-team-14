//! Calculate weekday station entries from shopping and health tours. Write only station aggregates.
//! Usage: shopping_health_stations DATA_MVP APRIL_ARRIVALS_CSV BASELINE_JSON PREDICTIONS_JSON
#[path = "../population.rs"]
mod population;
#[path = "../shopping_health_stations.rs"]
mod shopping_health_stations;
#[path = "../stations.rs"]
mod stations;

use population::{Population, shopping_health::ShoppingHealthDay};
use shopping_health_stations::{Day, StationFit, StationLegs};
use stations::{Station, access::walk_links};
use std::{fs::File, path::PathBuf};

fn main() {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let data = &args[0];
    let stations: Vec<Station> =
        serde_json::from_reader(File::open(data.join("metro_training.json")).unwrap()).unwrap();
    let baseline: Day = serde_json::from_reader(File::open(&args[2]).unwrap()).unwrap();
    let blocks = Population::homes(data);
    let day = ShoppingHealthDay::build(data, &blocks);
    let homes: Vec<_> = blocks
        .iter()
        .map(|p| walk_links(&stations, p.longitude, p.latitude))
        .collect();
    let destinations: Vec<_> = day
        .destinations
        .iter()
        .map(|p| walk_links(&stations, p[0], p[1]))
        .collect();
    let mut legs = StationLegs::new(stations.len());
    day.visit_tours(|home, destination, time| {
        if time.out_mode == 4 {
            legs.leg(
                &homes[home],
                &destinations[destination],
                time.out_start_bin,
                time.out_end_bin,
                true,
            );
        }
        if time.return_mode == 4 {
            legs.leg(
                &destinations[destination],
                &homes[home],
                time.return_start_bin,
                time.return_end_bin,
                false,
            );
        }
    });
    let share = csv::Reader::from_path(data.join("eod_tours/metro_coverage.csv"))
        .unwrap()
        .records()
        .map(Result::unwrap)
        .find(|r| &r[0] == "OTROS")
        .unwrap()[3]
        .parse()
        .unwrap();
    let fit = StationFit::fit(&stations, &legs, &baseline, share, &args[1]);
    serde_json::to_writer(
        File::create(&args[3]).unwrap(),
        &fit.apply(&stations, &legs, &baseline),
    )
    .unwrap();
    println!(
        "{}",
        serde_json::json!({"population":day.coverage,"stations":fit})
    );
}
