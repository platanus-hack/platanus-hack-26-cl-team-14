//! Usage: cargo run --release --bin choice_fit -- <data dir>
//! Fits the choice rules of choice.rs and prints the fit and four self-checks.
#[path = "../choice.rs"]
mod choice;
use choice::{Choice, gravity, shares};

fn corr(a: &[f64], b: &[f64]) -> f64 {
    let (ma, mb) = (a.iter().sum::<f64>() / a.len() as f64, b.iter().sum::<f64>() / b.len() as f64);
    let s = |x: &[f64], mx: f64, y: &[f64], my: f64| x.iter().zip(y).map(|(p, q)| (p - mx) * (q - my)).sum::<f64>();
    s(a, ma, b, mb) / (s(a, ma, a, ma) * s(b, mb, b, mb)).sqrt()
}

fn main() {
    let c = Choice::load(std::path::Path::new(&std::env::args().nth(1).unwrap()));
    let n = c.n;
    let ix = |cut: &str| c.cuts.iter().position(|x| x == cut).unwrap();
    let homes: Vec<f64> = c.flows.chunks(n).map(|r| r.iter().sum()).collect(); // workers by home comuna
    let total: f64 = homes.iter().sum();
    let flows = |p: &[f64]| -> Vec<f64> { (0..n * n).map(|i| homes[i / n] * p[i]).collect() };
    let into = |f: &[f64], d: usize| (0..n).map(|o| f[o * n + d]).sum::<f64>();
    let wrong = |f: &[f64]| f.iter().zip(&c.flows).map(|(a, b)| (a - b).abs()).sum::<f64>() / 2.0 / total;

    println!("workers {total} zones {n} minutes: {} pairs from travel_times.csv, {} from the distance fit ({} of them intra-comuna)", n * n - c.filled, c.filled, n);
    println!("jobs.csv total {} (work_od RM total {total})", c.jobs.iter().sum::<f64>());

    println!("\nDESTINATION  beta = {:.5} per minute", c.beta);
    let ones = vec![1.0; n * n];
    for (label, beta) in [("jobs only (beta 0)", 0.0), ("gravity, no k", c.beta)] {
        let f = flows(&gravity(n, &c.jobs, &c.minutes, beta, &ones));
        println!("  {label}: workers misallocated {:.1}%, flow correlation {:.3}", 100.0 * wrong(&f), corr(&f, &c.flows));
    }

    let m = c.mode_coef;
    println!("\nMODE  car = 0, public = {:+.4} {:+.5} × minutes {:+.4} × ln km, other = {:+.4} {:+.4} × ln km", m[0], m[1], m[2], m[3], m[4]);
    let seen: [f64; 3] = [0, 1, 2].map(|j| c.mode_flows.iter().map(|w| w[j]).sum::<f64>() / total);
    println!("  observed shares public {:.3} car {:.3} other {:.3}", seen[0], seen[1], seen[2]);
    for (label, fitted) in [("constant shares", false), ("logit, no residual", true)] {
        let (mut bad, mut a, mut b) = (0.0, vec![], vec![]);
        for (i, w) in c.mode_flows.iter().enumerate() {
            let (p, tot) = (if fitted { shares(&m, c.minutes[i], c.km[i], &[1.0; 3]) } else { seen }, w[0] + w[1] + w[2]);
            bad += (0..3).map(|j| (tot * p[j] - w[j]).abs()).sum::<f64>();
            if tot >= 100.0 { a.push(p[0]); b.push(w[0] / tot); }
        }
        println!("  {label}: commuters in the wrong mode {:.1}%, correlation of the public share on {} pairs with 100+ commuters {:.3}", 100.0 * bad / 2.0 / total, a.len(), corr(&a, &b));
    }

    let t = std::time::Instant::now();
    let (p0, m0) = (c.destinations(&c.jobs, &c.minutes), c.modes(&c.minutes));
    let ms = t.elapsed().as_secs_f64() * 1e3;
    let f0 = flows(&p0);
    println!("\n(a) baseline against work_od.csv: max abs error {:.2e} workers", f0.iter().zip(&c.flows).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max));
    println!("    baseline against mode_od.csv: max abs error {:.2e} commuters", (0..n * n).flat_map(|i| (0..3).map(move |j| i * 3 + j)).map(|q| (f0[q / 3] * m0[q / 3][q % 3] - c.mode_flows[q / 3][q % 3]).abs()).fold(0.0, f64::max));

    let prov = ix("13123");
    let mut jobs = c.jobs.clone();
    jobs[prov] *= 2.0;
    let f = flows(&c.destinations(&jobs, &c.minutes));
    println!("\n(b) jobs of {} × 2: incoming workers {:.0} -> {:.0}", c.names[prov], into(&f0, prov), into(&f, prov));
    let mut loss: Vec<(f64, usize)> = (0..n).map(|d| (into(&f, d) - into(&f0, d), d)).collect();
    loss.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (x, d) in &loss[..5] {
        println!("    {} {:.0} ({:+.1}%)", c.names[*d], x, 100.0 * x / into(&f0, *d));
    }

    let qui = ix("13125");
    let mut minutes = c.minutes.clone();
    (0..n).for_each(|o| minutes[o * n + qui] -= 10.0);
    let (f, m) = (flows(&c.destinations(&c.jobs, &minutes)), c.modes(&minutes));
    let public = |f: &[f64], m: &[[f64; 3]]| (0..n).map(|o| f[o * n + qui] * m[o * n + qui][0]).sum::<f64>() / into(f, qui);
    println!("\n(c) 10 minutes less into {}: incoming workers {:.0} -> {:.0} ({:+.0}), public share {:.3} -> {:.3}", c.names[qui], into(&f0, qui), into(&f, qui), into(&f, qui) - into(&f0, qui), public(&f0, &m0), public(&f, &m));

    println!("\n(d) one destinations + modes call: {ms:.3} ms");
}
