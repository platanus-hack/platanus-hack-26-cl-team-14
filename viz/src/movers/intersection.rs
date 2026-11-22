//! Display-only admission for conflicting paths at shared ground road nodes.
mod topology;

use super::{BIN, DAY, Life, Path, Replay, Vehicle, departure_offset};
use bevy::prelude::*;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    sync::Arc,
    time::Instant,
};
use topology::{Sample, Topology, Window, cell, conflict_windows, register};

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
    curve: Vec<Sample>,
    horizon: u32,
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
    conflicts: Vec<Vec<Vec<Window>>>,
    lanes: Vec<Lane>,
    occupied: Vec<(usize, u32)>,
    wave_end: u32,
    wake: Option<u32>,
}

impl Site {
    fn register(&mut self, entry: Vec2, heading: Vec2, curve: Vec<Sample>) -> usize {
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
                && m.curve.len() == curve.len()
                && m.curve.iter().zip(&curve).all(|(a, b)| {
                    a.time == b.time && a.position.distance_squared(b.position) < 0.25_f32.powi(2)
                })
        }) {
            return index;
        }
        let index = self.movements.len();
        let mut conflicts = Vec::with_capacity(index + 1);
        let mut horizon = 0;
        for (i, movement) in self.movements.iter_mut().enumerate() {
            let (older_movement, newer_movement) = conflict_windows(&movement.curve, &curve);
            if let Some(window) = newer_movement.last() {
                movement.horizon = movement.horizon.max(*window.end());
            }
            if let Some(window) = older_movement.last() {
                horizon = horizon.max(*window.end());
            }
            self.conflicts[i].push(older_movement);
            conflicts.push(newer_movement);
        }
        conflicts.push(Vec::new());
        self.conflicts.push(conflicts);
        self.movements.push(Movement {
            lane,
            curve,
            horizon,
        });
        index
    }

    fn active(&self) -> bool {
        self.movements.iter().enumerate().any(|(i, movement)| {
            self.movements
                .iter()
                .enumerate()
                .any(|(j, other)| movement.lane != other.lane && !self.conflicts[i][j].is_empty())
        })
    }

    fn blocked_until(&self, movement: usize, now: u32) -> Option<u32> {
        self.occupied
            .iter()
            .filter_map(|&(other, release)| {
                let offset = now - release;
                let windows = &self.conflicts[movement][other];
                let i = windows.partition_point(|window| *window.end() < offset);
                let window = windows.get(i).filter(|window| window.contains(&offset))?;
                Some(release + window.end() + 1)
            })
            .max()
    }

    fn admit(&mut self, now: u32, trips: &[Trip]) -> Option<(Pending, Option<usize>)> {
        self.occupied
            .retain(|&(id, release)| release + self.movements[id].horizon >= now);
        let move_for = |p: Pending| trips[p.trip].approaches[p.stop].movement;
        let ready = self.lanes.iter().enumerate().filter_map(|(i, lane)| {
            (lane.ready <= now)
                .then(|| lane.waiting.front().map(|&p| (i, p)))
                .flatten()
        });
        let order = |&(_, p): &(usize, Pending)| (p.arrive, p.trip);
        // End the wave if the oldest ready approach needs the next safe gap.
        let lane = if now >= self.wave_end {
            let (i, oldest) = ready.min_by_key(order)?;
            if self.blocked_until(move_for(oldest), now).is_some() {
                return None;
            }
            i
        } else {
            ready
                .filter(|&(_, p)| self.blocked_until(move_for(p), now).is_none())
                .min_by_key(order)?
                .0
        };
        let pending = self.lanes[lane].waiting.pop_front().unwrap();
        let movement = move_for(pending);
        if self.occupied.is_empty() {
            self.wave_end = now + WAVE;
        }
        self.occupied.push((movement, now));
        self.lanes[lane].ready = now + HEADWAY;
        Some((pending, self.lanes[lane].previous))
    }

    fn next_wake(&self, now: u32, trips: &[Trip]) -> Option<u32> {
        self.lanes
            .iter()
            .filter_map(|lane| {
                let next = lane.waiting.front()?;
                let ready = now.max(lane.ready);
                let movement = trips[next.trip].approaches[next.stop].movement;
                Some(self.blocked_until(movement, ready).unwrap_or(ready))
            })
            .filter(|&time| time > now)
            .min()
    }
}

