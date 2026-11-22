//! Attach venue-bound network paths to the fixed fan groups.
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;
use std::process::Command;

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

fn car_routes(data: &Path) -> Option<Value> {
    let output = Command::new("gzip")
        .arg("-dc")
        .arg(data.join("event_car_routes.json.gz"))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
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

pub(super) fn attach(data: &Path, fans: &Value, request: Request, event: &mut Value) {
    let venue_id = &request.venue_id;
    let gate_arrival_minutes = request.gate_arrival_minutes.map(f64::from);
    let car = car_routes(data).unwrap_or(Value::Null);
    let car_venue = &car["venues"][venue_id];
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
            let route = if mode == "car" {
                car_legs(car_venue, home_cut)
            } else {
                None
            };
            let group = match route {
                Some(legs) => {
                    let minutes: f64 = legs.iter().map(|leg| leg["minutes"].as_f64().unwrap()).sum();
                    coverage[index][0] += people;
                    json!({
                        "id": id, "home_cut": home_cut, "mode": mode, "people": people,
                        "status": "routed", "legs": legs,
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
    let counts = MODES.into_iter().zip(coverage).map(|(mode, [routed, unrouted])| {
        (mode.to_owned(), json!({ "routed": routed, "unrouted": unrouted }))
    }).collect::<serde_json::Map<_, _>>();
    event["itineraries"] = json!(groups);
    event["itinerary_coverage"] = Value::Object(counts);
    event["itinerary_assumptions"] = json!({
        "venue_id": venue_id,
        "gate_arrival_minutes": gate_arrival_minutes,
        "time_unit": "Chile local minutes since midnight",
        "walk_metres_per_minute": WALK_METRES_PER_MINUTE,
        "car_source": car["source"],
        "car_closed_osm_ways": car_venue["closed_osm_ways"],
    });
}
