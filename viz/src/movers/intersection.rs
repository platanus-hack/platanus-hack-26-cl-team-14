//! Display-only yielding at selected shared road nodes in the UOCT inventory.
use super::{BIN, DAY, Life, Path, Replay, Vehicle, departure_offset, world};
use bevy::prelude::Vec3;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

// City seconds between entries into the junction. At 9 m/s, the preceding
// car has passed the centre by 26 m before the next one reaches its stop line.
const CLEARANCE: f32 = 4.5;
// At the close camera scale this leaves room for the full rendered body.
const HEADWAY: f32 = 1.3;
const STOP_DISTANCE: f32 = 14.;
const QUEUE_RADIUS: f32 = 125.;

// Shared OSM nodes, matched to UOCT inventory locations.
const LOCATIONS: [(f64, f64); 2] = [
    (-70.558968, -33.4521462),  // J071171 · Tobalaba/Larraín
    (-70.6141251, -33.4379143), // J005181 · Antonio Varas/Pocuro
];

type Car = (usize, usize, usize);

struct Trip {
    car: Car,
    start: f32,
    approaches: Arc<[Approach]>,
}

#[derive(Clone, Copy)]
struct Approach {
    junction: usize,
    lane: usize,
    stop_age: f32,
}

struct Pass {
    approach: Approach,
    arrive: f32,
    release: f32,
    ages: [f32; 2],
}

#[derive(Default)]
pub(super) struct Queues {
    junctions: [Vec3; LOCATIONS.len()],
    passes: Vec<Pass>,
    by_car: HashMap<Car, Vec<usize>>,
    now: f32,
    pub(super) longest_wait: f32,
}

impl Queues {
    pub(super) fn rebuild(&mut self, replay: &Replay) {
        self.passes.clear();
        self.by_car.clear();
        self.longest_wait = 0.;
        self.junctions = LOCATIONS.map(|(lon, lat)| world(lon, lat));
        let mut routes = HashMap::<*const Path, Arc<[Approach]>>::new();
        let mut trips = Vec::new();
        let mut due = BinaryHeap::new();
        for (bin, groups) in replay.departures.iter().enumerate() {
            for (group, flow) in groups.iter().enumerate() {
                if matches!(flow.lifecycle, Life::Leave)
                    || flow.cars.is_empty()
                    || flow.path.origin_preview
                {
                    continue;
                }
                let approaches = routes.entry(Arc::as_ptr(&flow.path)).or_insert_with(|| {
                    let mut matches: Vec<_> = self
                        .junctions
                        .iter()
                        .copied()
                        .enumerate()
                        .filter_map(|(id, junction)| approach(&flow.path, id, junction, replay))
                        .collect();
                    matches.sort_by(|a, b| a.stop_age.total_cmp(&b.stop_age));
                    matches.into()
                });
                let Some(first) = approaches.first() else {
                    continue;
                };
                for copy in 0..flow.cars.len() {
                    let start = bin as f32 * BIN + departure_offset(flow.phase, copy);
                    due.push(Reverse((
                        arrival_ms(start + first.stop_age),
                        trips.len(),
                        0,
                    )));
                    trips.push(Trip {
                        car: (bin, group, copy),
                        start,
                        approaches: approaches.clone(),
                    });
                }
            }
        }
        // Schedule FIFO admissions at each site. After a release, propagate
        // its delay to this car's next stop along the original route.
        let mut next = [f32::NEG_INFINITY; LOCATIONS.len()];
        while let Some(Reverse((arrival, index, stop))) = due.pop() {
            let trip = &trips[index];
            let approach = trip.approaches[stop];
            let arrive = arrival as f32 / 1_000.;
            let release = arrive.max(next[approach.junction]);
            next[approach.junction] = release + CLEARANCE;
            self.longest_wait = self
                .longest_wait
                .max(release - trip.start - approach.stop_age);
            self.by_car
                .entry(trip.car)
                .or_default()
                .push(self.passes.len());
            self.passes.push(Pass {
                approach,
                arrive,
                release,
                ages: [0.; 2],
            });
            if let Some(following) = trip.approaches.get(stop + 1) {
                let arrival = release + (following.stop_age - approach.stop_age);
                due.push(Reverse((arrival_ms(arrival), index, stop + 1)));
            }
        }
    }

    pub(super) fn seek(&mut self, now: f32) {
        self.now = now;
        for day in 0..2 {
            let now = now + day as f32 * DAY;
            let mut leader = [[f32::INFINITY; 4]; LOCATIONS.len()];
            for pass in &mut self.passes {
                let free = if now < pass.release {
                    (now - pass.arrive).min(0.)
                } else {
                    now - pass.release
                };
                let last = &mut leader[pass.approach.junction][pass.approach.lane];
                let progress = free.min(*last - HEADWAY);
                *last = progress;
                pass.ages[day] = pass.approach.stop_age + progress;
            }
        }
    }

    // Large map-scale bodies would overlap a close-spaced queue. Keep
    // the local footprint stable and restore map-scale sizing beyond it.
    pub(super) fn car_scale(&self, position: Vec3, scale: f32) -> f32 {
        let distance = self
            .junctions
            .iter()
            .map(|&junction| {
                let offset = position - junction;
                offset.x.hypot(offset.z)
            })
            .fold(f32::INFINITY, f32::min);
        let blend = ((distance - QUEUE_RADIUS) / 30.).clamp(0., 1.);
        1.35 + (scale - 1.35) * blend
    }

    pub(super) fn age(&self, day_bin: i32, group: usize, copy: usize, age: f32) -> f32 {
        let key = (day_bin.rem_euclid(48) as usize, group, copy);
        let Some(stops) = self.by_car.get(&key) else {
            return age;
        };
        let day = usize::from(day_bin < 0);
        let now = self.now + day as f32 * DAY;
        let stop = stops
            .partition_point(|&i| self.passes[i].release <= now)
            .min(stops.len() - 1);
        let age = self.passes[stops[stop]].ages[day];
        if stop == 0 {
            age
        } else {
            // The next approach must never pull a car behind a stop it passed.
            age.max(self.passes[stops[stop - 1]].approach.stop_age)
        }
    }
}

fn arrival_ms(seconds: f32) -> u32 {
    (seconds * 1_000.).ceil() as u32
}

fn approach(path: &Path, id: usize, junction: Vec3, replay: &Replay) -> Option<Approach> {
    // Route packs may simplify a straight crossing into one segment. Match its
    // passage through the shared node, including ample route on both sides.
    let (near, dir, metres) = path.nearest(junction);
    let margin = STOP_DISTANCE + Vehicle::Car.speed() * 6.;
    if near.distance(junction) > 10.
        || metres < margin
        || *path.metres.last()? - metres < margin
        || path.on_bridge(metres)
        || replay.road_edits.hides(near, dir)
    {
        return None;
    }
    let stop = metres - STOP_DISTANCE;
    let (before, _, _) = path.at(stop, &path.metres)?;
    let inbound = near - before;
    let lane = if inbound.x.abs() > inbound.z.abs() {
        usize::from(inbound.x < 0.)
    } else {
        2 + usize::from(inbound.z < 0.)
    };
    Some(Approach {
        junction: id,
        lane,
        stop_age: stop / Vehicle::Car.speed(),
    })
}
