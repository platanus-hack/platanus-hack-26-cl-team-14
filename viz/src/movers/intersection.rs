//! Display-only admission for conflicting paths at shared ground road nodes.
mod topology;

use super::{BIN, DAY, Life, Path, Replay, departure_offset};
use bevy::prelude::*;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, VecDeque},
    sync::Arc,
    time::Instant,
};
use topology::{Topology, cell, conflict};

const CLEARANCE: u32 = 4_500;
const HEADWAY: u32 = 1_300;
const WAVE: u32 = 9_000;
const STOP_DISTANCE: f32 = 14.;
const SCALE_RADIUS: f32 = 155.;

type Car = (usize, usize, usize);
type Calendar = BinaryHeap<Reverse<(u32, bool, usize, usize)>>;

struct Trip {
    car: Car,
    start: f32,
    approaches: Arc<[Approach]>,
}

#[derive(Clone, Copy)]
struct Approach {
    junction: usize,
    movement: usize,
    stop_age: f32,
}

#[derive(Clone, Copy)]
struct Pending {
    trip: usize,
    stop: usize,
    arrive: u32,
}

struct Movement {
    lane: usize,
    curve: [Vec2; 6],
}

#[derive(Default)]
struct Lane {
    ready: u32,
    previous: Option<usize>,
    waiting: VecDeque<Pending>,
}

#[derive(Default)]
struct Site {
    approaches: Vec<(Vec2, Vec2)>,
    movements: Vec<Movement>,
    conflicts: Vec<Vec<bool>>,
    lanes: Vec<Lane>,
    occupied: Vec<(usize, u32)>,
    wave_end: u32,
    wake: Option<u32>,
}

impl Site {
    fn register(&mut self, entry: Vec2, heading: Vec2, curve: [Vec2; 6]) -> usize {
        let lane = self
            .approaches
            .iter()
            .position(|&(p, d)| entry.distance_squared(p) < 5_f32.powi(2) && heading.dot(d) > 0.84)
            .unwrap_or_else(|| {
                self.approaches.push((entry, heading));
                self.approaches.len() - 1
            });
        if let Some(index) = self.movements.iter().position(|m| {
            m.lane == lane
                && m.curve
                    .iter()
                    .zip(curve)
                    .all(|(a, b)| a.distance_squared(b) < 1.5_f32.powi(2))
        }) {
            return index;
        }
        let index = self.movements.len();
        let mut conflicts = Vec::with_capacity(index + 1);
        for (i, m) in self.movements.iter().enumerate() {
            let blocked = conflict(&m.curve, &curve);
            self.conflicts[i].push(blocked);
            conflicts.push(blocked);
        }
        conflicts.push(false);
        self.conflicts.push(conflicts);
        self.movements.push(Movement { lane, curve });
        index
    }

    fn active(&self) -> bool {
        self.movements.iter().enumerate().any(|(i, movement)| {
            self.movements
                .iter()
                .enumerate()
                .any(|(j, other)| movement.lane != other.lane && self.conflicts[i][j])
        })
    }

    fn blocked(&self, movement: usize) -> bool {
        self.occupied
            .iter()
            .any(|&(other, _)| self.conflicts[movement][other])
    }

    fn admit(&mut self, now: u32, trips: &[Trip]) -> Option<(Pending, Option<usize>)> {
        self.occupied.retain(|&(_, until)| until > now);
        let move_for = |p: Pending| trips[p.trip].approaches[p.stop].movement;
        // Stop extending a green wave when a conflicting approach is waiting.
        // This bounds cross-stream starvation without serializing common movements.
        if now >= self.wave_end
            && self.lanes.iter().any(|lane| {
                lane.waiting
                    .front()
                    .is_some_and(|&p| self.blocked(move_for(p)))
            })
        {
            return None;
        }
        let lane = self
            .lanes
            .iter()
            .enumerate()
            .filter_map(|(i, lane)| lane.waiting.front().map(|&p| (i, lane, p)))
            .filter(|&(_, lane, p)| lane.ready <= now && !self.blocked(move_for(p)))
            .min_by_key(|&(_, _, p)| (p.arrive, p.trip))?
            .0;
        let pending = self.lanes[lane].waiting.pop_front().unwrap();
        let movement = move_for(pending);
        if self.occupied.is_empty() {
            self.wave_end = now + WAVE;
        }
        if let Some((_, until)) = self.occupied.iter_mut().find(|(id, _)| *id == movement) {
            *until = now + CLEARANCE;
        } else {
            self.occupied.push((movement, now + CLEARANCE));
        }
        self.lanes[lane].ready = now + HEADWAY;
        Some((pending, self.lanes[lane].previous))
    }

