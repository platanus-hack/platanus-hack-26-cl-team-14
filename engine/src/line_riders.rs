//! Person-level riders of an added transit line (any mode: boat, bus, Metro).
//! Each worker compares a door-to-door trip on the new line with the fitted public, car and other utilities
//! of its comuna pair (choice.rs). The new line is a fourth alternative with the public coefficients and the
//! public residual of the pair, so at equal minutes it is as attractive as the existing public mode.
use crate::choice::Choice;
use crate::population::{Point, Population};
use santiago_scenario_core::editable_transit::AddedLine;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;

// Same walk rules as editable_transit/routing.rs.
const ACCESS_METRES: f64 = 1_000.0;
const WALK_METRES_PER_MINUTE: f64 = 80.0;

fn metres(a: [f64; 2], b: [f64; 2]) -> f64 {
    let latitude = ((a[1] + b[1]) * 0.5).to_radians();
    ((a[0] - b[0]) * latitude.cos() * 111_320.0).hypot((a[1] - b[1]) * 110_574.0)
}

/// Stops of the line within walking distance of each point that allow the call (rule 0), with walk minutes.
fn near(points: &[Point], line: &AddedLine, rules: &[u8]) -> Vec<Vec<(usize, f64)>> {
    points
        .iter()
        .map(|p| {
            line.stops
                .iter()
                .enumerate()
                .filter(|&(k, _)| rules[k] == 0)
                .map(|(k, &s)| (k, metres([p.longitude, p.latitude], s)))
                .filter(|&(_, d)| d <= ACCESS_METRES)
                .map(|(k, d)| (k, d / WALK_METRES_PER_MINUTE))
                .collect()
        })
        .collect()
}

/// Weekday departure shares of public-transport trips to work by half hour (work_departures.csv).
fn departures(data: &Path) -> [f64; 48] {
    let mut shares = [0.0; 48];
    for r in csv::Reader::from_path(data.join("work_departures.csv"))
        .unwrap()
        .into_records()
        .map(|r| r.unwrap())
    {
        if &r[0] == "public" {
            shares[r[1].parse::<usize>().unwrap()] = r[3].parse().unwrap();
        }
    }
    shares
}

