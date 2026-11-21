//! The workers of a `Population` choose again after a change of the city (job weights, new job points, minutes).
//! The rules of choice.rs give quotas: workers by comuna pair, by job point and by mode of each pair. A worker
//! keeps its job point and its mode while their quotas have room. Only the surplus moves. No random numbers.
use crate::choice::Choice;
use crate::population::Population;
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

/// `job_weights`: one weight per point of `pop.jobs`, same order (push new points to `pop.jobs` before the call).
/// `minutes`: n×n comuna matrix, as `Choice::minutes`. Students do not change. Returns the workers that changed `dest`.
pub fn rechoose(c: &Choice, pop: &mut Population, job_weights: &[f64], minutes: &[f64]) -> usize {
    let n = c.n;
    let ix: HashMap<&str, usize> = c.cuts.iter().enumerate().map(|(i, s)| (s.as_str(), i)).collect();
    let home: Vec<usize> = pop.homes.iter().map(|p| ix[p.cut.as_str()]).collect();
    let work: Vec<usize> = pop.jobs.iter().map(|p| ix[p.cut.as_str()]).collect();
    let mut jobs = vec![0.0; n];
    for (d, w) in work.iter().zip(job_weights) { jobs[*d] += w; }
    let (prob, share) = (c.destinations(&jobs, minutes), c.modes(minutes));

    // Quotas: comuna pair, mode of each pair, job point.
    let mut workers = vec![0.0; n];
    for p in pop.persons.iter().filter(|p| !p.student) { workers[home[p.home as usize]] += 1.0; }
    let mut pair: Vec<u32> = (0..n).flat_map(|o| split(workers[o], prob[o * n..][..n].iter().copied())).collect();
    let mut mode: Vec<Vec<u32>> = (0..n * n).map(|i| split(pair[i] as f64, share[i].into_iter())).collect();
    let mut point = vec![0; work.len()];
    for d in 0..n {
        let into: f64 = (0..n).map(|o| pair[o * n + d] as f64).sum();
        let ids: Vec<usize> = (0..work.len()).filter(|&p| work[p] == d).collect();
        for (p, q) in ids.iter().zip(split(into, ids.iter().map(|&p| job_weights[p] / jobs[d]))) { point[*p] = q; }
    }

    // Each worker keeps what has room.
    let class = |m: i8| match m { 2 => 0, 1 => 1, _ => 2 };
    let (mut pool, mut no_mode) = (vec![], vec![]);
    for (i, p) in pop.persons.iter().enumerate().filter(|(_, p)| !p.student) {
        let od = home[p.home as usize] * n + work[p.dest as usize];
        if pair[od] > 0 && point[p.dest as usize] > 0 {
            pair[od] -= 1;
            point[p.dest as usize] -= 1;
            if mode[od][class(p.mode)] > 0 { mode[od][class(p.mode)] -= 1 } else { no_mode.push(i) }
        } else {
            pool.push(i);
        }
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
        moved += (dest != p.dest) as usize;
        p.dest = dest;
    }
    for &i in pool.iter().chain(&no_mode) {
        let p = &mut pop.persons[i];
        let m = &mut mode[home[p.home as usize] * n + work[p.dest as usize]];
        let j = if m[class(p.mode)] > 0 { class(p.mode) } else { m.iter().position(|&q| q > 0).unwrap() };
        m[j] -= 1;
        // shortcut: a new "other" worker gets census code 7, upgrade if a consumer needs the split of codes 3 to 7.
        if j != class(p.mode) { p.mode = [2, 1, 7][j]; }
    }
    moved
}