struct Pass {
    stop_age: f32,
    arrive: f32,
    release: f32,
    previous: Option<usize>,
    cached: (f32, f32),
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
    group_waits: Vec<Vec<f32>>,
    pending_progress: Vec<usize>,
    now: f32,
    pub(super) longest_wait: f32,
}

impl Queues {
    pub(super) fn rebuild(&mut self, replay: &Replay) {
        self.passes.clear();
        self.by_car.clear();
        self.controls.clear();
        self.group_waits = replay
            .departures
            .iter()
            .map(|groups| vec![0.; groups.len()])
            .collect();
        self.longest_wait = 0.;
        if replay.departures.is_empty() {
            return;
        }
        let started = Instant::now();
        let topology = self.topology.get_or_insert_with(Topology::load);
        let mut nodes = HashMap::<usize, Site>::new();
        let mut seen = HashSet::new();
        let mut routes = Vec::new();
        for flow in replay.departures.iter().flatten() {
            if !matches!(flow.lifecycle, Life::Leave)
                && !flow.cars.is_empty()
                && !flow.path.origin_preview
                && seen.insert(Arc::as_ptr(&flow.path))
            {
                let hits = topology.approaches(&flow.path, replay);
                for &(node, metres) in &hits {
                    register(&flow.path, node, metres, metres, &mut nodes);
                }
                routes.push((&flow.path, hits));
            }
        }
        nodes.retain(|_, site| site.active());
        let mut zones: HashMap<_, _> = nodes.keys().map(|&node| (node, node)).collect();
        // These routes cannot stop again before clearing the first control.
        // Give their connected nodes one admission and cover the complete visit.
        for (_, hits) in &mut routes {
            hits.retain(|(node, _)| nodes.contains_key(node));
            for pair in hits.windows(2) {
                let (first, from) = pair[0];
                let (second, to) = pair[1];
                if ms((to - from) / Vehicle::Car.speed()) < CLEARANCE {
                    let a = zone(&mut zones, first);
                    let b = zone(&mut zones, second);
                    zones.insert(a.max(b), a.min(b));
                }
            }
        }
        let mut sites = HashMap::<usize, Site>::new();
        let mut widest_visit = 0_f32;
        let routes: HashMap<_, Arc<[Approach]>> = routes
            .into_iter()
            .map(|(path, mut hits)| {
                for (node, _) in &mut hits {
                    *node = zone(&mut zones, *node);
                }
                let approaches: Vec<_> = hits
                    .chunk_by(|a, b| {
                        a.0 == b.0 && ms((b.1 - a.1) / Vehicle::Car.speed()) < CLEARANCE
                    })
                    .map(|visit| {
                        let (junction, first) = visit[0];
                        let last = visit.last().unwrap().1;
                        widest_visit = widest_visit.max(last - first);
                        register(path, junction, first, last, &mut sites)
                    })
                    .collect();
                (Arc::as_ptr(path), approaches.into())
            })
            .collect();
        for &node in nodes.keys() {
            let p = topology.nodes[node].1;
            self.controls
                .entry(cell(p, SCALE_RADIUS))
                .or_default()
                .push(p);
        }
        for site in sites.values_mut() {
            site.lanes = (0..site.approaches.len())
                .map(|_| Lane::default())
                .collect();
        }

        let mut trips = Vec::new();
        let mut calendar = Calendar::new();
        for (bin, groups) in replay.departures.iter().enumerate() {
            for (group, flow) in groups.iter().enumerate() {
                if matches!(flow.lifecycle, Life::Leave) {
                    continue;
                }
                let Some(approaches) = routes.get(&Arc::as_ptr(&flow.path)) else {
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
        let mut delays = vec![0_f32; trips.len()];
        let mut worst_wait = (0, 0);
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
                    cached: (f32::NAN, 0.),
                });
                let delay = release - trip.start - approach.stop_age;
                delays[pending.trip] = delay;
                let wait = now - pending.arrive;
                if wait > worst_wait.0 {
                    worst_wait = (wait, junction);
                }
                let (bin, group, _) = trip.car;
                let group_wait = &mut self.group_waits[bin][group];
                *group_wait = group_wait.max(delay);
                self.longest_wait = self.longest_wait.max(delay);
                coverage.0 += usize::from(now > pending.arrive);
                coverage.1 += usize::from(pending.stop > 0);
                if let Some(following) = trip.approaches.get(pending.stop + 1) {
                    let next = now + ms(following.stop_age - approach.stop_age);
                    calendar.push(Reverse((next, false, pending.trip, pending.stop + 1)));
                }
            }
            let next = site.next_wake(now, &trips);
            if next != site.wake || next == Some(now) {
                site.wake = next;
                if let Some(next) = next {
                    calendar.push(Reverse((next, true, junction, 0)));
                }
            }
        }
        delays.sort_unstable_by(f32::total_cmp);
        let percentile = |p: usize| delays.get(delays.len() * p / 100).copied().unwrap_or(0.);
        info!(
            "Road yielding: {} mapped nodes, {} active nodes in {} controls, widest visit {:.1}m, {} cars, {} admissions ({} waits, {} downstream), cumulative delay p95 {:.1}s p99 {:.1}s max {:.1}s; longest stop {:.1}s at node {}; rebuild {:.0}ms",
            topology.nodes.len(),
            nodes.len(),
            sites.len(),
            widest_visit,
            trips.len(),
            self.passes.len(),
            coverage.0,
            coverage.1,
            percentile(95),
            percentile(99),
            self.longest_wait,
            worst_wait.0 as f32 / 1_000.,
            topology.nodes.get(worst_wait.1).map_or(0, |entry| entry.0),
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

    pub(super) fn group_wait(&self, day_bin: i32, group: usize) -> f32 {
        self.group_waits[day_bin.rem_euclid(48) as usize][group]
    }

    pub(super) fn age(&mut self, day_bin: i32, group: usize, copy: usize, age: f32) -> f32 {
        let key = (day_bin.rem_euclid(48) as usize, group, copy);
        let Some(stops) = self.by_car.get(&key) else {
            return age;
        };
        let now = self.now + if day_bin < 0 { DAY } else { 0. };
        let stop = stops
            .partition_point(|&i| self.passes[i].release <= now)
            .min(stops.len() - 1);
        let index = stops[stop];
        let minimum_age = if stop == 0 {
            f32::NEG_INFINITY
        } else {
            self.passes[stops[stop - 1]].stop_age
        };
        let stop_age = self.passes[index].stop_age;
        (stop_age + self.queue_progress(index, now)).max(minimum_age)
    }

    fn queue_progress(&mut self, mut index: usize, now: f32) -> f32 {
        let mut progress = loop {
            let pass = &self.passes[index];
            if pass.cached.0 == now {
                break pass.cached.1;
            }
            let previous = pass.previous.filter(|_| now < pass.release);
            let Some(leader) = previous else {
                break pass.progress(now);
            };
            self.pending_progress.push(index);
            index = leader;
        };
        let gap = HEADWAY as f32 / 1_000.;
        while let Some(index) = self.pending_progress.pop() {
            let pass = &mut self.passes[index];
            progress = pass.progress(now).min(progress - gap);
            pass.cached = (now, progress);
        }
        progress
    }
}

// Follow compressed links to the representative of one compact control.
fn zone(zones: &mut HashMap<usize, usize>, mut node: usize) -> usize {
    while zones[&node] != node {
        let parent = zones[&node];
        let grandparent = zones[&parent];
        zones.insert(node, grandparent);
        node = grandparent;
    }
    node
}

fn ms(seconds: f32) -> u32 {
    (seconds * 1_000.).ceil() as u32
}
