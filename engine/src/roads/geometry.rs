use super::{Road, Segment, feature, metres};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

type Point = [f64; 2];

// The drawing rounds simplified vertices to five decimals (under 0.8 m here).
// Match the drawing to this way's ordered source nodes, never to another way's interior.
const DRAWING_PRECISION_M: f64 = 0.8;
const RETAINED_WAY_M: f64 = 8.0 + DRAWING_PRECISION_M;

struct Span<'a> {
    start: Point,
    end: Point,
    source: &'a [Segment],
}

fn spans<'a>(road: &'a Road, point: impl Fn(usize) -> Point) -> Result<Vec<Span<'a>>, String> {
    let mut rest = road.segments.as_slice();
    let mut out = Vec::new();
    let parts = feature::lines(&road.feature["geometry"])?;
    for (part_index, part) in parts.iter().enumerate() {
        for (pair_index, pair) in part.windows(2).enumerate() {
            if rest.is_empty() || pair[0] == pair[1] {
                continue;
            }
            // A clipped way can have separate parts. Never search past the current chain.
            let count = rest
                .windows(2)
                .position(|s| s[0].to != s[1].from)
                .map_or(rest.len(), |i| i + 1);
            let end = if pair_index + 2 == part.len()
                && (count < rest.len() || part_index + 1 == parts.len())
            {
                count
            } else {
                rest[..count]
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        metres(point(a.to), pair[1]).total_cmp(&metres(point(b.to), pair[1]))
                    })
                    .unwrap()
                    .0
                    + 1
            };
            let (source, tail) = rest.split_at(end);
            out.push(Span {
                start: pair[0],
                end: pair[1],
                source,
            });
            rest = tail;
        }
    }
    Ok(out)
}

// Fraction and projected position on the infinite line in the same local metres as routing.
fn project(p: Point, a: Point, b: Point) -> (f64, Point) {
    let x = (b[0] - a[0]) * (111_320. * (-33.45_f64).to_radians().cos());
    let y = (b[1] - a[1]) * 110_540.;
    let px = (p[0] - a[0]) * (111_320. * (-33.45_f64).to_radians().cos());
    let py = (p[1] - a[1]) * 110_540.;
    let t = (px * x + py * y) / (x * x + y * y);
    (t, [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])])
}

