//! Any list of changes of the city -> the full day of all persons, as JSON.
//! Usage: cargo run --release --manifest-path engine/Cargo.toml --bin scenario -- data/mvp changes.json [--summary]
//! `-` in place of the file reads the changes from stdin. `--summary` prints only applied, summary and elapsed_ms as text.
#[allow(dead_code)]
#[path = "../choice.rs"]
mod choice;
#[path = "../population.rs"]
mod population;
#[path = "../rechoose.rs"]
mod rechoose;
#[allow(dead_code)]
#[path = "../scenario.rs"]
mod scenario;
#[allow(dead_code)]
#[path = "../stations.rs"]
mod stations;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let t = std::time::Instant::now();
    let sc = scenario::Scenario::load(std::path::Path::new(&args[1]));
    eprintln!("load {} ms", t.elapsed().as_millis());
    let changes: Vec<serde_json::Value> = if args[2] == "-" { serde_json::from_reader(std::io::stdin()) } else { serde_json::from_reader(std::fs::File::open(&args[2]).unwrap()) }.unwrap();
    let r = sc.run(&changes);
    if args.len() < 4 { return println!("{r}"); }
    for a in r["applied"].as_array().unwrap() { println!("applied  {a}"); }
    for a in r["not_modeled"].as_array().unwrap() { println!("not modeled  {a}"); }
    let s = &r["summary"];
    println!("workers  {} -> {} ({} new, {} moved to another job point)", s["workers_base"], s["workers_scenario"], s["workers_new"], s["workers_moved"]);
    println!("boardings/day  {} -> {}", s["boardings_base"], s["boardings_scenario"]);
    for k in ["top_station_gains", "top_station_losses"] {
        println!("{k}");
        for x in s[k].as_array().unwrap() { println!("  {:<28} {:>+10.1}", x["name"].as_str().unwrap(), x["delta_daily"].as_f64().unwrap()); }
    }
    println!("elapsed_ms  {}", r["elapsed_ms"]);
}
