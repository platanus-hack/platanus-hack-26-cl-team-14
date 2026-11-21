//! L0 station prediction (clever-heron's v0 contract, 06:58 UTC): work mass from the internal
//! work OD, split inside each comuna by block population (homes) and by modeled jobs (work places),
//! each point to its nearest Metro parent. 00:00-12:00 from homes, 12:00-24:00 from jobs, times a
//! training-only half-hour profile, times one scale fitted on training stations.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize)]
struct Od { origin_cut: String, destination_cut: String, workers: f64 }
#[derive(Deserialize)]
struct Mode { cut: String, mode: i32, commuters: f64 }
#[derive(Deserialize)]
struct Block { cut: String, longitude: f64, latitude: f64, population: f64 }
#[derive(Deserialize)]
struct Job { cut: String, longitude: f64, latitude: f64, jobs: f64 }
#[derive(Deserialize)]
pub struct Station { pub name: String, longitude: f64, latitude: f64, pub holdout: bool, laboral: Option<Vec<Option<f64>>> }

#[derive(Serialize)]
pub struct Report { pub scale: f64, pub class_cut: f64, pub training_class_counts: [usize; 2], pub heldout_class_counts: [usize; 2], pub profile_max_station_share: [[f64; 2]; 2], pub training_stations: usize, pub training_cells: usize, pub profile_support: Vec<usize>, pub support_by_class: Vec<Vec<usize>>, pub distance_m_quantiles: [f64; 5], pub home_mass: f64, pub job_mass: f64 }

fn rows<T: for<'de> Deserialize<'de>>(p: &Path) -> Vec<T> {
    csv::Reader::from_path(p).unwrap().deserialize().map(|r| r.unwrap()).collect()
}

/// Nearest parent by local flat metres; returns (index, distance).
fn nearest(st: &[Station], x: f64, y: f64) -> (usize, f64) {
    let kx = 111_320.0 * (-33.45f64).to_radians().cos();
    st.iter().enumerate()
        .map(|(i, s)| (i, ((s.longitude - x) * kx).hypot((s.latitude - y) * 110_540.0)))
        .min_by(|a, b| a.1.total_cmp(&b.1)).unwrap()
}

/// Spread each comuna total over its points by weight, add each point's share to its nearest station.
/// Points farther than `radius` metres from their nearest parent add nothing (L1); their mass is not redistributed.
fn mass(st: &[Station], total: &HashMap<String, f64>, pts: &[(String, f64, f64, f64)], radius: f64, dist: &mut Vec<(f64, f64)>) -> Vec<f64> {
    let mut wsum: HashMap<&str, f64> = HashMap::new();
    for p in pts { *wsum.entry(&p.0).or_default() += p.3; }
    let mut m = vec![0.0; st.len()];
    for p in pts {
        if p.3 == 0.0 { continue; }
        let v = total.get(&p.0).copied().unwrap_or(0.0) * p.3 / wsum[p.0.as_str()];
        let (i, d) = nearest(st, p.1, p.2);
        dist.push((d, v));
        if d <= radius { m[i] += v; }
    }
    m
}

