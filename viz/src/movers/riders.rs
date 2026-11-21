//! Follow each public trip through its transfers on the regular-weekday service clock.
use super::*;
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(super) enum RideKey {
    Metro { track: usize, run: usize },
    Surface { template: usize, start: i32 },
}

#[derive(Clone, Copy)]
struct Boarding {
    arrival: f32,
    departure: f32,
    alight: f32,
    key: RideKey,
}

impl Boarding {
    fn shifted(mut self, day: i32) -> Self {
        let delta = day as f32 * DAY;
        self.arrival += delta;
        self.departure += delta;
        self.alight += delta;
        if let RideKey::Surface { start, .. } = &mut self.key {
            *start += day * DAY as i32;
        }
        self
    }
}

struct RideOptions {
    platform: Option<(usize, f32)>,
    gates: Option<[Vec<Vec3>; 2]>,
    metro_stations: Option<[String; 2]>,
    departures: Vec<Boarding>,
    max_dwell: f32,
}

impl RideOptions {
    fn next(&self, ready: f32) -> Option<Boarding> {
        let day = (ready / DAY).floor() as i32;
        let ready = ready.rem_euclid(DAY);
        let i = self
            .departures
            .partition_point(|trip| trip.departure < ready);
        let mut best: Option<Boarding> = None;
        for &trip in &self.departures[i..] {
            // A later departure can arrive first when its platform dwell is longer.
            if best.is_some_and(|b| trip.departure > b.arrival.max(ready) + self.max_dwell) {
                break;
            }
            if best.is_none_or(|b| {
                (trip.arrival.max(ready), trip.departure) < (b.arrival.max(ready), b.departure)
            }) {
                best = Some(trip);
            }
        }
        best.map(|trip| trip.shifted(day))
    }
}

struct SurfaceService {
    path: Arc<Path>,
    run: RailRun,
    starts: Vec<i32>,
    vehicle: Vehicle,
}

pub(super) struct RiderNetwork {
    by_endpoints: HashMap<(String, [i64; 4]), usize>,
    options: Vec<RideOptions>,
    surface: Vec<SurfaceService>,
}

fn ride_key(service: &str, from: &Value, to: &Value) -> (String, [i64; 4]) {
    let pos = [&from[0], &from[1], &to[0], &to[1]]
        .map(|n| (n.as_f64().unwrap() * 100000.).round() as i64);
    (service.to_owned(), pos)
}

fn row_index(v: &Value) -> usize {
    v.as_u64().unwrap() as usize
}

