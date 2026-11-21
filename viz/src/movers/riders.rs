//! Follow each public trip through its transfers on the regular-weekday service clock.
use super::*;
use std::collections::HashSet;

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
    board_distance: f32,
    exit_distance: f32,
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

    fn door(self, routes: &Routes, distance: f32, copy: usize) -> Vec3 {
        match self.key {
            RideKey::Metro { track, .. } => {
                let (rail, dir) = routes.tracks[track].coach(distance + platform_offset(copy));
                rail + right(dir) * 1.4 + Vec3::Y * 0.58
            }
            RideKey::Surface { template, .. } => {
                let path = &routes.riders.surface[template].path;
                let (p, dir, _) = path.at(distance, &path.metres).unwrap();
                p + right(dir) * 2.9 + Vec3::Y * 0.25
            }
        }
    }
}

fn platform_offset(copy: usize) -> f32 {
    (copy % 17) as f32 * 2.2 - 17.6
}

fn platform_pose(line: &RailLine, distance: f32, copy: usize) -> (Vec3, Vec3) {
    let (rail, dir) = line.coach(distance);
    let side = right(dir);
    (rail + dir * platform_offset(copy) + side * 4.9 + Vec3::Y * 0.58, -side)
}

struct RideOptions {
    platform: Option<(usize, f32)>,
    gates: Option<[Vec<Vec3>; 2]>,
    metro_stations: Option<[String; 2]>,
    departures: Vec<Boarding>,
    max_dwell: f32,
}

impl RideOptions {
    fn next(&self, ready: f32, crossing: impl Fn(Boarding) -> f32) -> Option<Boarding> {
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
            let walk = crossing(trip);
            let aboard = match trip.key {
                RideKey::Metro { .. } => trip.arrival.max(ready) + walk,
                RideKey::Surface { .. } => trip.arrival.max(ready + walk),
            };
            if aboard > trip.departure {
                continue;
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
    articulated_starts: HashSet<(usize, i32)>,
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
        // DTPM class C denotes articulated buses. Older packs have no fleet assignments.
        let articulated_starts = data["planned_buses"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|bus| {
                bus["dtpm_type"].as_str().is_some_and(|kind| kind.starts_with('C'))
            })
            .map(|bus| {
                let start = bus["start"].as_i64().unwrap() as i32;
                (row_index(&bus["template"]), start.rem_euclid(DAY as i32))
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
                                board_distance: b.distance,
                                exit_distance: e.distance,
                                key: RideKey::Metro { track, run },
                            });
                        }
                    }
                }
            }
            let leg_key = ride_key(row[0].as_str().unwrap(), &row[1], &row[2]);
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
                        board_distance: b.distance,
                        exit_distance: e.distance,
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
            by_endpoints.insert(leg_key, options.len());
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
            articulated_starts,
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

    pub(super) fn interchange(&self, from: Option<usize>, to: Option<usize>) -> bool {
        let stations = |i: usize| self.options[i].metro_stations.as_ref();
        matches!((from.and_then(stations), to.and_then(stations)),
            (Some(a), Some(b)) if a[1] == b[0])
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

    pub(super) fn bus_rear(&self, template: usize, start: i32, now: f32, front: Vec3, heading: Vec3) -> Option<(Vec3, Vec3)> {
        if !self.articulated_starts.contains(&(template, start.rem_euclid(DAY as i32))) {
            return None;
        }
        let service = &self.surface[template];
        let distance = service.run.distance(now - start as f32)?;
        Some(service.path.articulated_rear(distance, front, heading, BUS_FRONT_TO_HINGE, BUS_TAIL_HALF))
    }

    pub(super) fn surface_pose(
        &self,
        template: usize,
        start: i32,
        now: f32,
    ) -> Option<(Vec3, Vec3, Vehicle)> {
        let service = &self.surface[template];
        let d = service.run.distance(now - start as f32)?;
        let (p, dir) = if service.vehicle == Vehicle::Bus {
            service.path.road_pose(d, Vehicle::Bus, 0, service.path.points.len() - 1)?
        } else {
            let (p, dir, _) = service.path.at(d, &service.path.metres)?;
            (p, dir)
        };
        Some((p, dir, service.vehicle))
    }
}

enum RiderAction {
    Unrouted,
    StreetWalk(Arc<Path>),
    StationWalk { from: Vec3, to: Vec3 },
    Wait { at: Vec3, facing: Vec3 },
    Ride(RideKey),
}

struct RiderStep {
    walked_before: f32,
    end_age: f32,
    action: RiderAction,
}
pub(super) struct RiderPlan {
    steps: Vec<RiderStep>,
    copy: usize,
}

#[derive(Clone, Copy)]
pub(super) enum RiderState {
    Walking(Vec3, Vec3),
    Waiting(Vec3, Vec3),
    Riding(RideKey),
}

impl RiderPlan {
    fn until(&mut self, end_age: f32, action: RiderAction) {
        if end_age > self.duration() {
            let walked_before = self.steps.last().map_or(0., |prev| {
                let length = match &prev.action {
                    RiderAction::StreetWalk(path) => *path.metres.last().unwrap(),
                    RiderAction::StationWalk { from, to } => from.distance(*to),
                    _ => 0.,
                };
                prev.walked_before + length
            });
            self.steps.push(RiderStep { end_age, action, walked_before });
        }
    }

    // Unmapped connections still consume time when choosing a departure.
    fn connect(&mut self, from: Vec3, to: Vec3, action: RiderAction) {
        self.until(self.duration() + from.distance(to) / Vehicle::Walk.speed(), action);
    }