/// `classes`: L7, two station classes with their own profile.
/// `public_share`: L2, weight each OD pair by the share of public-transport commuters (mode 2) of its home comuna.
pub fn predict(data: &Path, st: &[Station], radius: f64, public_share: bool, classes: bool) -> (Vec<Vec<f64>>, Report) {
    let mut share: HashMap<String, (f64, f64)> = HashMap::new();
    for m in rows::<Mode>(&data.join("mode_share.csv")) {
        let e = share.entry(m.cut).or_default();
        e.1 += m.commuters;
        if m.mode == 2 { e.0 += m.commuters; }
    }
    let (mut home, mut work) = (HashMap::new(), HashMap::new());
    for f in rows::<Od>(&data.join("work_od.csv")) {
        if !f.destination_cut.starts_with("13") { continue; } // RM-internal only, as main.rs
        let m = if public_share { let (p, t) = share[&f.origin_cut]; p / t } else { 1.0 };
        *home.entry(f.origin_cut).or_insert(0.0) += f.workers * m;
        *work.entry(f.destination_cut).or_insert(0.0) += f.workers * m;
    }
    let blocks: Vec<_> = rows::<Block>(&data.join("blocks.csv")).into_iter().map(|b| (b.cut, b.longitude, b.latitude, b.population)).collect();
    let jobs: Vec<_> = rows::<Job>(&data.join("jobs.csv")).into_iter().map(|j| (j.cut, j.longitude, j.latitude, j.jobs)).collect();
    let mut dist = Vec::new();
    let h = mass(st, &home, &blocks, radius, &mut dist);
    let j = mass(st, &work, &jobs, radius, &mut dist);

    // L7: class 1 ("work") if J/H inside the radius is above the training median, else class 0 ("home").
    let ratio: Vec<f64> = (0..st.len()).map(|i| if h[i] == 0.0 { f64::INFINITY } else { j[i] / h[i] }).collect();
    let mut tr: Vec<f64> = (0..st.len()).filter(|&i| !st[i].holdout).map(|i| ratio[i]).collect();
    tr.sort_by(f64::total_cmp);
    let cut = (tr[(tr.len() - 1) / 2] + tr[tr.len() / 2]) / 2.0;
    let class: Vec<usize> = ratio.iter().map(|&r| (classes && r > cut) as usize).collect();

    // Profile p_t per class: mean over training stations observed at t, normalized to 1 inside each half.
    let mut p = vec![vec![0.0; 48]; 2];
    let mut support = vec![0; 48];
    let mut support_by_class = vec![vec![0; 48]; 2];
    for c in 0..2 {
        let train: Vec<&Vec<Option<f64>>> = (0..st.len()).filter(|&i| !st[i].holdout && class[i] == c).map(|i| st[i].laboral.as_ref().unwrap()).collect();
        if train.is_empty() { continue; }
        for t in 0..48 {
            let v: Vec<f64> = train.iter().filter_map(|b| b[t]).collect();
            support[t] += v.len();
            support_by_class[c][t] = v.len();
            if !v.is_empty() { p[c][t] = v.iter().sum::<f64>() / v.len() as f64; }
        }
        for half in [0..24, 24..48] {
            let s: f64 = p[c][half.clone()].iter().sum();
            for t in half { p[c][t] /= s; }
        }
    }
    let train_count = st.iter().filter(|s| !s.holdout).count();
    // Each observation contributes y / observed station count to its class-bin mean.
    let mut maxshare = [[0.0; 2]; 2];
    for c in 0..2 {
        for (k, half) in [0..24, 24..48].into_iter().enumerate() {
            let m: Vec<f64> = (0..st.len()).filter(|&i| !st[i].holdout && class[i] == c)
                .map(|i| half.clone().filter_map(|t| st[i].laboral.as_ref().unwrap()[t]
                    .map(|y| y / support_by_class[c][t] as f64)).sum()).collect();
            let tot: f64 = m.iter().sum();
            if tot > 0.0 { maxshare[c][k] = m.iter().cloned().fold(0.0, f64::max) / tot; }
        }
    }
    let q: Vec<Vec<f64>> = (0..st.len()).map(|i| (0..48).map(|t| p[class[i]][t] * if t < 24 { h[i] } else { j[i] }).collect()).collect();

    // One scale over the observed training cells.
    let (mut obs, mut mod_, mut cells) = (0.0, 0.0, 0);
    for (i, s) in st.iter().enumerate().filter(|(_, s)| !s.holdout) {
        for (t, o) in s.laboral.as_ref().unwrap().iter().enumerate() {
            if let Some(o) = o { obs += o; mod_ += q[i][t]; cells += 1; }
        }
    }
    let a = obs / mod_;
    dist.sort_by(|x, y| x.0.total_cmp(&y.0));
    let w: f64 = dist.iter().map(|d| d.1).sum();
    let mut qs = [0.0; 5];
    for (k, f) in [0.1, 0.25, 0.5, 0.75, 0.9].iter().enumerate() {
        let mut acc = 0.0;
        qs[k] = dist.iter().find(|d| { acc += d.1; acc >= f * w }).unwrap().0.round();
    }
    let count = |h: bool| { let mut c = [0; 2]; for i in 0..st.len() { if st[i].holdout == h { c[class[i]] += 1; } } c };
    let report = Report { scale: a, class_cut: cut, training_class_counts: count(false), heldout_class_counts: count(true), profile_max_station_share: maxshare, training_stations: train_count, training_cells: cells, profile_support: support, support_by_class,
        distance_m_quantiles: qs, home_mass: h.iter().sum(), job_mass: j.iter().sum() };
    (q.into_iter().map(|r| r.into_iter().map(|v| v * a).collect()).collect(), report)
}
