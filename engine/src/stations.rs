#![allow(dead_code)] // each binary uses a different half of this module
//! L0 station prediction (clever-heron's v0 contract, 06:58 UTC): work mass from the internal
//! work OD, split inside each comuna by block population (homes) and by modeled jobs (work places),
//! each point to its nearest Metro parent. 00:00-12:00 from homes, 12:00-24:00 from jobs, times a
//! training-only half-hour profile, times one scale fitted on training stations.
#[path = "station_access.rs"]
pub mod access;

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
pub struct Station { pub id: String, pub name: String, longitude: f64, latitude: f64, pub holdout: bool, laboral: Option<Vec<Option<f64>>> }

#[derive(Serialize)]
pub struct Report { pub scale: f64, pub class_cut: f64, pub training_class_counts: [usize; 2], pub heldout_class_counts: [usize; 2], pub profile_max_station_share: [[f64; 2]; 2], pub training_stations: usize, pub training_cells: usize, pub profile_support: Vec<usize>, pub support_by_class: Vec<Vec<usize>>, pub distance_m_quantiles: [f64; 5], pub home_mass: f64, pub job_mass: f64, pub mass_share_with_2_or_more_parents: [f64; 2] }

fn rows<T: for<'de> Deserialize<'de>>(p: &Path) -> Vec<T> {
    csv::Reader::from_path(p).unwrap().deserialize().map(|r| r.unwrap()).collect()
}

fn metres(lon: f64, lat: f64, x: f64, y: f64) -> f64 {
    let kx = 111_320.0 * (-33.45f64).to_radians().cos();
    ((lon - x) * kx).hypot((lat - y) * 110_540.0)
}

/// Nearest parent by local flat metres; returns (index, distance).
fn nearest(st: &[Station], x: f64, y: f64) -> (usize, f64) {
    st.iter().enumerate()
        .map(|(i, s)| (i, metres(s.longitude, s.latitude, x, y)))
        .min_by(|a, b| a.1.total_cmp(&b.1)).unwrap()
}

/// Parents within `radius` of (x, y) with raw weights and their sum: 1 each (L4), or 1/max(d, 50 m) (L4b).
/// The share of parent k is w_k / sum.
fn parents_in_reach(st: &[Station], x: f64, y: f64, radius: f64, inverse: bool) -> (Vec<(usize, f64)>, f64) {
    let near: Vec<(usize, f64)> = (0..st.len()).map(|k| (k, metres(st[k].longitude, st[k].latitude, x, y)))
        .filter(|&(_, d)| d <= radius).map(|(k, d)| (k, if inverse { 1.0 / d.max(50.0) } else { 1.0 })).collect();
    let tot = near.iter().map(|n| n.1).sum();
    (near, tot)
}

type PointMass = (f64, f64, f64);

fn point_masses(total: &HashMap<String, f64>, pts: &[(String, f64, f64, f64)]) -> Vec<PointMass> {
    let mut wsum: HashMap<&str, f64> = HashMap::new();
    for p in pts { *wsum.entry(&p.0).or_default() += p.3; }
    pts.iter().filter(|p| p.3 != 0.0).map(|p| {
        (p.1, p.2, total.get(&p.0).copied().unwrap_or(0.0) * p.3 / wsum[p.0.as_str()])
    }).collect()
}

/// Spread each comuna total over its points by weight, add each point's share to its nearest station.
/// Points farther than `radius` metres from their nearest parent add nothing (L1); their mass is not redistributed.
/// `split` (L4): a point gives mass/n to each of the n parents within `radius`, not all to the nearest.
/// Returns the station masses and the share of assigned mass whose point has 2 or more parents within the radius.
fn mass(st: &[Station], pts: &[PointMass], radius: f64, split: bool, inverse: bool, dist: &mut Vec<(f64, f64)>) -> (Vec<f64>, f64) {
    let mut m = vec![0.0; st.len()];
    let (mut multi, mut assigned) = (0.0, 0.0);
    for &(x, y, v) in pts {
        let (i, d) = nearest(st, x, y);
        dist.push((d, v));
        if !split { if d <= radius { m[i] += v; } continue; }
        let (near, tot) = parents_in_reach(st, x, y, radius, inverse);
        if !near.is_empty() { assigned += v; }
        if near.len() >= 2 { multi += v; }
        for &(k, w) in &near { m[k] += v * w / tot; }
    }
    (m, if assigned > 0.0 { multi / assigned } else { 0.0 })
}