    fn next_wake(&self, now: u32) -> Option<u32> {
        if self.lanes.iter().all(|lane| lane.waiting.is_empty()) {
            return None;
        }
        self.occupied
            .iter()
            .map(|&(_, until)| until)
            .chain(
                self.lanes
                    .iter()
                    .filter(|lane| !lane.waiting.is_empty())
                    .map(|lane| lane.ready),
            )
            .chain([self.wave_end])
            .filter(|&time| time > now)
            .min()
    }
}

struct Pass {
    stop_age: f32,
    arrive: f32,
    release: f32,
    previous: Option<usize>,
}

impl Pass {
    fn progress(&self, now: f32) -> f32 {
        if now < self.release {
            (now - self.arrive).min(0.)
        } else {
            now - self.release
        }
    }
}

#[derive(Default)]
pub(super) struct Queues {
    topology: Option<Topology>,
    controls: HashMap<(i32, i32), Vec<Vec2>>,
    passes: Vec<Pass>,
    by_car: HashMap<Car, Vec<usize>>,
    now: f32,
    pub(super) longest_wait: f32,
}

impl Queues {
    pub(super) fn rebuild(&mut self, replay: &Replay) {
        self.passes.clear();
        self.by_car.clear();
        self.controls.clear();
        self.longest_wait = 0.;
        if replay.departures.is_empty() {
            return;
        }
        let started = Instant::now();
        let topology = self.topology.get_or_insert_with(Topology::load);
        let mut sites = HashMap::new();
        let mut routes = HashMap::<*const Path, Vec<Approach>>::new();
        for flow in replay.departures.iter().flatten() {
            if !matches!(flow.lifecycle, Life::Leave)
                && !flow.cars.is_empty()
                && !flow.path.origin_preview
            {
                routes
                    .entry(Arc::as_ptr(&flow.path))
                    .or_insert_with(|| topology.approaches(&flow.path, replay, &mut sites));
            }
        }
        sites.retain(|_, site| site.active());
        for (&id, site) in &mut sites {
            let p = topology.nodes[id].1;
            self.controls
                .entry(cell(p, SCALE_RADIUS))
                .or_default()
                .push(p);
            site.lanes = (0..site.approaches.len())
                .map(|_| Lane::default())
                .collect();
        }
        let mut routes: HashMap<_, Arc<[Approach]>> = routes
            .into_iter()
            .map(|(key, mut approaches)| {
                approaches.retain(|a| sites.contains_key(&a.junction));
                let mut previous = f32::NEG_INFINITY;
                approaches.retain(|a| {
                    if a.stop_age - previous < CLEARANCE as f32 / 1_000. {
                        return false;
                    }
                    previous = a.stop_age;
                    true
                });
                (key, approaches.into())
            })
            .collect();

        let mut trips = Vec::new();
        let mut calendar = Calendar::new();
        for (bin, groups) in replay.departures.iter().enumerate() {
            for (group, flow) in groups.iter().enumerate() {
                if matches!(flow.lifecycle, Life::Leave) {
                    continue;
                }
                let Some(approaches) = routes.get_mut(&Arc::as_ptr(&flow.path)) else {
                    continue;
                };
                let Some(first) = approaches.first() else {
                    continue;
                };
                for copy in 0..flow.cars.len() {
                    let start = bin as f32 * BIN + departure_offset(flow.phase, copy);
                    calendar.push(Reverse((ms(start + first.stop_age), false, trips.len(), 0)));
                    trips.push(Trip {
                        car: (bin, group, copy),
                        start,
                        approaches: approaches.clone(),
                    });
                }
            }
        }

        let mut coverage = (0usize, 0usize);
        while let Some(Reverse((now, wake, id, stop))) = calendar.pop() {
            let junction = if wake {
                let site = sites.get_mut(&id).unwrap();
                if site.wake != Some(now) {
                    continue;
                }
                id
            } else {
                let approach = trips[id].approaches[stop];
                let site = sites.get_mut(&approach.junction).unwrap();
                let lane = site.movements[approach.movement].lane;
                site.lanes[lane].waiting.push_back(Pending {
                    trip: id,
                    stop,
                    arrive: now,
                });
                approach.junction
            };
            let site = sites.get_mut(&junction).unwrap();
            while let Some((pending, previous)) = site.admit(now, &trips) {
                let trip = &trips[pending.trip];
                let approach = trip.approaches[pending.stop];
                let release = now as f32 / 1_000.;
                let lane = site.movements[approach.movement].lane;
                let index = self.passes.len();
                site.lanes[lane].previous = Some(index);
                self.by_car.entry(trip.car).or_default().push(index);
                self.passes.push(Pass {
                    stop_age: approach.stop_age,
                    release,
                    previous,
                    arrive: pending.arrive as f32 / 1_000.,
                });
                self.longest_wait = self
                    .longest_wait
                    .max(release - trip.start - approach.stop_age);
                coverage.0 += usize::from(now > pending.arrive);
                coverage.1 += usize::from(pending.stop > 0);
                if let Some(following) = trip.approaches.get(pending.stop + 1) {
                    let next = now + ms(following.stop_age - approach.stop_age);
                    calendar.push(Reverse((next, false, pending.trip, pending.stop + 1)));
                }
            }
            let next = site.next_wake(now);
            if next != site.wake || next == Some(now) {
                site.wake = next;
                if let Some(next) = next {
                    calendar.push(Reverse((next, true, junction, 0)));
                }
            }
        }
        info!(
            "Road yielding: {} mapped nodes, {} active controls, {} cars, {} admissions ({} waits, {} downstream), max cumulative delay {:.1}s; rebuild {:.0}ms",
            topology.nodes.len(),
            sites.len(),
            trips.len(),
            self.passes.len(),
            coverage.0,
            coverage.1,
            self.longest_wait,
            started.elapsed().as_secs_f64() * 1_000.
        );
    }