    pub(super) fn build(path: &Path, legs: usize, start: f32, copy: usize, routes: &Routes) -> Self {
        let mut plan = Self { steps: Vec::new(), copy };
        let mut transfer = None;
        let mut street_end = None;
        let legs = &path.legs[..legs];
        for (i, leg) in legs.iter().enumerate() {
            let first = if i == 0 { 0 } else { legs[i - 1].end };
            let Some(index) = leg.scheduled else {
                if transfer.is_some() {
                    continue;
                }
                if let Some(walk) = &leg.walk_path {
                    let duration = *walk.seconds.last().unwrap();
                    if duration > 0. {
                        plan.until(plan.duration() + duration, RiderAction::StreetWalk(walk.clone()));
                        street_end = walk.points.last().map(|p| *p + Vec3::Y * 0.25);
                    }
                } else {
                    let duration = path.seconds[leg.end] - path.seconds[first];
                    if duration > 0. {
                        street_end = None;
                        plan.until(plan.duration() + duration, RiderAction::Unrouted);
                    }
                }
                continue;
            };
            let options = &routes.riders.options[index];
            let gate = path.points[first] + Vec3::Y * 0.25;
            let (wait, facing) = match options.platform {
                Some((track, distance)) => platform_pose(&routes.tracks[track], distance, copy),
                None => {
                    let along = ((copy % 5) as f32 - 2.) * 1.4;
                    let across = ((copy / 5 % 3) as f32 - 1.) * 1.2;
                    (gate + Vec3::new(along, 0., across), Vec3::X)
                }
            };
            let at_platform = transfer == Some(wait);
            let from = transfer.take().unwrap_or(gate);
            let waiting_at = if options.platform.is_none() {
                street_end
            } else if at_platform {
                Some(wait)
            } else {
                None
            };
            let waiting = || {
                waiting_at.map_or(RiderAction::Unrouted, |at| RiderAction::Wait { at, facing })
            };
            plan.connect(from, wait, waiting());
            let ready = start + plan.duration();
            let Some(service) = options.next(ready, |trip| {
                wait.distance(trip.door(routes, trip.board_distance, copy)) / Vehicle::Walk.speed()
            }) else {
                plan.until(DAY - start, waiting());
                break;
            };
            let door = service.door(routes, service.board_distance, copy);
            // Bus calls have zero reported dwell. Reserve the approach time before arrival.
            let approach = match service.key {
                RideKey::Metro { .. } => service.arrival,
                RideKey::Surface { .. } => service.arrival - wait.distance(door) / Vehicle::Walk.speed(),
            };
            plan.until(approach - start, waiting());
            let crossing = if at_platform {
                RiderAction::StationWalk { from: wait, to: door }
            } else {
                waiting()
            };
            plan.connect(wait, door, crossing);
            plan.until(service.alight - start, RiderAction::Ride(service.key));
            let exit_door = service.door(routes, service.exit_distance, copy);
            let exit = if let RideKey::Metro { track, .. } = service.key {
                let (platform, _) = platform_pose(&routes.tracks[track], service.exit_distance, copy);
                plan.connect(exit_door, platform, RiderAction::StationWalk {
                    from: exit_door,
                    to: platform,
                });
                street_end = None;
                platform
            } else {
                street_end = Some(exit_door);
                exit_door
            };
            // A blocked transfer ends on the last reachable platform.
            let next = path.legs[i + 1..].iter().find_map(|l| l.scheduled);
            if routes.riders.interchange(Some(index), next) {
                transfer = Some(exit);
            } else {
                plan.connect(exit, path.points[leg.end] + Vec3::Y * 0.25, RiderAction::Unrouted);
            }
        }
        plan
    }

    pub(super) fn duration(&self) -> f32 {
        self.steps.last().map_or(0., |s| s.end_age)
    }

    fn step_at(&self, age: f32) -> Option<(&RiderStep, f32, f32)> {
        if age < 0. { return None; }
        let i = self.steps.partition_point(|step| step.end_age <= age);
        let step = self.steps.get(i)?;
        let start_age = if i == 0 { 0. } else { self.steps[i - 1].end_age };
        Some((step, age - start_age, step.end_age - start_age))
    }

    pub(super) fn walking_model(&self, age: f32, scale: f32) -> Model {
        let (step, elapsed, duration) = self.step_at(age).unwrap();
        let travel = motion::walking_time(elapsed, duration, self.copy) * Vehicle::Walk.speed();
        Model::Walking(motion::gait_frame(step.walked_before + travel, scale, self.copy))
    }

    pub(super) fn at(&self, age: f32) -> Option<RiderState> {
        let (step, elapsed, duration) = self.step_at(age)?;
        Some(match step.action {
            RiderAction::Unrouted => return None,
            RiderAction::StreetWalk(ref walk) => {
                let t = motion::walking_time(elapsed, duration, self.copy);
                let (p, dir, _) = walk.at(t.min(*walk.seconds.last().unwrap()), &walk.seconds)?;
                RiderState::Walking(p + Vec3::Y * 0.25, dir)
            }
            RiderAction::StationWalk { from, to } => {
                let t = motion::walking_time(elapsed, duration, self.copy) / duration;
                RiderState::Walking(from.lerp(to, t), (to - from).normalize())
            }
            RiderAction::Wait { at, facing } => RiderState::Waiting(at, facing),
            RiderAction::Ride(key) => RiderState::Riding(key),
        })
    }
}