fn baseline_points(data: &Path, share: &HashMap<String, (f64, f64)>, public_share: bool) -> (Vec<PointMass>, Vec<PointMass>) {
    let (mut home, mut work) = (HashMap::new(), HashMap::new());
    for f in rows::<Od>(&data.join("work_od.csv")) {
        if !f.destination_cut.starts_with("13") { continue; } // RM-internal only, as main.rs
        let m = if public_share { let (p, t) = share[&f.origin_cut]; p / t } else { 1.0 };
        *home.entry(f.origin_cut).or_insert(0.0) += f.workers * m;
        *work.entry(f.destination_cut).or_insert(0.0) += f.workers * m;
    }
    let blocks: Vec<_> = rows::<Block>(&data.join("blocks.csv")).into_iter().map(|b| (b.cut, b.longitude, b.latitude, b.population)).collect();
    let jobs: Vec<_> = rows::<Job>(&data.join("jobs.csv")).into_iter().map(|j| (j.cut, j.longitude, j.latitude, j.jobs)).collect();
    (point_masses(&home, &blocks), point_masses(&work, &jobs))
}

/// `classes`: L7, two station classes with their own profile.
/// `public_share`: L2, weight each OD pair by the share of public-transport commuters (mode 2) of its home comuna.
/// `split`: L4, equal split of each point among all parents within the radius.
/// `inverse`: L4b, with `split`, shares are 1/max(d, 50 m) normalized instead of equal parts.
pub fn predict(data: &Path, st: &[Station], radius: f64, public_share: bool, classes: bool, split: bool, inverse: bool) -> (Vec<Vec<f64>>, Report, Fit) {
    let mut share: HashMap<String, (f64, f64)> = HashMap::new();
    for m in rows::<Mode>(&data.join("mode_share.csv")) {
        let e = share.entry(m.cut).or_default();
        e.1 += m.commuters;
        if m.mode == 2 { e.0 += m.commuters; }
    }
    let (homes, jobs) = baseline_points(data, &share, public_share);
    let mut dist = Vec::new();
    let (h, multi_h) = mass(st, &homes, radius, split, inverse, &mut dist);
    let (j, multi_j) = mass(st, &jobs, radius, split, inverse, &mut dist);

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
        distance_m_quantiles: qs, home_mass: h.iter().sum(), job_mass: j.iter().sum(), mass_share_with_2_or_more_parents: [multi_h, multi_j] };
    (q.into_iter().map(|r| r.into_iter().map(|v| v * a).collect()).collect(), report, Fit { class, p, a, radius, share, public_share, split, inverse })
}

/// The frozen parts of a fit that a scenario reuses: baseline classes, profiles, scale, radius, mode shares.
pub struct Fit { class: Vec<usize>, p: Vec<Vec<f64>>, a: f64, radius: f64, share: HashMap<String, (f64, f64)>, public_share: bool, split: bool, inverse: bool }

/// Block-level stocks of a scenario for `StationModel::project`.
#[derive(Deserialize, Default)]
pub struct BlockStock {
    #[serde(default)] pub population: HashMap<String, f64>,
    #[serde(default)] pub jobs: HashMap<String, f64>,
    /// (cut, longitude, latitude, population)
    #[serde(default)] pub new_homes: Vec<(String, f64, f64, f64)>,
}
#[derive(Deserialize)]
struct BlockId { block_id: String, cut: String, longitude: f64, latitude: f64, population: f64 }
#[derive(Deserialize)]
struct JobId { block_id: String, cut: String, longitude: f64, latitude: f64, jobs: f64 }

const MUT_BLOCK: &str = "13114131009033";
const MUT_CUT: &str = "13114";