    pub(super) fn seek(&mut self, now: f32) {
        self.now = now;
    }

    pub(super) fn car_scale(&self, position: Vec3, scale: f32) -> f32 {
        if scale <= 1.35 {
            return scale;
        }
        let p = position.xz();
        let (x, z) = cell(p, SCALE_RADIUS);
        let mut distance = f32::INFINITY;
        for i in x - 1..=x + 1 {
            for j in z - 1..=z + 1 {
                for &site in self.controls.get(&(i, j)).into_iter().flatten() {
                    distance = distance.min(p.distance_squared(site));
                }
            }
        }
        let blend = ((distance.sqrt() - 125.) / 30.).clamp(0., 1.);
        1.35 + (scale - 1.35) * blend
    }

    pub(super) fn age(&self, day_bin: i32, group: usize, copy: usize, age: f32) -> f32 {
        let key = (day_bin.rem_euclid(48) as usize, group, copy);
        let Some(stops) = self.by_car.get(&key) else {
            return age;
        };
        let now = self.now + if day_bin < 0 { DAY } else { 0. };
        let stop = stops
            .partition_point(|&i| self.passes[i].release <= now)
            .min(stops.len() - 1);
        let pass = &self.passes[stops[stop]];
        let mut progress = pass.progress(now);
        if now < pass.release {
            let mut previous = pass.previous;
            let mut gap = HEADWAY as f32 / 1_000.;
            while let Some(index) = previous {
                let leader = &self.passes[index];
                progress = progress.min(leader.progress(now) - gap);
                if now >= leader.release {
                    break;
                }
                previous = leader.previous;
                gap += HEADWAY as f32 / 1_000.;
            }
        }
        let age = pass.stop_age + progress;
        if stop == 0 {
            age
        } else {
            age.max(self.passes[stops[stop - 1]].stop_age)
        }
    }
}

fn ms(seconds: f32) -> u32 {
    (seconds * 1_000.).ceil() as u32
}
