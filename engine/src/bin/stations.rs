//! Usage: cargo run --release --bin stations -- <data dir> <out predictions.json> [radius_m] [public] [classes] [split] [bus_daily]
//! No radius = L0 (every point to its nearest parent). With a radius = L1 (points farther away dropped).
//! `classes` = L7 (two station classes by job/home mass ratio, own profile each).
//! `split` = L4 (a point splits its mass equally among all parents within the radius).
//! `bus_daily` = L3b (plus beta x daily bus activity near the station x its class's whole-day profile).
//! `public` = L2 (each OD pair times the mode-2 share of its home comuna, mode_share.csv).
//! Reads work_od.csv, blocks.csv, jobs.csv and metro_training.json (held-out counts are null there).
#[path = "../stations.rs"]
mod stations;
use std::collections::BTreeMap;
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = Path::new(&a[1]);
    let st: Vec<stations::Station> = serde_json::from_reader(std::fs::File::open(data.join("metro_training.json")).unwrap()).unwrap();
    let t = std::time::Instant::now();
    let (pred, report, _) = stations::predict(data, &st, a.get(3).map_or(f64::INFINITY, |r| r.parse().unwrap()), a.iter().any(|x| x == "public"), a.iter().any(|x| x == "classes"), a.iter().any(|x| x == "split"), a.iter().any(|x| x == "bus_daily"));
    let ms = t.elapsed().as_secs_f64() * 1e3;
    let out: BTreeMap<&str, &Vec<f64>> = st.iter().map(|s| s.name.as_str()).zip(pred.iter()).collect();
    serde_json::to_writer(std::fs::File::create(&a[2]).unwrap(), &out).unwrap();
    println!("{}", serde_json::to_string(&report).unwrap());
    println!("elapsed_ms {ms:.1}");
}