#[derive(Deserialize)]
struct ArtifactStation { id: String, predicted: Vec<f64> }
#[derive(Deserialize)]
struct Artifact { layer: String, stations_args: Vec<String>, wape_bins: f64, #[serde(default)] model_source_sha256: String,
    #[serde(default)] input_sha256: HashMap<String, String>, #[serde(default)] target_sha256: String,
    #[serde(default)] scorer_sha256: String, #[serde(default)] observed_cells: usize, stations: Vec<ArtifactStation> }

/// Frozen score: target bytes of PR #35 and the scorer of PR #42 (tools/pack/check_score.py).
const TARGET_SHA256: &str = "ecd8131fac2bfcf401bb5700dda7a7a0a31534990b8b962134d31449651ef5a3";
const SCORER_SHA256: &str = "ea807561049a3e62a847455b98863594744ae1768201a498f3d720af2991dcaa";

/// Files the fit reads; their hashes must equal the artifact's before its score is shown.
const FIT_INPUTS: [&str; 5] = ["work_od.csv", "mode_share.csv", "blocks.csv", "jobs.csv", "metro_training.json"];

fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Same predictions up to floating-point roundoff between platforms: 48 finite values, each within 1e-9 boardings
/// (the Linux/macOS gap seen for L4b was 2.3e-13; real model changes move values by far more).
fn same_vector(a: &[f64], b: &[f64]) -> bool {
    a.len() == 48 && b.len() == 48 && a.iter().zip(b).all(|(x, y)| x.is_finite() && y.is_finite() && (x - y).abs() <= 1e-9)
}

fn layer_of(args: &[String]) -> Option<&'static str> {
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => Some("L0"), ["1000"] => Some("L1"), ["1000", "public"] => Some("L2"), ["1000", "public", "classes"] => Some("L7"), ["1000", "public", "classes", "split"] => Some("L4"), ["1000", "public", "classes", "split", "inverse"] => Some("L4b"), _ => None,
    }
}
#[derive(Deserialize)]
struct Commune { cut: String, population: f64 }
#[derive(Deserialize)]
struct JobRow { block_id: String, longitude: f64, latitude: f64, jobs: f64 }

/// The accepted station model, fitted once with the arguments of `station_score_run.json`.
pub struct StationModel {
    st: Vec<Station>, base: Vec<Vec<f64>>, fit: Fit, layer: String, args: Vec<String>, wape_bins: Option<f64>,
    blocks: Vec<(String, String, f64, f64, f64)>, jobs: Vec<(String, String, f64, f64, f64)>,
    mut_point: (f64, f64), mut_jobs: f64, commuters_per_resident: f64, home_public_share: f64, job_public_factor: f64,
}

#[derive(Deserialize)]
pub struct MutRequest { pub site_id: String, pub dwellings_added: f64, pub jobs_removed: f64, pub occupied_share: f64, pub persons_per_occupied_dwelling: f64, pub occupancy_source: String }

#[derive(Serialize)]
pub struct HomeConversion { occupied_share: f64, persons_per_occupied_dwelling: f64, occupancy_source: String, commuters_per_resident: f64, public_share: f64 }
#[derive(Serialize)]
pub struct StationChange { id: String, name: String, predicted_base: Vec<f64>, predicted_scenario: Vec<f64> }
#[derive(Serialize)]
pub struct MutResponse {
    site_id: String, dwellings_added: f64, jobs_removed: f64, layer: String, model_args: Vec<String>,
    /// None when the artifact does not hold this exact fit (other arguments or other inputs).
    wape_bins: Option<f64>, scope_label: &'static str, assumptions: [&'static str; 3], delta_home_mass: f64, delta_job_mass: f64,
    home_conversion: HomeConversion, job_public_factor: f64, stations: Vec<StationChange>,
}

