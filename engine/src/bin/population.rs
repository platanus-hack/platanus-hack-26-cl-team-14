//! Build the synthetic person-level state, then run one full day of station boardings from it.
//! Usage: cargo run --release --manifest-path engine/Cargo.toml --bin population -- data/mvp <out predictions.json>
#[path = "../population.rs"]
mod population;
#[path = "../stations.rs"]
mod stations;
use std::collections::BTreeMap;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::path::Path::new(&a[1]);
    let t = std::time::Instant::now();
    let pop = population::Population::build(data);
    let build_ms = t.elapsed().as_secs_f64() * 1e3;
    let model = stations::StationModel::new(data);
    let t = std::time::Instant::now();
    // L4b weighting kept on purpose: each worker counts with the public share of its home comuna (L2), so the
    // persons state can be compared with the accepted fit; students are not in the fit.
    let (mut h, mut j) = (vec![0.0; pop.homes.len()], vec![0.0; pop.jobs.len()]);
    for p in pop.persons.iter().filter(|p| !p.student) {
        let w = model.public_share(&pop.homes[p.home as usize].cut);
        h[p.home as usize] += w;
        j[p.dest as usize] += w;
    }
    let pts = |v: &[population::Point], m: &[f64]| v.iter().zip(m).map(|(p, &x)| (p.longitude, p.latitude, x)).collect::<Vec<_>>();
    let pred = model.project_points(&pts(&pop.homes, &h), &pts(&pop.jobs, &j));
    let day_ms = t.elapsed().as_secs_f64() * 1e3;
    let out: BTreeMap<String, Vec<f64>> = pred.into_iter().map(|(id, _, v)| (id, v)).collect();
    serde_json::to_writer(std::fs::File::create(&a[2]).unwrap(), &out).unwrap();
    let w = pop.persons.iter().filter(|p| !p.student).count();
    println!("persons {} (workers {}, students {}), bytes per person {}, person memory {:.1} MB, build_ms {:.0}, day_ms {:.0}",
        pop.persons.len(), w, pop.persons.len() - w, std::mem::size_of::<population::Person>(),
        (pop.persons.len() * std::mem::size_of::<population::Person>()) as f64 / 1e6, build_ms, day_ms);
}
