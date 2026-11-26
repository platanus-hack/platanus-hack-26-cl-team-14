//! Attach venue-bound network paths to the fixed fan groups.
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

const WALK_METRES_PER_MINUTE: f64 = 80.0;
const MODES: [&str; 3] = ["public", "car", "other"];

#[derive(Deserialize)]
pub(super) struct Request {
    venue_id: String,
    gate_arrival_minutes: [u16; 2],
}

impl Request {
    pub(super) fn parse(input: &Value) -> Option<Self> {
        let request: Self = serde_json::from_value(input.clone()).ok()?;
        let [start, end] = request.gate_arrival_minutes;
        (start < end && end <= 1440).then_some(request)
    }
}

fn compressed_routes(data: &Path, file: &str) -> Option<Value> {
    let file = std::io::BufReader::new(std::fs::File::open(data.join(file)).ok()?);
    serde_json::from_reader(flate2::read::GzDecoder::new(file)).ok()
}

fn car_legs(venue: &Value, home_cut: &str) -> Option<Vec<Value>> {
    let route = venue["communes"].get(home_cut)?;
    let drive_minutes = route["seconds"].as_f64()? / 60.0;
    let walk_minutes = venue["walk_metres"].as_f64()? / WALK_METRES_PER_MINUTE;
    Some(vec![
        json!({
            "mode": "car",
            "minutes": drive_minutes,
            "geometry": {"type": "LineString", "coordinates": route["path"]},
            "crossings": route["crossings"],
        }),
        json!({
            "mode": "walk",
            "minutes": walk_minutes,
            "geometry": {"type": "LineString", "coordinates": venue["walk_path"]},
        }),
    ])
}

fn read_routes(data: &Path, file: &str) -> Value {
    std::fs::File::open(data.join(file))
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
        .unwrap_or(Value::Null)
}

fn scheduled_legs(
    plan: &Value,
    access: &Value,
    gate_window: [f64; 2],
) -> Option<(Vec<Value>, f64)> {
    let access_seconds = access["seconds"].as_f64()?;
    let access_path = access["path"].as_array()?;
    let gate = &plan["walk_path"];
    let gate_seconds = gate["seconds"].as_f64()?;
    let gate_path = gate["geometry"]["coordinates"].as_array()?;
    let lines = plan["legs"].as_array()?;
    let first = lines.first()?;
    let ingress = if first["mode"] == "metro" {
        let platform = first["board_platform_id"].as_str()?;
        let route = access["station_ingress"].get(platform)?;
        Some((route, route["seconds"].as_f64()?))
    } else {
        None
    };
    let options = plan["departure_options"].as_array()?;
    let midpoint = (gate_window[0] + gate_window[1]) * 0.5;
    let (option, board, arrives) = options
        .iter()
        .filter_map(|option| {
            let board = option["departure_seconds"].as_f64()?;
            let arrives = option["gate_arrival_seconds"].as_f64()?;
            let gate_minute = arrives / 60.0;
            (gate_minute >= gate_window[0] && gate_minute < gate_window[1])
                .then_some((option, board, arrives))
        })
        .min_by(|a, b| {
            (a.2 / 60.0 - midpoint)
                .abs()
                .total_cmp(&(b.2 / 60.0 - midpoint).abs())
        })?;
    let rides = option["legs"].as_array()?;
    if lines.len() != rides.len() {
        return None;
    }
    let walk = |path: &[Value], start: f64, end: f64| {
        json!({
            "mode": "walk", "minutes": (end - start) / 60.0,
            "departure_seconds": start, "arrival_seconds": end,
            "geometry": {"type": "LineString", "coordinates": path},
        })
    };
    let door = board - ingress.map_or(0.0, |(_, seconds)| seconds);
    let depart = door - access_seconds;
    let mut previous_end = board;
    let mut legs = vec![walk(access_path, depart, door)];
    if let Some((route, seconds)) = ingress {
        legs.push(json!({
            "mode": "station_ingress", "minutes": seconds / 60.0,
            "departure_seconds": door, "arrival_seconds": board,
            "route_ref": route,
        }));
    }
    for (line, ride) in lines.iter().zip(rides) {
        let begin = ride["departure_seconds"].as_f64()?;
        let end = ride["arrival_seconds"].as_f64()?;
        if begin < previous_end || end < begin {
            return None;
        }
        if begin > previous_end {
            legs.push(json!({
                "mode": "wait", "minutes": (begin - previous_end) / 60.0,
                "departure_seconds": previous_end, "arrival_seconds": begin,
            }));
        }
        let mut leg = line.clone();
        leg["minutes"] = json!((end - begin) / 60.0);
        leg["departure_seconds"] = json!(begin);
        leg["arrival_seconds"] = json!(end);
        leg["route_ref"] = ride.clone();
        leg["departure_options"] = json!([ride]);
        legs.push(leg);
        previous_end = end;
    }
    let exit = arrives - gate_seconds;
    if exit < previous_end {
        return None;
    }
    if exit > previous_end {
        legs.push(json!({
            "mode": "station_egress",
            "minutes": (exit - previous_end) / 60.0,
            "departure_seconds": previous_end, "arrival_seconds": exit,
            "route_ref": gate["station_egress"],
        }));
    }
    legs.push(walk(gate_path, exit, arrives));
    Some((legs, depart / 60.0))
}