impl StationModel {
    pub fn new(data: &Path) -> StationModel {
        let art: Artifact = serde_json::from_reader(std::fs::File::open(data.join("station_score_run.json")).unwrap()).unwrap();
        let st: Vec<Station> = serde_json::from_reader(std::fs::File::open(data.join("metro_training.json")).unwrap()).unwrap();
        let has = |k: &str| art.stations_args.iter().any(|x| x == k);
        let radius = art.stations_args.first().and_then(|r| r.parse().ok()).unwrap_or(f64::INFINITY);
        let (base, _, fit) = predict(data, &st, radius, has("public"), has("classes"), has("split"), has("inverse"));
        // The stored score belongs to this fit only if: same model source, frozen target bytes and scorer, 1,123 cells, same input bytes, the label of these
        // arguments, all 31 frozen held-out stations present, and exactly the same held-out predictions.
        let held: std::collections::BTreeSet<&str> = st.iter().filter(|s| s.holdout).map(|s| s.id.as_str()).collect();
        let listed: std::collections::BTreeSet<&str> = art.stations.iter().map(|a| a.id.as_str()).collect();
        let same = art.model_source_sha256 == sha256(include_bytes!("stations.rs"))
            && art.target_sha256 == TARGET_SHA256 && sha256(&std::fs::read(data.join("metro_observations.json")).unwrap()) == TARGET_SHA256
            && art.scorer_sha256 == SCORER_SHA256 && art.observed_cells == 1123
            && FIT_INPUTS.iter().all(|f| art.input_sha256.get(*f) == Some(&sha256(&std::fs::read(data.join(f)).unwrap())))
            && layer_of(&art.stations_args) == Some(art.layer.as_str())
            && held.len() == 31 && listed == held && art.stations.len() == 31
            && art.stations.iter().all(|a| st.iter().position(|s| s.id == a.id).is_some_and(|i| same_vector(&base[i], &a.predicted)));
        let m = |cut: &str| if fit.public_share { let (p, t) = fit.share[cut]; p / t } else { 1.0 };
        let (mut out, mut inb, mut inb_w) = (0.0, 0.0, 0.0);
        for f in rows::<Od>(&data.join("work_od.csv")) {
            if !f.destination_cut.starts_with("13") { continue; }
            if f.origin_cut == MUT_CUT { out += f.workers; }
            if f.destination_cut == MUT_CUT { inb += f.workers; inb_w += f.workers * m(&f.origin_cut); }
        }
        let pop = rows::<Commune>(&data.join("communes.csv")).into_iter().find(|c| c.cut == MUT_CUT).unwrap().population;
        let row = rows::<JobRow>(&data.join("jobs.csv")).into_iter().find(|j| j.block_id == MUT_BLOCK).unwrap();
        let blocks = rows::<BlockId>(&data.join("blocks.csv")).into_iter().map(|b| (b.block_id, b.cut, b.longitude, b.latitude, b.population)).collect();
        let jobs = rows::<JobId>(&data.join("jobs.csv")).into_iter().map(|j| (j.block_id, j.cut, j.longitude, j.latitude, j.jobs)).collect();
        StationModel {
            layer: art.layer, args: art.stations_args, wape_bins: same.then_some(art.wape_bins), blocks, jobs,
            mut_point: (row.longitude, row.latitude), mut_jobs: row.jobs, commuters_per_resident: out / pop,
            home_public_share: m(MUT_CUT), job_public_factor: inb_w / inb, st, base, fit,
        }
    }

    /// Station boardings for any RM-internal OD flows `(origin_cut, destination_cut, workers)`, through the frozen
    /// fit: only the home and job masses are recomputed (same public share, point split and 1/d shares); scale,
    /// classes and profiles stay those of the baseline fit. With the baseline `work_od.csv` flows, in file order,
    /// it returns the baseline predictions exactly. On changed flows it is a projection, not a validated forecast.
    /// `stock` holds the block-level state of the same scenario: a block's new population or jobs replaces the file
    /// value (the weight that splits its comuna's mass), and `new_homes` adds home points (e.g. new dwellings on a
    /// block with no residents). Empty stock = the file weights.
    pub fn project(&self, flows: &[(String, String, f64)], stock: &BlockStock) -> Vec<(String, String, Vec<f64>)> {
        let (mut home, mut work) = (HashMap::new(), HashMap::new());
        for (o, d, w) in flows {
            if !d.starts_with("13") { continue; }
            let m = if self.fit.public_share { let (p, t) = self.fit.share[o]; p / t } else { 1.0 };
            *home.entry(o.clone()).or_insert(0.0) += w * m;
            *work.entry(d.clone()).or_insert(0.0) += w * m;
        }
        let pts = |v: &[(String, String, f64, f64, f64)], o: &HashMap<String, f64>| -> Vec<(String, f64, f64, f64)> {
            v.iter().map(|(id, c, x, y, w)| (c.clone(), *x, *y, *o.get(id).unwrap_or(w))).collect()
        };
        let mut homes = pts(&self.blocks, &stock.population);
        homes.extend(stock.new_homes.iter().cloned());
        let mut dist = Vec::new();
        let (h, _) = mass(&self.st, &point_masses(&home, &homes), self.fit.radius, self.fit.split, self.fit.inverse, &mut dist);
        let (j, _) = mass(&self.st, &point_masses(&work, &pts(&self.jobs, &stock.jobs)), self.fit.radius, self.fit.split, self.fit.inverse, &mut dist);
        self.st.iter().enumerate().map(|(i, s)| {
            let p = &self.fit.p[self.fit.class[i]];
            (s.id.clone(), s.name.clone(), (0..48).map(|t| p[t] * if t < 24 { h[i] } else { j[i] } * self.fit.a).collect())
        }).collect()
    }

