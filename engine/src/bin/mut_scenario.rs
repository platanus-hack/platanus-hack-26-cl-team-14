//! Manual run of the MUT station scenario: cargo run --release --bin mut_scenario -- <data dir> '<request json>'
#[path = "../stations.rs"]
mod stations;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let model = stations::StationModel::new(std::path::Path::new(&a[1]));
    let t = std::time::Instant::now();
    let r = model.mut_scenario(serde_json::from_str(&a[2]).unwrap());
    eprintln!("elapsed_ms {:.3}", t.elapsed().as_secs_f64() * 1e3);
    match r { Ok(v) => println!("{}", serde_json::to_string(&v).unwrap()), Err(e) => println!("422 {e}") }
}