fn public_legs(
    transit: &Value,
    walks: &Value,
    venue_id: &str,
    home_cut: &str,
    gate_window: [f64; 2],
) -> Option<(Vec<Value>, f64)> {
    let plans = transit["venues"][venue_id]["communes"][home_cut].as_array()?;
    plans.iter().find_map(|plan| {
        let lines = plan["legs"].as_array()?;
        let board = lines.first()?["board_stop_id"].as_str()?;
        let access = &walks["communes"][home_cut][board];
        scheduled_legs(plan, access, gate_window)
    })
}

fn venue_matches(data: &Path, venue_id: &str, point: [f64; 2]) -> bool {
    let Ok(mut places) = csv::Reader::from_path(data.join("named_places.csv")) else {
        return false;
    };
    places.records().flatten().any(|row| {
        &row[0] == venue_id
            && row
                .iter()
                .skip(3)
                .take(2)
                .zip(point)
                .all(|(coordinate, actual)| {
                    coordinate
                        .parse::<f64>()
                        .is_ok_and(|named| (actual - named).abs() < 0.000001)
                })
    })
}

pub(super) fn attach(
    data: &Path,
    fans: &Value,
    request: Request,
    point: [f64; 2],
    event: &mut Value,
) {
    let venue_id = &request.venue_id;
    let gate_arrival_minutes = request.gate_arrival_minutes.map(f64::from);
    let matches = venue_matches(data, venue_id, point);
    let car = matches
        .then(|| compressed_routes(data, "event_car_routes.json.gz"))
        .flatten()
        .unwrap_or(Value::Null);
    let car_venue = &car["venues"][venue_id];
    let transit = matches
        .then(|| read_routes(data, "event_transit_routes.json"))
        .unwrap_or(Value::Null);
    let walks = matches
        .then(|| compressed_routes(data, "event_transit_access.json.gz"))
        .flatten()
        .unwrap_or(Value::Null);
    let mut groups = Vec::new();
    let mut coverage = [[0.0; 2]; MODES.len()];
    for home in fans["by_home"].as_array().unwrap() {
        let home_cut = home["cut"].as_str().unwrap();
        for (index, mode) in MODES.into_iter().enumerate() {
            let people = home[mode].as_f64().unwrap();
            if people <= 0.0 {
                continue;
            }
            let id = format!("{home_cut}:{mode}");
            let route = match mode {
                "car" => car_legs(car_venue, home_cut).map(|legs| (legs, None)),
                "public" => public_legs(&transit, &walks, venue_id, home_cut, gate_arrival_minutes)
                    .map(|(legs, depart)| (legs, Some(depart))),
                _ => None,
            };
            let group = match route {
                Some((legs, scheduled_departure)) => {
                    let minutes: f64 = legs
                        .iter()
                        .map(|leg| leg["minutes"].as_f64().unwrap())
                        .sum();
                    coverage[index][0] += people;
                    json!({
                        "id": id, "home_cut": home_cut, "mode": mode, "people": people,
                        "status": "routed", "legs": legs,
                        "scheduled_departure_minutes": scheduled_departure,
                        "departure_window_minutes": gate_arrival_minutes.map(|arrival| arrival - minutes),
                    })
                }
                None => {
                    coverage[index][1] += people;
                    json!({
                        "id": id, "home_cut": home_cut, "mode": mode, "people": people,
                        "status": "route_unavailable", "reason": "No complete venue-bound route in the input packs.",
                        "legs": [], "departure_window_minutes": null,
                    })
                }
            };
            groups.push(group);
        }
    }
    let counts = MODES
        .into_iter()
        .zip(coverage)
        .map(|(mode, [routed, unrouted])| {
            (
                mode.to_owned(),
                json!({ "routed": routed, "unrouted": unrouted }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    event["itineraries"] = json!(groups);
    event["itinerary_coverage"] = Value::Object(counts);
    event["itinerary_assumptions"] = json!({
        "venue_id": venue_id,
        "gate_arrival_minutes": gate_arrival_minutes,
        "time_unit": "Chile local minutes since midnight",
        "car_walk_metres_per_minute": WALK_METRES_PER_MINUTE,
        "public_service_day": transit["service_day"],
        "public_source": transit["source"],
        "public_native_pack_sha256": transit["native_pack_sha256"],
        "public_walk_source": walks["source"],
        "car_source": car["source"],
        "car_closed_osm_ways": car_venue["closed_osm_ways"],
    });
}