impl RiderNetwork {
    pub(super) fn load(tracks: &[RailLine], entrances: &stations::Entrances) -> Self {
        let data = gzip_json("web/data/rider_services.json.gz");
        let shapes: Vec<_> = data["shapes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| Path::single(points(row), Vehicle::Bus))
            .collect();
        let surface: Vec<_> = data["templates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|template| {
                let pattern = &data["patterns"][row_index(&template[0])];
                let path = shapes[row_index(&pattern[0])].clone();
                let calls = template[1]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(pattern[1].as_array().unwrap())
                    .enumerate()
                    .map(|(station, (times, vertex))| {
                        let k = vertex.as_f64().unwrap();
                        let a = path.metres[k.floor() as usize];
                        let b = path.metres[k.ceil() as usize];
                        Call {
                            station,
                            distance: a + (b - a) * k.fract() as f32,
                            arrival: times[0].as_f64().unwrap() as f32,
                            departure: times[1].as_f64().unwrap() as f32,
                        }
                    })
                    .collect();
                SurfaceService {
                    path,
                    run: RailRun { calls },
                    starts: template[2]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_i64().unwrap() as i32)
                        .collect(),
                    vehicle: if template[3] == "0" {
                        Vehicle::Rail
                    } else {
                        Vehicle::Bus
                    },
                }
            })
            .collect();
        let mut by_endpoints = HashMap::new();
        let mut options = Vec::new();
        for row in data["legs"].as_array().unwrap() {
            let mut departures = Vec::new();
            let mut platform = None;
            let mut gates = None;
            let mut metro_stations = None;
            for candidate in row[3].as_array().unwrap() {
                let track = row_index(&candidate[0]);
                let from = row_index(&candidate[1]);
                let to = row_index(&candidate[2]);
                metro_stations = Some([from, to].map(|s| tracks[track].stations[s].clone()));
                gates = Some([from, to].map(|s| {
                    entrances[&tracks[track].stations[s]]
                        .iter()
                        .map(|e| e.position)
                        .collect()
                }));
                for (run, service) in tracks[track].runs.iter().enumerate() {
                    let board = service.calls.iter().find(|s| s.station == from);
                    let exit = service.calls.iter().find(|s| s.station == to);
                    if let (Some(b), Some(e)) = (board, exit) {
                        if e.arrival >= b.departure {
                            platform = Some((track, b.distance));
                            departures.push(Boarding {
                                arrival: b.arrival,
                                departure: b.departure,
                                alight: e.arrival,
                                key: RideKey::Metro { track, run },
                            });
                        }
                    }
                }
            }
            for candidate in row[4].as_array().unwrap() {
                let template = row_index(&candidate[0]);
                let service = &surface[template];
                let b = &service.run.calls[row_index(&candidate[1])];
                let e = &service.run.calls[row_index(&candidate[2])];
                for &start in &service.starts {
                    let clock = start as f32;
                    departures.push(Boarding {
                        arrival: clock + b.arrival,
                        departure: clock + b.departure,
                        alight: clock + e.arrival,
                        key: RideKey::Surface { template, start },
                    });
                }
            }
            // The scene repeats a regular weekday. A late service can still be running after midnight.
            let adjacent: Vec<_> = departures
                .iter()
                .flat_map(|&d| {
                    let yesterday = (d.departure >= DAY).then(|| d.shifted(-1));
                    [yesterday, Some(d.shifted(1))].into_iter().flatten()
                })
                .collect();
            departures.extend(adjacent);
            departures.sort_by(|a, b| a.departure.total_cmp(&b.departure));
            let max_dwell = departures
                .iter()
                .map(|c| c.departure - c.arrival)
                .fold(0., f32::max);
            by_endpoints.insert(
                ride_key(row[0].as_str().unwrap(), &row[1], &row[2]),
                options.len(),
            );
            options.push(RideOptions {
                gates,
                metro_stations,
                platform,
                departures,
                max_dwell,
            });
        }
        Self {
            by_endpoints,
            options,
            surface,
        }
    }

    pub(super) fn gate(&self, leg: usize, exit: bool, towards: Vec3) -> Vec3 {
        *self.options[leg].gates.as_ref().unwrap()[usize::from(exit)]
            .iter()
            .min_by(|a, b| {
                a.distance_squared(towards)
                    .total_cmp(&b.distance_squared(towards))
            })
            .unwrap()
    }

    pub(super) fn leg(&self, row: &Value, segment: &Value) -> Option<usize> {
        let service = segment["service"].as_str()?;
        let p = &row["path"];
        let key = ride_key(
            service,
            &p[row_index(&segment["start"])],
            &p[row_index(&segment["end"])],
        );
        Some(self.by_endpoints[&key])
    }

    // The engine changes station access but publishes no replacement itinerary for a group.
    pub(super) fn accessible_legs(&self, path: &Path, closed: &std::collections::HashSet<&str>) -> usize {
        let blocked = path.legs.iter().position(|leg| {
            leg.scheduled
                .and_then(|i| self.options[i].metro_stations.as_ref())
                .is_some_and(|ids| ids.iter().any(|id| closed.contains(id.as_str())))
        });
        let Some(mut end) = blocked else { return path.legs.len() };
        while end > 0 && path.legs[end - 1].vehicle == Vehicle::Walk {
            end -= 1;
        }
        end
    }

    pub(super) fn for_each_surface(
        &self,
        now: f32,
        mut draw: impl FnMut((usize, i32), Vec3, Vec3, Vehicle),
    ) {
        for (template, service) in self.surface.iter().enumerate() {
            let first = service.run.calls.first().unwrap().arrival;
            let last = service.run.calls.last().unwrap().departure;
            // GTFS starts belong to a service day and can extend beyond midnight.
            let first_day = ((now - last - *service.starts.last().unwrap() as f32) / DAY).ceil() as i32;
            let last_day = ((now - first - service.starts[0] as f32) / DAY).floor() as i32;
            for day in first_day..=last_day {
                let offset = day * DAY as i32;
                let time = now - offset as f32;
                let from = service.starts.partition_point(|&s| (s as f32) < time - last);
                let to = service.starts.partition_point(|&s| (s as f32) <= time - first);
                for &start in &service.starts[from..to] {
                    let start = start + offset;
                    if let Some(d) = service.run.distance(now - start as f32)
                        && let Some((p, dir, _)) = service.path.at(d, &service.path.metres)
                    {
                        draw((template, start), p, dir, service.vehicle);
                    }
                }
            }
        }
    }
}