    /// Station boardings from home and job masses already placed on points `(longitude, latitude, mass)`, e.g.
    /// aggregated from a person-level state, through the same 1/d shares and the frozen scale, classes and profiles.
    pub fn project_points(&self, homes: &[(f64, f64, f64)], jobs: &[(f64, f64, f64)]) -> Vec<(String, String, Vec<f64>)> {
        let place = |pts: &[(f64, f64, f64)]| {
            let mut m = vec![0.0; self.st.len()];
            for &(x, y, v) in pts {
                if v == 0.0 { continue; }
                let (near, tot) = parents_in_reach(&self.st, x, y, self.fit.radius, self.fit.inverse);
                for (k, w) in near { m[k] += v * w / tot; }
            }
            m
        };
        let (h, j) = (place(homes), place(jobs));
        self.st.iter().enumerate().map(|(i, s)| {
            let p = &self.fit.p[self.fit.class[i]];
            (s.id.clone(), s.name.clone(), (0..48).map(|t| p[t] * if t < 24 { h[i] } else { j[i] } * self.fit.a).collect())
        }).collect()
    }

    /// The mode-2 share of a home comuna in the fit (L2), for weighting persons the way the fit weighted flows.
    pub fn public_share(&self, cut: &str) -> f64 { if self.fit.public_share { let (p, t) = self.fit.share[cut]; p / t } else { 1.0 } }

    /// MUT only: new dwellings become home mass, removed jobs leave job mass, both at the MUT jobs row,
    /// through the frozen fit. The MUT parent is always returned; at zero change both arrays are equal.
    pub fn mut_scenario(&self, r: MutRequest) -> Result<MutResponse, String> {
        if r.site_id != "MUT" { return Err("site_id must be MUT".into()); }
        if !(r.dwellings_added >= 0.0 && r.dwellings_added.is_finite()) { return Err("dwellings_added must be finite and >= 0".into()); }
        if !(0.0..=self.mut_jobs).contains(&r.jobs_removed) { return Err(format!("jobs_removed must be in 0..={}", self.mut_jobs)); }
        if !(0.0..=1.0).contains(&r.occupied_share) { return Err("occupied_share must be in 0..=1".into()); }
        if !(r.persons_per_occupied_dwelling >= 0.0 && r.persons_per_occupied_dwelling.is_finite()) { return Err("persons_per_occupied_dwelling must be finite and >= 0".into()); }
        let dh = r.dwellings_added * r.occupied_share * r.persons_per_occupied_dwelling * self.commuters_per_resident * self.home_public_share;
        let dj = -r.jobs_removed * self.job_public_factor;
        // Same assignment rule as the fit: nearest parent, or (L4/L4b) a split among all parents within the radius.
        let (i, d) = nearest(&self.st, self.mut_point.0, self.mut_point.1);
        let near: Vec<(usize, f64)> = if self.fit.split {
            let (n, tot) = parents_in_reach(&self.st, self.mut_point.0, self.mut_point.1, self.fit.radius, self.fit.inverse);
            n.into_iter().map(|(k, w)| (k, w / tot)).collect()
        } else if d <= self.fit.radius { vec![(i, 1.0)] } else { vec![] };
        let mut stations = Vec::new();
        for &(i, w) in &near {
            let p = &self.fit.p[self.fit.class[i]];
            let scen = (0..48).map(|t| self.base[i][t] + self.fit.a * p[t] * w * if t < 24 { dh } else { dj }).collect();
            stations.push(StationChange { id: self.st[i].id.clone(), name: self.st[i].name.clone(), predicted_base: self.base[i].clone(), predicted_scenario: scen });
        }
        Ok(MutResponse {
            site_id: r.site_id, dwellings_added: r.dwellings_added, jobs_removed: r.jobs_removed, layer: self.layer.clone(),
            model_args: self.args.clone(), wape_bins: self.wape_bins, scope_label: "error del modelo base, no del escenario",
            assumptions: [
                "The scale was fitted so that work mass explains all boardings of a station; applying it to removed jobs is an unvalidated sensitivity, not a per-worker Metro probability.",
                "Mode 2 shares are transferred from home comunas (residents) and from origin comunas (arriving workers); they are not observed at this site.",
                "jobs_removed are modeled jobs of jobs.csv; mall visitors and other trips are not in the model.",
            ],
            delta_home_mass: dh, delta_job_mass: dj,
            home_conversion: HomeConversion { occupied_share: r.occupied_share, persons_per_occupied_dwelling: r.persons_per_occupied_dwelling,
                occupancy_source: r.occupancy_source, commuters_per_resident: self.commuters_per_resident, public_share: self.home_public_share },
            job_public_factor: self.job_public_factor, stations,
        })
    }
}