fn r1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn direct(
    line: &AddedLine,
    access: &[(usize, f64)],
    egress: &[(usize, f64)],
) -> Option<(f64, usize, usize)> {
    access
        .iter()
        .flat_map(|&(on, walk_in)| {
            egress
                .iter()
                .filter(move |&&(off, _)| {
                    off > on
                        && !line
                            .retained_calls
                            .as_ref()
                            .is_some_and(|old| old.contains(&on) && old.contains(&off))
                })
                .map(move |&(off, walk_out)| {
                    let minutes = walk_in + line.headway_minutes / 2.0 + line.calls[off][0]
                        - line.calls[on][1]
                        + walk_out;
                    (minutes, on, off)
                })
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
}

pub fn project(
    line: &AddedLine,
    pop: &Population,
    c: &Choice,
    minutes: &[f64],
    data: &Path,
) -> Value {
    project_selected(line, pop, c, minutes, data, None)
}

fn project_selected(
    line: &AddedLine,
    pop: &Population,
    c: &Choice,
    minutes: &[f64],
    data: &Path,
    selection: Option<(&[Option<usize>], usize)>,
) -> Value {
    let n = c.n;
    let ix: HashMap<&str, usize> = c
        .cuts
        .iter()
        .enumerate()
        .map(|(i, s)| (s.as_str(), i))
        .collect();
    let (access, egress) = (
        near(&pop.homes, line, &line.pickup_type),
        near(&pop.jobs, line, &line.drop_off_type),
    );
    let k = line.stops.len();
    let (mut eligible, mut faster, mut demand) = (0u32, 0u32, 0.0);
    let (mut boardings, mut alightings, mut from) = (vec![0.0; k], vec![0.0; k], [0.0; 3]);
    let mut on_board = vec![0.0; k - 1]; // daily demand on each segment between consecutive stops
    for (index, p) in pop.persons.iter().enumerate().filter(|(_, p)| !p.student) {
        if selection.is_some_and(|(directions, direction)| directions[index] != Some(direction)) {
            continue;
        }
        let (a, e) = (&access[p.home as usize], &egress[p.dest as usize]);
        let Some((t, on, off)) = direct(line, a, e) else {
            continue;
        };
        let i = ix[pop.homes[p.home as usize].cut.as_str()] * n
            + ix[pop.jobs[p.dest as usize].cut.as_str()];
        eligible += 1;
        if t < minutes[i] {
            faster += 1;
        }
        let (co, rk, l) = (&c.mode_coef, &c.mode_k[i], c.km[i].ln());
        let w = [
            (co[0] + co[1] * minutes[i] + co[2] * l).exp() * rk[0],
            rk[1],
            (co[3] + co[4] * l).exp() * rk[2],
        ];
        let new = (co[0] + co[1] * t + co[2] * l).exp() * rk[0];
        let (old_total, new_total) = (w[0] + w[1] + w[2], w[0] + w[1] + w[2] + new);
        let share = new / new_total;
        demand += share;
        boardings[on] += share;
        alightings[off] += share;
        for segment in &mut on_board[on..off] {
            *segment += share;
        }
        for j in 0..3 {
            from[j] += w[j] / old_total - w[j] / new_total;
        }
    }
    let shares = departures(data);
    // Places per half hour on each segment. All riders share one departure profile, so the busiest segment binds.
    let capacity = line
        .vehicle_capacity
        .map(|seats| seats * 30.0 / line.headway_minutes);
    let busiest = on_board.iter().cloned().fold(0.0, f64::max);
    let demand_hh: Vec<f64> = shares.iter().map(|s| demand * s).collect();
    let served_hh: Vec<f64> = shares
        .iter()
        .map(|s| capacity.map_or(1.0, |cap| (cap / (busiest * s)).min(1.0)))
        .collect();
    let riders_hh: Vec<f64> = demand_hh
        .iter()
        .zip(&served_hh)
        .map(|(d, f)| d * f)
        .collect();
    let riders: f64 = riders_hh.iter().sum();
    let peak = shares.iter().cloned().fold(0.0, f64::max);
    let served = if demand > 0.0 { riders / demand } else { 0.0 };
    json!({
        "id": line.id, "mode": line.mode,
        "workers_near_both_ends": eligible,
        "faster_than_public_share": if eligible > 0 { (f64::from(faster) / f64::from(eligible) * 1000.0).round() / 1000.0 } else { 0.0 },
        "demand_to_work": r1(demand), "riders_to_work": r1(riders),
        "taken_from": { "public": r1(from[0] * served), "car": r1(from[1] * served), "other": r1(from[2] * served) },
        "capacity_per_half_hour": capacity.map(r1),
        "peak_segment_demand_to_capacity": capacity.map(|cap| (busiest * peak / cap * 100.0).round() / 100.0),
        "peak_segment_load_factor": capacity.map(|cap| ((busiest * peak / cap).min(1.0) * 100.0).round() / 100.0),
        "segment_on_board_daily": on_board.iter().map(|&x| r1(x * served)).collect::<Vec<_>>(),
        "stops": line.stop_ids.iter().zip(&line.stops).enumerate().map(|(s, (id, at))| json!({
            "id": id, "longitude": at[0], "latitude": at[1],
            "boardings": r1(boardings[s] * served), "alightings": r1(alightings[s] * served)
        })).collect::<Vec<_>>(),
        "demand_half_hour": demand_hh.iter().map(|&x| r1(x)).collect::<Vec<_>>(),
        "riders_half_hour": riders_hh.iter().map(|&x| r1(x)).collect::<Vec<_>>(),
    })
}

/// One direct service choice per worker across both directions of a confirmed project.
pub fn project_metro(
    project_id: &str,
    lines: &[AddedLine],
    pop: &Population,
    c: &Choice,
    data: &Path,
) -> Value {
    let walks: Vec<_> = lines
        .iter()
        .map(|line| {
            (
                near(&pop.homes, line, &line.pickup_type),
                near(&pop.jobs, line, &line.drop_off_type),
            )
        })
        .collect();
    let directions: Vec<_> = pop
        .persons
        .iter()
        .map(|p| {
            if p.student {
                return None;
            }
            lines
                .iter()
                .zip(&walks)
                .enumerate()
                .filter_map(|(j, (line, (access, egress)))| {
                    let (minutes, _, _) =
                        direct(line, &access[p.home as usize], &egress[p.dest as usize])?;
                    Some((minutes, j))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, j)| j)
        })
        .collect();
    let patterns: Vec<_> = lines
        .iter()
        .enumerate()
        .map(|(j, line)| project_selected(line, pop, c, &c.minutes, data, Some((&directions, j))))
        .collect();
    let sum = |field: &str| {
        r1(patterns
            .iter()
            .map(|pattern| pattern[field].as_f64().unwrap())
            .sum())
    };
    let from = |mode: &str| {
        r1(patterns
            .iter()
            .map(|pattern| pattern["taken_from"][mode].as_f64().unwrap())
            .sum())
    };
    let activities: Vec<_> = ["study", "shopping", "health", "event"]
        .into_iter()
        .map(|purpose| {
            json!({"purpose":purpose, "riders":null, "taken_from":null,
            "reason":"The line adoption model currently covers work journeys only."})
        })
        .collect();
    json!({
        "project_id":project_id, "purpose":"work", "unit":"one_way_weekday_trips",
        "workers_near_both_ends":directions.iter().flatten().count(),
        "demand_to_work":sum("demand_to_work"), "riders_to_work":sum("riders_to_work"),
        "taken_from":{"public":from("public"),"car":from("car"),"other":from("other")},
        "patterns":patterns, "activities":activities,
        "assumptions":[
            "One direct service alternative per worker, using the faster eligible direction and the pre-project comuna commute baseline.",
            "Extension trips use at least one proposed segment. Transfers affect the ordinary routing graph and are outside this direct-use estimate.",
            "Weekday work departure shares distribute demand over half-hour bins. Headway is assumed for this demand estimate."
        ]
    })
}