enum RiderAction {
    Walk {
        path_time: f32,
    },
    Wait {
        at: Vec3,
        platform: Option<(usize, f32)>,
    },
    Ride(RideKey),
}

struct RiderStep {
    end_age: f32,
    action: RiderAction,
}
pub(super) struct RiderPlan {
    steps: Vec<RiderStep>,
}

pub(super) enum RiderState {
    Walking(Vec3, Vec3),
    Waiting(Vec3, Option<(usize, f32)>),
    Riding(RideKey),
}

impl RiderPlan {
    pub(super) fn build(path: &Path, legs: usize, start: f32, network: &RiderNetwork) -> Self {
        let mut steps = Vec::new();
        let mut first = 0;
        let mut now = start;
        for leg in &path.legs[..legs] {
            if let Some(index) = leg.scheduled {
                let at = path.points[first];
                let platform = network.options[index].platform;
                let Some(service) = network.options[index].next(now) else {
                    // No published service joins these stops.
                    if now < DAY {
                        steps.push(RiderStep {
                            end_age: DAY - start,
                            action: RiderAction::Wait { at, platform },
                        });
                    }
                    break;
                };
                let board = now.max(service.arrival);
                if board > now {
                    steps.push(RiderStep {
                        end_age: board - start,
                        action: RiderAction::Wait { at, platform },
                    });
                }
                now = service.alight;
                steps.push(RiderStep {
                    end_age: now - start,
                    action: RiderAction::Ride(service.key),
                });
            } else {
                let path_time = path.seconds[first];
                let duration = path.seconds[leg.end] - path_time;
                if duration > 0. {
                    now += duration;
                    steps.push(RiderStep {
                        end_age: now - start,
                        action: RiderAction::Walk { path_time },
                    });
                }
            }
            first = leg.end;
        }
        Self { steps }
    }

    pub(super) fn duration(&self) -> f32 {
        self.steps.last().map_or(0., |s| s.end_age)
    }

    pub(super) fn at(&self, age: f32, path: &Path) -> Option<RiderState> {
        if age < 0. {
            return None;
        }
        let i = self.steps.partition_point(|step| step.end_age <= age);
        let step = self.steps.get(i)?;
        let start_age = if i == 0 {
            0.
        } else {
            self.steps[i - 1].end_age
        };
        Some(match step.action {
            RiderAction::Walk { path_time } => {
                let (p, dir, _) = path.at(path_time + age - start_age, &path.seconds)?;
                RiderState::Walking(p, dir)
            }
            RiderAction::Wait { at, platform } => RiderState::Waiting(at, platform),
            RiderAction::Ride(key) => RiderState::Riding(key),
        })
    }
}
