//! A representative origin/destination pairing from the cached stop margins.
//! The result has weekday totals per pier and a departure-time profile per line.
use super::vessel::Line;
use serde_json::Value;

pub(super) const TRIPS_PER_FIGURE: f64 = 4.;
pub(super) const DAY: f32 = 86_400.;

pub(super) struct Trip {
    pub on: usize,
    pub off: usize,
    pub board_at: f32,
    pub leave_at: f32,
    pub seat: usize,
    pub wait: f32,
}

pub(super) struct Sailing {
    pub start: f32,
    pub trips: Vec<Trip>,
    pub deck_rows: usize,
}

fn cumulative(stops: &[&Value], field: &str) -> Vec<f64> {
    let mut sum = 0.;
    let mut shares: Vec<_> = stops
        .iter()
        .map(|s| {
            sum += s[field].as_f64().unwrap();
            sum
        })
        .collect();
    if sum > 0. {
        for v in &mut shares {
            *v /= sum;
        }
    }
    shares
}

pub(super) fn schedule(line: &Line, row: Option<&Value>) -> Vec<Sailing> {
    let stops: Vec<_> = row
        .into_iter()
        .flat_map(|r| r["stops"].as_array())
        .flatten()
        .collect();
    let ordered: Vec<_> = line
        .stops
        .iter()
        .filter_map(|stop| {
            stops
                .iter()
                .copied()
                .find(|s| s["id"].as_str() == Some(&stop.id))
        })
        .collect();
    let boarding = cumulative(&ordered, "boardings");
    let alighting = cumulative(&ordered, "alightings");
    let bins = row.and_then(|r| r["riders_half_hour"].as_array());
    let mut runs = Vec::new();
    let mut expected: f64 = 0.;
    let mut drawn = 0;
    let mut start = 0.;
    while start < DAY {
        let mut trips = Vec::new();
        if ordered.len() == line.stops.len()
            && let Some(bins) = bins
        {
            // Allocate each half-hour across its departures. Carry fractional symbols
            // between departures so that low-volume sailings do not vanish for the day.
            let end = (start + line.headway).min(DAY);
            let mut at = start;
            while at < end {
                let bin = (at / 1800.) as usize;
                let next = end.min((bin + 1) as f32 * 1800.);
                expected +=
                    bins[bin].as_f64().unwrap() * ((next - at) / 1800.) as f64 / TRIPS_PER_FIGURE;
                at = next;
            }
            let target = expected.round() as usize;
            for serial in drawn..target {
                // The same quantile in both cumulative margins pairs an upstream
                // boarding with a downstream alighting. No observed OD is implied.
                let q = ((serial as f64 + 0.5) * 0.61803398875).fract();
                let on = boarding.partition_point(|&v| v <= q);
                let off = alighting.partition_point(|&v| v <= q);
                if on >= off || off >= line.stops.len() {
                    continue;
                }
                let fraction = ((serial as f64 + 0.5) * 0.41421356237).fract() as f32;
                let from = &line.stops[on];
                let to = &line.stops[off];
                let board_at = if on == 0 {
                    -12. + 7. * fraction
                } else {
                    from.arrival + (from.departure - from.arrival) * (0.48 + 0.38 * fraction)
                };
                let leave_at = to.arrival + (to.departure - to.arrival) * (0.04 + 0.25 * fraction);
                trips.push(Trip {
                    on,
                    off,
                    board_at,
                    leave_at,
                    seat: 0,
                    wait: 7. + 35. * fraction,
                });
            }
            drawn = target;
        }
        // Keep a place on deck until its passenger has finished disembarking.
        trips.sort_by(|a, b| a.board_at.total_cmp(&b.board_at));
        let mut occupied_until = Vec::<f32>::new();
        for t in &mut trips {
            t.seat = occupied_until
                .iter()
                .position(|&end| end <= t.board_at)
                .unwrap_or(occupied_until.len());
            let free_at = t.leave_at + super::CROSSING;
            if t.seat == occupied_until.len() {
                occupied_until.push(free_at);
            } else {
                occupied_until[t.seat] = free_at;
            }
        }
        runs.push(Sailing {
            start,
            trips,
            deck_rows: occupied_until.len().div_ceil(4),
        });
        start += line.headway;
    }
    runs
}
