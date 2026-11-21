//! The workers of a `Population` choose again after a change of the city (job weights, new job points, minutes).
//! The rules of choice.rs give quotas: workers by comuna pair, by job point and by mode of each pair. A worker
//! keeps its job point and its mode while their quotas have room. Only the surplus moves. No random numbers.
use crate::choice::Choice;
use crate::population::{NEW_OTHER_MODE, Person, Population, UNKNOWN};
use std::collections::HashMap;

/// Integers near `total × share` that sum to `total` (rounding of the cumulative sum).
fn split(total: f64, shares: impl Iterator<Item = f64>) -> Vec<u32> {
    let (mut cum, mut done) = (0.0, 0.0);
    shares.map(|s| {
        cum += total * s;
        let q = cum.round() - done;
        done += q;
        q as u32
    }).collect()
}

/// Free places in golden-ratio order (the sequence of population.rs). Thus each run of takers gets all kinds
/// of place in proportion, and the order of the persons does not tie one origin to one job point.
fn spread(places: Vec<u32>) -> Vec<u32> {
    let mut v: Vec<(f64, u32)> = places.into_iter().enumerate().map(|(i, p)| (((i as f64 + 0.5) * 0.618_033_988_749_894_9).fract(), p)).collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v.into_iter().map(|x| x.1).collect()
}

fn class(m: i8) -> usize { match m { 2 => 0, 1 => 1, _ => 2 } }

/// The baseline placement of population.rs minus the quotas of the baseline inputs, per comuna pair, per mode
/// of each pair and per job point. `rechoose` adds it to the new quotas. Thus the quotas of the baseline inputs
/// are the baseline placement and no worker moves, and other inputs move only the true surplus.
/// `stay`: per home comuna, age band and household size, the baseline share of workers in each mode class.
/// When a mode quota shrinks, a worker keeps its mode with a weight equal to this share for its group.
pub struct Base { pair: Vec<i32>, mode: Vec<[i32; 3]>, point: Vec<i32>, stay: Vec<[f32; 3]> }

const HOUSEHOLD_GROUPS: usize = 6;
const GROUPS_PER_COMUNA: usize = 5 * HOUSEHOLD_GROUPS;

/// Keep workers with unknown household size in a separate group within their home comuna and age band.
fn group(home: usize, p: &Person) -> usize {
    let household = match p.household_size {
        UNKNOWN => HOUSEHOLD_GROUPS - 1,
        size => usize::from(size) - 1,
    };
    home * GROUPS_PER_COMUNA + usize::from(p.age_band) * HOUSEHOLD_GROUPS + household
}

impl Base {
    /// `pop`: the baseline population, before any change.
    pub fn new(c: &Choice, pop: &Population) -> Base {
        let n = c.n;
        let weights: Vec<f64> = pop.jobs.iter().map(|p| p.weight).collect();
        let zero = Base { pair: vec![0; n * n], mode: vec![[0; 3]; n * n], point: vec![0; weights.len()], stay: vec![] };
        let (home, work, pair, mode, point) = quotas(c, pop, &weights, &c.minutes, &c.minutes, &zero);
        let mut b = Base { pair: pair.iter().map(|&q| -(q as i32)).collect(), mode: mode.iter().map(|m| [0, 1, 2].map(|j| -(m[j] as i32))).collect(),
            point: point.iter().map(|&q| -(q as i32)).collect(), stay: vec![[0.0; 3]; n * GROUPS_PER_COMUNA] };
        for p in pop.persons.iter().filter(|p| !p.student) {
            let od = home[p.home as usize] * n + work[p.dest as usize];
            b.pair[od] += 1;
            b.mode[od][class(p.mode)] += 1;
            b.point[p.dest as usize] += 1;
            b.stay[group(home[p.home as usize], p)][class(p.mode)] += 1.0;
        }
        for g in &mut b.stay {
            let total: f32 = g.iter().sum();
            if total > 0.0 { g.iter_mut().for_each(|x| *x /= total); }
        }
        b
    }
}