// A vertex inserted on the drawn chord splits the cost of its original source
// edge. The chord can be several metres away from the unsimplified source edge.
fn apportion_splits(
    chain: &[usize],
    old_len: usize,
    spans: &[Span<'_>],
    position: impl Fn(usize) -> Point,
    lengths: &mut [f64],
) {
    let mut start = 0;
    for end in 1..chain.len() {
        if chain[end] >= old_len {
            continue;
        }
        let from = chain[start];
        let to = chain[end];
        let run = start..end;
        start = end;
        if run.len() < 2 || from >= old_len || from == to {
            continue;
        }
        for span in spans {
            let Some(edge) = span
                .source
                .iter()
                .find(|s| (s.from == from && s.to == to) || (s.from == to && s.to == from))
            else {
                continue;
            };
            let along = |n| {
                if n == span.source[0].from {
                    0.
                } else if n == span.source.last().unwrap().to {
                    1.
                } else {
                    project(position(n), span.start, span.end).0
                }
            };
            let (a, b) = (along(from), along(to));
            if a == b {
                continue;
            }
            let mut boundaries = vec![0.];
            for &n in &chain[run.start + 1..end] {
                let p = position(n);
                let (t, q) = project(p, span.start, span.end);
                let fraction = (t - a) / (b - a);
                if metres(p, q) > 0.000001 || !(0.0..1.0).contains(&fraction) {
                    break;
                }
                boundaries.push(edge.metres * fraction);
            }
            if boundaries.len() == run.len() && boundaries.windows(2).all(|w| w[1] > w[0]) {
                boundaries.push(edge.metres);
                for (length, cost) in lengths[run.clone()].iter_mut().zip(boundaries.windows(2)) {
                    *length = cost[1] - cost[0];
                }
                break;
            }
        }
    }
}

pub(super) fn segments(
    previous: Option<&Road>,
    after: &Value,
    coords: &[Point],
    extra: &[Point],
    active: &[bool],
    anchors: &[usize],
) -> Result<(Vec<Point>, Vec<Segment>, usize), String> {
    let old_len = coords.len() + extra.len();
    let point = |n: usize| {
        if n < coords.len() {
            coords[n]
        } else {
            extra[n - coords.len()]
        }
    };
    let spans = match previous {
        Some(r) => spans(r, point)?,
        None => Vec::new(),
    };
    let anchors: HashSet<_> = anchors.iter().copied().collect();
    let prior_lengths: HashMap<_, _> = previous
        .into_iter()
        .flat_map(|r| &r.segments)
        .map(|s| ((s.from.min(s.to), s.from.max(s.to)), s.metres))
        .collect();
    let mut added = Vec::new();
    let mut segments = Vec::new();
    let mut endpoints = HashMap::new();
    let mut snapped = 0;

    for part in feature::lines(&after["geometry"])? {
        let mut ids = vec![None; part.len()];
        let mut inserts = vec![Vec::<(f64, usize)>::new(); part.len() - 1];
        for span in &spans {
            let first = span.source[0].from;
            let last = span.source.last().unwrap().to;
            for (i, &p) in part.iter().enumerate() {
                if p == span.start {
                    ids[i].get_or_insert(first);
                }
                if p == span.end {
                    ids[i].get_or_insert(last);
                }
            }
            for (i, pair) in part.windows(2).enumerate() {
                let (a, b) = (pair[0], pair[1]);
                // Subdividing a retained span keeps its source junctions and fixed anchors,
                // including those removed by the 8 m display simplification.
                if a == b {
                    continue;
                }
                let (ta, qa) = project(a, span.start, span.end);
                let (tb, qb) = project(b, span.start, span.end);
                let offset = metres(a, qa).max(metres(b, qb));
                // A transverse crossing of a short old span is not a continuation.
                if offset > RETAINED_WAY_M || metres(qa, qb) < 0.9 * metres(a, b) {
                    continue;
                }
                let subdivision = offset <= DRAWING_PRECISION_M
                    && ta.min(tb) >= -1e-8
                    && ta.max(tb) <= 1.0 + 1e-8;
                for node in std::iter::once(first).chain(span.source.iter().map(|s| s.to)) {
                    if !subdivision && !active[node] && !anchors.contains(&node) {
                        continue;
                    }
                    let drawn = if node == first {
                        span.start
                    } else if node == last {
                        span.end
                    } else {
                        let (along, p) = project(point(node), span.start, span.end);
                        if !(0.0..=1.0).contains(&along) {
                            continue;
                        }
                        p
                    };
                    let (t, p) = project(drawn, a, b);
                    if metres(drawn, p) > RETAINED_WAY_M {
                        continue;
                    }
                    if metres(drawn, a) < 0.000001
                        || (!subdivision
                            && (node == first || node == last)
                            && t <= 0.
                            && metres(drawn, a) <= RETAINED_WAY_M)
                    {
                        ids[i].get_or_insert(node);
                    } else if metres(drawn, b) < 0.000001
                        || (!subdivision
                            && (node == first || node == last)
                            && t >= 1.
                            && metres(drawn, b) <= RETAINED_WAY_M)
                    {
                        ids[i + 1].get_or_insert(node);
                    } else if (0.0..1.0).contains(&t) {
                        inserts[i].push((t, node));
                    }
                }
            }
        }
        let mut chain = Vec::new();
        for (i, (&p, inherited)) in part.iter().zip(ids).enumerate() {
            let endpoint = i == 0 || i + 1 == part.len();
            let key = (p[0].to_bits(), p[1].to_bits());
            let node = inherited
                .or_else(|| {
                    if !endpoint {
                        return None;
                    }
                    let best = coords
                        .iter()
                        .chain(extra)
                        .enumerate()
                        .filter(|(n, _)| active[*n])
                        .map(|(n, &q)| (n, metres(p, q)))
                        .filter(|&(_, d)| d <= 30.)
                        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
                    if let Some((n, _)) = best {
                        snapped += 1;
                        Some(n)
                    } else {
                        endpoints.get(&key).copied()
                    }
                })
                .unwrap_or_else(|| {
                    let n = old_len + added.len();
                    added.push(p);
                    n
                });
            if endpoint {
                endpoints.insert(key, node);
            }
            chain.push(node);
            if let Some(interior) = inserts.get_mut(i) {
                interior.sort_by(|a, b| a.0.total_cmp(&b.0));
                chain.extend(interior.iter().map(|(_, n)| *n));
            }
        }
        let position = |n| {
            if n < old_len {
                point(n)
            } else {
                added[n - old_len]
            }
        };
        chain.dedup();
        let mut lengths: Vec<_> = chain
            .windows(2)
            .map(|pair| {
                let (from, to) = (pair[0], pair[1]);
                prior_lengths
                    .get(&(from.min(to), from.max(to)))
                    .copied()
                    .unwrap_or_else(|| metres(position(from), position(to)))
            })
            .collect();
        apportion_splits(&chain, old_len, &spans, position, &mut lengths);
        for (pair, length) in chain.windows(2).zip(lengths) {
            if length > 0. {
                segments.push(Segment {
                    from: pair[0],
                    to: pair[1],
                    metres: length,
                });
            }
        }
    }
    Ok((added, segments, snapped))
}
