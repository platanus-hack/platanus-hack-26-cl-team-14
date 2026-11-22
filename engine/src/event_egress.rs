//! Estimated match departures on the baseline Metro network.
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use super::attendance::AttendanceRequest;

// Fit to the 26 April 2025 Monumental match and its reference Saturday.
const CAPTURE: f64 = 0.689;
const PROFILE: [f64; 4] = [0.068, 0.516, 0.305, 0.111];

fn records(path: &Path) -> impl Iterator<Item = csv::StringRecord> {
    csv::Reader::from_path(path)
        .unwrap()
        .into_records()
        .map(Result::unwrap)
}

fn km(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0) * 92.9).hypot((a.1 - b.1) * 110.6)
}

pub(super) fn project(
    dir: &Path,
    fans: &Value,
    event: &AttendanceRequest,
    stations: &[(String, String, f64, f64)],
    homes: impl Iterator<Item = (String, (f64, f64))>,
) -> Option<Value> {
    let shares_path = dir.join("eod_saturday_metro_share.csv");
    if !shares_path.is_file() {
        return None;
    }
    let sectors: HashMap<_, _> = records(&dir.join("eod_sectors/zones.csv"))
        .filter(|r| !r[2].is_empty())
        .map(|r| (r[1].to_owned(), r[2].parse::<u8>().unwrap()))
        .collect();
    let venue_sector = *sectors.get(&event.venue_cut)?;
    let venue = (event.longitude, event.latitude);
    let nearby: Vec<_> = stations
        .iter()
        .filter(|(_, _, x, y)| km(venue, (*x, *y)) <= 1.0)
        .collect();
    if nearby.is_empty() {
        return None;
    }
    let shares: HashMap<_, f64> = records(&shares_path)
        .map(|r| {
            (
                (r[0].parse::<u8>().unwrap(), r[1].parse::<u8>().unwrap()),
                r[3].parse().unwrap(),
            )
        })
        .collect();
    let homes: HashMap<_, _> = homes.collect();
    let mut users = BTreeMap::new();
    for home in fans["by_home"].as_array().unwrap() {
        let cut = home["cut"].as_str().unwrap();
        let Some(share) = sectors
            .get(cut)
            .and_then(|sector| shares.get(&(venue_sector, *sector)))
        else {
            continue;
        };
        let cost = |station: &&(String, String, f64, f64)| {
            let pos = (station.2, station.3);
            km(venue, pos) + km(pos, homes[cut])
        };
        let station = nearby
            .iter()
            .copied()
            .min_by(|a, b| cost(a).total_cmp(&cost(b)))
            .unwrap();
        *users.entry((&station.0, &station.1)).or_insert(0.0) +=
            home["attendees"].as_f64().unwrap() * share * CAPTURE;
    }
    let by_station: Vec<_> = users
        .iter()
        .map(|((id, name), users)| {
            let entries: BTreeMap<_, _> = PROFILE
                .iter()
                .enumerate()
                .map(|(offset, share)| {
                    let bin =
                        (i32::from(event.assumed_end_half_hour) + offset as i32 - 1).rem_euclid(48);
                    (bin, users * share)
                })
                .collect();
            json!({ "id": id, "name": name, "metro_users": users, "entries_by_half_hour": entries })
        })
        .collect();
    Some(json!({
        "metro_users": users.values().sum::<f64>(),
        "stations": by_station,
        "assumption": "Saturday EOD mode shares and baseline stations within 1 km. Capture and departure profile fit the April 2025 Monumental match. The estimate uses straight-line distance and ignores service edits and capacity."
    }))
}
