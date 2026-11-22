//! Display-only yielding at UOCT junction J071171 (Tobalaba/Larraín).
use super::{BIN, DAY, Life, Path, Replay, Vehicle, departure_offset, world};
use bevy::prelude::Vec3;
use std::{collections::HashMap, sync::Arc};

// City seconds between entries into the junction. At 9 m/s, the preceding
// car has passed the centre by 26 m before the next one reaches its stop line.
const CLEARANCE: f32 = 4.5;
// At the close camera scale this leaves room for the full rendered body.
const HEADWAY: f32 = 1.3;
const STOP_DISTANCE: f32 = 14.;
const QUEUE_RADIUS: f32 = 80.;

type Car = (usize, usize, usize);

#[derive(Clone, Copy)]
struct Approach {
    lane: usize,
    stop_age: f32,
}

struct Pass {
    car: Car,
    approach: Approach,
    arrive: f32,
    release: f32,
    ages: [f32; 2],
}

#[derive(Default)]
pub(super) struct Queues {
    junction: Vec3,
    passes: Vec<Pass>,
    by_car: HashMap<Car, usize>,
    pub(super) longest_wait: f32,
}

impl Queues {
    pub(super) fn rebuild(&mut self, replay: &Replay) {
        self.passes.clear();
        self.by_car.clear();
        self.longest_wait = 0.;
        // OSM node 256366826 is the shared node on the two routed streets.
        self.junction = world(-70.558968, -33.4521462);
        let mut routes = HashMap::new();
        for (bin, groups) in replay.departures.iter().enumerate() {
            for (group, flow) in groups.iter().enumerate() {
                if matches!(flow.lifecycle, Life::Leave)
                    || flow.cars.is_empty()
                    || flow.path.origin_preview
                {
                    continue;
                }
                let approach = routes
                    .entry(Arc::as_ptr(&flow.path))
                    .or_insert_with(|| approach(&flow.path, self.junction, replay));
                let Some(approach) = *approach else {
                    continue;
                };
                for copy in 0..flow.cars.len() {
                    let arrive =
                        bin as f32 * BIN + departure_offset(flow.phase, copy) + approach.stop_age;
                    self.passes.push(Pass {
                        car: (bin, group, copy),
                        approach,
                        arrive,
                        release: 0.,
                        ages: [0.; 2],
                    });
                }
            }
        }
        // FIFO reservations cannot starve one approach. Slots are tied to
        // the replay clock and remain the same when the viewer seeks or pans.
        self.passes.sort_by(|a, b| a.arrive.total_cmp(&b.arrive));
        let mut next = f32::NEG_INFINITY;
        for (index, pass) in self.passes.iter_mut().enumerate() {
            pass.release = pass.arrive.max(next);
            next = pass.release + CLEARANCE;
            self.longest_wait = self.longest_wait.max(pass.release - pass.arrive);
            self.by_car.insert(pass.car, index);
        }
    }

    pub(super) fn seek(&mut self, now: f32) {
        for day in 0..2 {
            let now = now + day as f32 * DAY;
            let mut leader = [f32::INFINITY; 4];
            for pass in &mut self.passes {
                let free = if now < pass.release {
                    (now - pass.arrive).min(0.)
                } else {
                    now - pass.release
                };
                let lane = pass.approach.lane;
                let progress = free.min(leader[lane] - HEADWAY);
                leader[lane] = progress;
                pass.ages[day] = pass.approach.stop_age + progress;
            }
        }
    }

    // Large map-scale bodies would overlap a close-spaced queue. Keep
    // the local footprint stable and restore map-scale sizing beyond it.
    pub(super) fn car_scale(&self, position: Vec3, scale: f32) -> f32 {
        let offset = position - self.junction;
        let distance = offset.x.hypot(offset.z);
        let blend = ((distance - QUEUE_RADIUS) / 30.).clamp(0., 1.);
        1.35 + (scale - 1.35) * blend
    }

    pub(super) fn age(&self, day_bin: i32, group: usize, copy: usize, age: f32) -> f32 {
        let key = (day_bin.rem_euclid(48) as usize, group, copy);
        self.by_car.get(&key).map_or(age, |&index| {
            self.passes[index].ages[usize::from(day_bin < 0)]
        })
    }
}

fn approach(path: &Path, junction: Vec3, replay: &Replay) -> Option<Approach> {
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
        lane,
        stop_age: stop / Vehicle::Car.speed(),
    })
}
