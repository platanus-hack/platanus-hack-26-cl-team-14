//! Calculate weekday station entries from the population study tours. Write only station aggregates.
//! Usage: study_stations DATA_MVP APRIL_ARRIVALS_CSV BASELINE_JSON PREDICTIONS_JSON
#[path = "../population.rs"]
mod population;
#[path = "../stations.rs"]
mod stations;
#[path = "../study_stations.rs"]
mod study_stations;

use population::{Population, study::StudyDay};
use stations::Station;
use std::{fs::File, path::PathBuf};
use study_stations::{Day, StudyFit, StudyStations, walk_links};

fn main() {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let data = &args[0];
    let stations: Vec<Station> =
        serde_json::from_reader(File::open(data.join("metro_training.json")).unwrap()).unwrap();
    let baseline: Day = serde_json::from_reader(File::open(&args[2]).unwrap()).unwrap();
    let pop = Population::build(data);
    let day = StudyDay::build(data, &pop);
    let homes: Vec<_> = pop
        .homes
        .iter()
        .map(|p| walk_links(&stations, p.longitude, p.latitude))
        .collect();
    let destinations: Vec<_> = day
        .destinations
        .iter()
        .map(|p| walk_links(&stations, p[0], p[1]))
        .collect();
    let mut study = StudyStations::new(stations.len());
    day.visit_tours(|home, destination, time| {
        if time.out_mode == 4 {
            study.leg(
                &homes[home],
                &destinations[destination],
                time.out_start_bin,
                time.out_end_bin,
                true,
            );
        }
        if time.return_mode == 4 {
            study.leg(
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
        .find(|r| &r[0] == "ESTUDIO")
        .unwrap()[3]
        .parse()
        .unwrap();
    let fit = StudyFit::fit(&stations, &study, &baseline, share, &args[1]);
    serde_json::to_writer(
        File::create(&args[3]).unwrap(),
        &fit.apply(&stations, &study, &baseline),
    )
    .unwrap();
    println!(
        "{}",
        serde_json::json!({"population":day.coverage,"stations":fit})
    );
}