/// Comuna of each home point and of each job point, then the quotas: comuna pair, mode of each pair, job point.
/// A quota is the rounded share of the rules plus the part of `base`, and is 0 or more.
fn quotas(c: &Choice, pop: &Population, job_weights: &[f64], job_minutes: &[f64], minutes: &[f64], base: &Base) -> (Vec<usize>, Vec<usize>, Vec<u32>, Vec<Vec<u32>>, Vec<u32>) {
    let n = c.n;
    let ix: HashMap<&str, usize> = c.cuts.iter().enumerate().map(|(i, s)| (s.as_str(), i)).collect();
    let home: Vec<usize> = pop.homes.iter().map(|p| ix[p.cut.as_str()]).collect();
    let work: Vec<usize> = pop.jobs.iter().map(|p| ix[p.cut.as_str()]).collect();
    let mut jobs = vec![0.0; n];
    for (d, w) in work.iter().zip(job_weights) { jobs[*d] += w; }
    let (prob, share) = (c.destinations(&jobs, job_minutes), c.modes(minutes));
    let fix = |q: u32, r: i32| (q as i32 + r).max(0) as u32;

    let mut workers = vec![0.0; n];
    for p in pop.persons.iter().filter(|p| !p.student) { workers[home[p.home as usize]] += 1.0; }
    let pair: Vec<u32> = (0..n).flat_map(|o| split(workers[o], prob[o * n..][..n].iter().copied())).zip(&base.pair).map(|(q, &r)| fix(q, r)).collect();
    let mode = (0..n * n).map(|i| split(pair[i] as f64, share[i].into_iter()).into_iter().zip(base.mode[i]).map(|(q, r)| fix(q, r)).collect()).collect();
    let mut point = vec![0; work.len()];
    for d in 0..n {
        let into: f64 = (0..n).map(|o| pair[o * n + d] as f64).sum();
        let ids: Vec<usize> = (0..work.len()).filter(|&p| work[p] == d).collect();
        // A new point has no part in `base`.
        for (p, q) in ids.iter().zip(split(into, ids.iter().map(|&p| job_weights[p] / jobs[d]))) { point[*p] = fix(q, base.point.get(*p).copied().unwrap_or(0)); }
    }
    (home, work, pair, mode, point)
}

/// `job_weights`: one weight per point of `pop.jobs`, same order (push new points to `pop.jobs` before the call).
/// `job_minutes`: n×n comuna matrix for job choice, `minutes` for mode choice, as `Choice::minutes`.
/// A temporary change (one day of service) moves `minutes` only, so workers keep their job comuna. `base`: `Base::new` of the population before the change.
/// Students do not change. Returns the workers that had a job point and changed it.
pub fn rechoose(c: &Choice, pop: &mut Population, job_weights: &[f64], job_minutes: &[f64], minutes: &[f64], base: &Base) -> usize {
    let n = c.n;
    let (home, work, mut pair, mut mode, mut point) = quotas(c, pop, job_weights, job_minutes, minutes, base);

    // Each worker keeps what has room.
    let (mut pool, mut kept, mut no_mode) = (vec![], vec![], vec![]);
    for (i, p) in pop.persons.iter().enumerate().filter(|(_, p)| !p.student) {
        // dest u32::MAX = a new worker with no job yet (scenario.rs): it goes to the pool.
        let od = home[p.home as usize] * n + work.get(p.dest as usize).copied().unwrap_or(0);
        if p.dest != u32::MAX && pair[od] > 0 && point[p.dest as usize] > 0 {
            pair[od] -= 1;
            point[p.dest as usize] -= 1;
            kept.push(i);
        } else {
            pool.push(i);
        }
    }
    // Who keeps a shrinking mode: a weighted draw without replacement (Efraimidis and Spirakis, 2006), the weight
    // being the share of that mode in the worker's group. Key u^(1/w), largest first. u is a SplitMix64 hash of
    // the person index: deterministic, and not tied to the golden-ratio order of the home and job samplers.
    let key = |i: usize| {
        let p = &pop.persons[i];
        let w = f64::from(base.stay[group(home[p.home as usize], p)][class(p.mode)]);
        let mut z = (i as u64).wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        let u = ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64;
        u.powf(1.0 / w)
    };
    let mut kept: Vec<(f64, usize)> = kept.into_iter().map(|i| (key(i), i)).collect();
    kept.sort_by(|a, b| b.0.total_cmp(&a.0));
    let kept = kept.into_iter().map(|(_, i)| i);
    for i in kept {
        let p = &pop.persons[i];
        let m = &mut mode[home[p.home as usize] * n + work[p.dest as usize]][class(p.mode)];
        if *m > 0 { *m -= 1 } else { no_mode.push(i) }
    }

    // The surplus takes the free places: first a comuna from the row of its home comuna, then a point in it.
    let (mut comunas, mut points) = (vec![vec![]; n], vec![vec![]; n]);
    for (i, q) in pair.iter().enumerate() { comunas[i / n].extend((0..*q).map(|_| (i % n) as u32)); }
    for (p, q) in point.iter().enumerate() { points[work[p]].extend((0..*q).map(|_| p as u32)); }
    let (mut comunas, mut points): (Vec<_>, Vec<_>) = (comunas.into_iter().map(spread).collect(), points.into_iter().map(spread).collect());
    let mut moved = 0;
    for &i in &pool {
        let p = &mut pop.persons[i];
        let d = comunas[home[p.home as usize]].pop().unwrap() as usize;
        let dest = points[d].pop().unwrap();
        moved += (p.dest != u32::MAX && dest != p.dest) as usize;
        p.dest = dest;
    }
    for &i in pool.iter().chain(&no_mode) {
        let p = &mut pop.persons[i];
        let m = &mut mode[home[p.home as usize] * n + work[p.dest as usize]];
        let j = if m[class(p.mode)] > 0 { class(p.mode) } else { m.iter().position(|&q| q > 0).unwrap() };
        m[j] -= 1;
        // FineModes::apply (population/fine_modes.rs) gives a new "other" worker its census code.
        if j != class(p.mode) { p.mode = [2, 1, NEW_OTHER_MODE][j]; }
    }
    moved
}
