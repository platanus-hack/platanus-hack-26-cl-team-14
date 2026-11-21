//! Build the synthetic person-level state, then run one full day of station boardings from it.
//! Usage: cargo run --release --manifest-path engine/Cargo.toml --bin population -- data/mvp <out predictions.json> [l9|april-transport] [closed parent IDs or line:L7, corridor:L6-east, design:L7-01 ...]
#[path = "../population.rs"]
mod population;
#[path = "../stations.rs"]
mod stations;
use stations::access::planned::{PlannedStations, Selector};
use std::collections::BTreeMap;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::path::Path::new(&a[1]);
    let t = std::time::Instant::now();
    let pop = population::Population::build(data);
    let build_ms = t.elapsed().as_secs_f64() * 1e3;
    let access = a.get(3).map(|layer| stations::access::StationAccess::load_layer(data, layer));
    let walk = access.is_none().then(|| stations::StationModel::new(data));
    let t = std::time::Instant::now();
    // L4b weighting kept on purpose: each worker counts with the public share of its home comuna (L2), so the
    // persons state can be compared with the accepted fit; students are not in the fit.
    let (mut h, mut j) = (vec![0.0; pop.homes.len()], vec![0.0; pop.jobs.len()]);
    let share: Vec<_> = pop.homes.iter().map(|home| match &access {
        Some(model) => model.public_share(&home.cut),
        None => walk.as_ref().unwrap().public_share(&home.cut),
    }).collect();
    for p in pop.persons.iter().filter(|p| !p.student) {
        let w = share[p.home as usize];
        h[p.home as usize] += w;
        j[p.dest as usize] += w;
    }
    let pts = |v: &[population::Point], m: &[f64]| v.iter().zip(m).map(|(p, &x)| (p.longitude, p.latitude, x)).collect::<Vec<_>>();
    let homes = pts(&pop.homes, &h);
    let pred = if let Some(model) = access {
        model.weight_job_masses(pop.jobs.iter().map(|p| (p.id.as_str(), p.cut.as_str())), &mut j);
        let mut active = model.all_parent_ids();
        let mut selectors = Vec::new();
        for arg in &a[4..] {
            match arg.split_once(':') {
                Some(("line", id)) => selectors.push(Selector::Line(id)),
                Some(("corridor", id)) => selectors.push(Selector::Corridor(id)),
                Some(("design", id)) => selectors.push(Selector::Id(id)),
                _ => { active.remove(arg); }
            }
        }
        let additions = if selectors.is_empty() {
            Vec::new()
        } else {
            let catalog = PlannedStations::load(data);
            let unknown: Vec<_> = selectors.iter()
                .filter(|s| catalog.select(std::slice::from_ref(*s)).physical_ids.is_empty())
                .collect();
            if !unknown.is_empty() {
                eprintln!("Unknown design station selectors: {unknown:?}");
                std::process::exit(2);
            }
            catalog.select(&selectors).additions
        };
        model.project_with_additions(&homes, &pts(&pop.jobs, &j), &active, &additions)
    } else {
        walk.unwrap().project_points(&homes, &pts(&pop.jobs, &j))
    };
    let day_ms = t.elapsed().as_secs_f64() * 1e3;
    let out: BTreeMap<String, Vec<f64>> = pred.into_iter().map(|(id, _, v)| (id, v)).collect();
    serde_json::to_writer(std::fs::File::create(&a[2]).unwrap(), &out).unwrap();
    let w = pop.persons.iter().filter(|p| !p.student).count();
    println!("persons {} (workers {}, students {}), bytes per person {}, person memory {:.1} MB, build_ms {:.0}, day_ms {:.0}",
        pop.persons.len(), w, pop.persons.len() - w, std::mem::size_of::<population::Person>(),
        (pop.persons.len() * std::mem::size_of::<population::Person>()) as f64 / 1e6, build_ms, day_ms);
}
