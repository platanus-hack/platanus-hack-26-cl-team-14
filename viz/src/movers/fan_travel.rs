//! One sample of the engine's venue-bound itineraries for vehicles and the gate walk.
use super::*;

pub(crate) const PEOPLE_PER_GROUP: f64 = 192.;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FanMode {
    Public,
    Car,
    Other,
}

#[derive(Clone, Copy)]
struct Ride {
    key: RideKey,
    board: f32,
    alight: f32,
}

struct CarTrip {
    path: Arc<Path>,
    start: f32,
    end: f32,
    style: u16,
    relocated: bool,
}

pub(crate) struct FanGroup {
    pub home_cut: String,
    pub index: usize,
    pub mode: FanMode,
    pub people: f64,
    pub alight_seconds: f32,
    pub gate_seconds: f32,
    pub walk_path: Vec<Vec3>,
    rides: Vec<Ride>,
    car: Option<CarTrip>,
}

#[derive(Resource, Default)]
pub(crate) struct FanJourneys(Vec<Vec<FanGroup>>);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RefreshFanJourneys;

impl FanJourneys {
    pub(crate) fn for_venue(&self, venue: usize) -> &[FanGroup] {
        self.0.get(venue).map_or(&[], Vec::as_slice)
    }

    pub(super) fn draw(
        &self,
        now: f32,
        routes: &Routes,
        road_edits: &road_edits::TrafficEdits,
        trains: &mut [Vec<Train<'_>>],
        buses: &mut std::collections::BTreeMap<(usize, i32), (Paint, usize)>,
        visible: &impl Fn(Vec3, f32) -> bool,
        size_at: &impl Fn(Vec3) -> f32,
        place: &mut impl FnMut(Model, Paint, Transform),
    ) {
        let mut aboard = HashMap::<RideKey, f64>::new();
        for group in self.0.iter().flatten() {
            if let Some(car) = &group.car
                && (car.start..car.end).contains(&now)
            {
                let fraction = (now - car.start) / (car.end - car.start);
                let age = fraction * car.path.seconds.last().unwrap();
                if let Some((p, dir, _, metres)) = car.path.trip_pose(age, 0)
                    && visible(p, 100.)
                    && !road_edits.hides(p, dir)
                    && let Some(at) = routes.water.surface_position(
                        &car.path,
                        metres,
                        p + right(dir) * 1.6 + Vec3::Y * 0.25,
                        dir,
                    )
                {
                    let transform = pose(at, dir, Vec3::splat(size_at(at)));
                    place(Model::Car(car.style), Paint::Plain, transform);
                    if car.relocated {
                        place(Model::CarChange, Paint::Rerouted, transform);
                    }
                }
            }
            for ride in &group.rides {
                if (ride.board..ride.alight).contains(&now) {
                    *aboard.entry(ride.key).or_default() += group.people;
                }
            }
        }
        // Quantize once per service so several small groups share a passenger mark.
        for (service, people) in aboard {
            let marks = (people / PERSONS_PER_MOVER).round() as usize;
            if marks == 0 {
                continue;
            }
            match service {
                RideKey::Metro { track, run } => {
                    if let Some(train) = trains[track].iter_mut().find(|t| t.run_index == run) {
                        train.riders += marks;
                    }
                }
                RideKey::Surface { template, start } => {
                    buses
                        .entry((template, start))
                        .or_insert((Paint::Plain, 0))
                        .1 += marks;
                }
            }
        }
    }
}

fn ride_option(leg: &Value, option: &Value, routes: &Routes) -> Option<Ride> {
    let from = option["from"].as_u64()? as usize;
    let to = option["to"].as_u64()? as usize;
    if let Some(track) = option["track"].as_u64() {
        let track = track as usize;
        let run = option["run"].as_u64()? as usize;
        let calls = &routes.tracks.get(track)?.runs.get(run)?.calls;
        let board = calls.iter().find(|c| c.station == from)?.departure;
        let alight = calls.iter().find(|c| c.station == to)?.arrival;
        return (alight > board).then_some(Ride {
            key: RideKey::Metro { track, run },
            board,
            alight,
        });
    }
    let _service = leg["service_id"].as_str()?;
    let template = option["template"].as_u64()? as usize;
    let start = option["start_seconds"].as_i64()? as i32;
    let (board, alight) = routes.riders.fan_ride_times(template, start, from, to)?;
    Some(Ride {
        key: RideKey::Surface { template, start },
        board,
        alight,
    })
}

// The nth option on each leg belongs to the same end-to-end schedule supplied by V1.
fn schedule(legs: &[Value], index: usize, routes: &Routes) -> Option<(Vec<Ride>, f32)> {
    let mut rides = Vec::new();
    let mut walk_seconds = 0.;
    let mut first_departure = 0.;
    let mut previous_exit = None;
    for leg in legs {
        if let Some(options) = leg["departure_options"].as_array() {
            let ride = ride_option(leg, options.get(index)?, routes)?;
            if let Some(exit) = previous_exit {
                if ride.board + 0.01 < exit + walk_seconds {
                    return None;
                }
            } else {
                first_departure = ride.board - walk_seconds;
            }
            previous_exit = Some(ride.alight);
            walk_seconds = 0.;
            rides.push(ride);
        } else {
            walk_seconds += leg["minutes"].as_f64()? as f32 * 60.;
        }
    }
    Some((rides, first_departure))
}

fn groups(event: &Value, routes: &Routes, fleet: &fleet::Fleet, relocated: bool) -> Vec<FanGroup> {
    let mut result = Vec::new();
    for row in event["itineraries"].as_array().into_iter().flatten() {
        let Some(people) = row["people"].as_f64() else {
            continue;
        };
        if people <= 0. || row["status"] != "routed" {
            continue;
        }
        // Each routed origin appears, including those below the target figure weight.
        let count = (people / PEOPLE_PER_GROUP).round().max(1.) as usize;
        let first = result.len();
        let Some(legs) = row["legs"].as_array() else {
            continue;
        };
        let Some(walk) = legs.last().filter(|l| l["mode"] == "walk") else {
            continue;
        };
        let Some(coords) = walk["geometry"]["coordinates"].as_array() else {
            continue;
        };
        if coords.len() < 2 {
            continue;
        }
        let walk_path: Vec<_> = coords.iter().map(point).collect();
        let walk_seconds = walk["minutes"].as_f64().unwrap_or_default() as f32 * 60.;
        let Some(window) = row["departure_window_minutes"]
            .as_array()
            .filter(|w| w.len() == 2)
        else {
            continue;
        };
        let lo = window[0].as_f64().unwrap() as f32 * 60.;
        let hi = window[1].as_f64().unwrap() as f32 * 60.;
        let mode = match row["mode"].as_str() {
            Some("public") => FanMode::Public,
            Some("car") => FanMode::Car,
            Some("other") => FanMode::Other,
            _ => continue,
        };
        let home = row["home_cut"].as_str().unwrap();
        let road = legs.iter().find(|l| l["mode"] == "car").and_then(|l| {
            let geometry = &l["geometry"]["coordinates"];
            if geometry.as_array()?.len() < 2 {
                return None;
            }
            let mut path = Path::single(points(geometry), Vehicle::Car);
            if let Some(raw) = l["crossings"].as_array() {
                let spans = serde_json::from_value(Value::Array(raw.clone())).ok()?;
                road_water::set_crossings(&mut path, Some(&spans));
            }
            Some((path, l["minutes"].as_f64()? as f32 * 60.))
        });
        if mode == FanMode::Car && road.is_none() {
            continue;
        }
        let schedules: Vec<_> = legs
            .iter()
            .find_map(|l| l["departure_options"].as_array())
            .into_iter()
            .flat_map(|o| 0..o.len())
            .filter_map(|i| schedule(legs, i, routes))
            .filter(|(_, departure)| (lo..hi).contains(departure))
            .collect();
        if mode == FanMode::Public && schedules.is_empty() {
            continue;
        }
        let styles = if road.is_some() {
            fleet.cars(home, first as u64, count)
        } else {
            Vec::new()
        };
        for copy in 0..count {
            let q = (copy as f32 + 0.5) / count as f32;
            let mut group = FanGroup {
                home_cut: home.to_owned(),
                index: first + copy,
                mode,
                people: people / count as f64,
                alight_seconds: lo + q * (hi - lo),
                gate_seconds: 0.,
                walk_path: walk_path.clone(),
                rides: Vec::new(),
                car: None,
            };
            if let Some((path, duration)) = &road {
                let start = group.alight_seconds;
                group.alight_seconds = start + duration;
                group.car = Some(CarTrip {
                    path: path.clone(),
                    start,
                    end: group.alight_seconds,
                    style: styles[copy],
                    relocated,
                });
            } else if mode == FanMode::Public {
                let (rides, _) = &schedules[(q * schedules.len() as f32) as usize];
                // The train loses its riders at the platform. The street figure
                // appears only after they traverse the GTFS station exit.
                let exit_seconds: f32 = legs
                    .iter()
                    .rev()
                    .skip(1)
                    .take_while(|leg| leg["mode"] == "station_egress")
                    .filter_map(|leg| leg["minutes"].as_f64())
                    .map(|minutes| (minutes * 60.) as f32)
                    .sum();
                group.alight_seconds = rides.last().unwrap().alight + exit_seconds;
                group.rides = rides.clone();
            }
            group.gate_seconds = group.alight_seconds + walk_seconds;
            result.push(group);
        }
    }
    result
}

pub(super) fn refresh(
    result: Res<Scenario>,
    routes: Res<Routes>,
    fleet: Res<fleet::Fleet>,
    replay: Res<Replay>,
    mut journeys: ResMut<FanJourneys>,
) {
    if result.is_changed() {
        journeys.0 = result.0["events"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|event| {
                let relocated = replay.stadium.as_ref().is_some_and(|footprint| {
                    let (Some(lon), Some(lat)) =
                        (event["longitude"].as_f64(), event["latitude"].as_f64())
                    else {
                        return false;
                    };
                    footprint.covers(world(lon, lat))
                });
                groups(event, &routes, &fleet, relocated)
            })
            .collect();
    }
}
