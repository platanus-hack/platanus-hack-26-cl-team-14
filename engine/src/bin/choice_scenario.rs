//! The persons choose again after three generic changes of the city. Prints the station changes of each one.
//! Usage: cargo run --release --manifest-path engine/Cargo.toml --bin choice_scenario -- data/mvp
//! Limits: minutes are comuna to comuna and biased low (0.65×), so (c) is an upper value. The mode rule is weak,
//! and the station projection weights each worker with the public share of its home comuna (L4b), not with `mode`.
#[allow(dead_code)]
#[path = "../choice.rs"]
mod choice;
#[path = "../population.rs"]
mod population;
#[path = "../rechoose.rs"]
mod rechoose;
#[path = "../stations.rs"]
mod stations;
use population::{Point, Population};
use std::time::Instant;

/// Daily boardings of each station: the projection of the `population` binary.
fn day(model: &stations::StationModel, pop: &Population) -> Vec<(String, f64)> {
    let (mut h, mut j) = (vec![0.0; pop.homes.len()], vec![0.0; pop.jobs.len()]);
    for p in pop.persons.iter().filter(|p| !p.student) {
        let w = model.public_share(&pop.homes[p.home as usize].cut);
        h[p.home as usize] += w;
        j[p.dest as usize] += w;
    }
    let pts = |v: &[Point], m: &[f64]| v.iter().zip(m).map(|(p, &x)| (p.longitude, p.latitude, x)).collect::<Vec<_>>();
    model.project_points(&pts(&pop.homes, &h), &pts(&pop.jobs, &j)).into_iter().map(|(_, name, v)| (name, v.iter().sum())).collect()
}

fn main() {
    let arg = std::env::args().nth(1).unwrap();
    let data = std::path::Path::new(&arg);
    let c = choice::Choice::load(data);
    let n = c.n;
    let mut pop = Population::build(data);
    let model = stations::StationModel::new(data);
    let (start, base, placed) = (pop.persons.clone(), day(&model, &pop), rechoose::Base::new(&c, &pop));
    let weights: Vec<f64> = pop.jobs.iter().map(|p| p.weight).collect();
    let public = |pop: &Population| pop.persons.iter().filter(|p| p.mode == 2).count() as i64;
    let od = |pop: &Population| {
        let ix = |cut: &str| c.cuts.iter().position(|x| x == cut).unwrap();
        let (h, j): (Vec<usize>, Vec<usize>) = (pop.homes.iter().map(|p| ix(&p.cut)).collect(), pop.jobs.iter().map(|p| ix(&p.cut)).collect());
        let mut f = vec![0.0; n * n];
        for p in pop.persons.iter().filter(|p| !p.student) { f[h[p.home as usize] * n + j[p.dest as usize]] += 1.0; }
        f
    };
    let far = |a: &[f64], b: &[f64]| a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max);
    let (od0, public0) = (od(&pop), public(&pop));
    println!("workers {}, job points {}, stations {}", od0.iter().sum::<f64>(), pop.jobs.len(), base.len());

    let run = |label: &str, pop: &mut Population, weights: &[f64], minutes: &[f64]| {
        pop.persons.clone_from(&start);
        let t = Instant::now();
        let moved = rechoose::rechoose(&c, pop, weights, minutes, &placed);
        let choose_ms = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        let after = day(&model, pop);
        println!("\n{label}\n  workers with a new dest {moved}, public commuters {:+}, rechoose {choose_ms:.0} ms, station projection {:.0} ms",
            public(pop) - public0, t.elapsed().as_secs_f64() * 1e3);
        after
    };

    let same = run("NO CHANGE", &mut pop, &weights, &c.minutes);
    println!("  comuna OD totals: max abs error {} against the persons before, {} against work_od.csv", far(&od(&pop), &od0), far(&od(&pop), &c.flows));
    let (b, s): (Vec<f64>, Vec<f64>) = (base.iter().map(|x| x.1).collect(), same.iter().map(|x| x.1).collect());
    println!("  daily boardings: max abs difference of a station {:.2}, total {:.2} -> {:.2}", far(&b, &s), b.iter().sum::<f64>(), s.iter().sum::<f64>());

    let top = |after: Vec<(String, f64)>| {
        let mut d: Vec<(f64, &str, f64)> = after.iter().zip(&base).map(|(a, b)| (a.1 - b.1, a.0.as_str(), b.1)).collect();
        d.sort_by(|a, b| b.0.abs().total_cmp(&a.0.abs()));
        for (delta, name, before) in &d[..8] { println!("  {name:<28} {before:>9.0} {delta:>+8.0}"); }
        println!("  all stations {:+.0}", d.iter().map(|x| x.0).sum::<f64>());
    };

    let mut w = weights.clone();
    w[pop.jobs.iter().position(|p| p.id == "13114131009033").unwrap()] = 0.0;
    top(run("(a) NO JOBS ON THE MUT BLOCK 13114131009033 (Tobalaba reference: -2,781 isolated station scenario, -2,701 block stock run)", &mut pop, &w, &c.minutes));

    let q = c.cuts.iter().position(|x| x == "13125").unwrap();
    let com = csv::Reader::from_path(data.join("communes.csv")).unwrap().into_records().nth(q).unwrap().unwrap();
    pop.jobs.push(Point { id: "new".into(), cut: "13125".into(), longitude: com[5].parse().unwrap(), latitude: com[6].parse().unwrap(), weight: 20000.0 });
    let mut w = weights.clone();
    w.push(20000.0);
    top(run("(b) 20,000 NEW JOBS AT ONE POINT IN QUILICURA", &mut pop, &w, &c.minutes));
    pop.jobs.pop();

    let mut minutes = c.minutes.clone();
    for o in (0..n).filter(|&o| o != q) {
        minutes[o * n + q] -= 10.0;
        minutes[q * n + o] -= 10.0;
    }
    top(run("(c) 10 MINUTES LESS ON EACH PAIR INTO AND OUT OF QUILICURA (upper value)", &mut pop, &weights, &minutes));
}
