//! Passenger marks on scheduled buses. Each mark uses one existing replay rider.
use super::*;

type Departure = (usize, i32);

struct OnBoard {
    // Bin start relative to the day of the service, including riders from the previous day.
    bin: i32,
    offset: f32,
    board_age: f32,
    alight_age: f32,
}

#[derive(Default)]
pub(crate) struct BaseBusLoads(HashMap<Departure, Vec<OnBoard>>);

impl BaseBusLoads {
    pub(crate) fn new(rows: &[Value], routes: &Routes) -> Self {
        let mut runs = HashMap::<Departure, Vec<OnBoard>>::new();
        for row in rows.iter().filter(|row| row["mode"] == "public") {
            let Some(path) = routes.group_path(row) else {
                continue;
            };
            let id = row["id"].as_u64().unwrap() as u32;
            let seed = id.wrapping_mul(2654435761) as f64 / 4294967296.;
            let copies =
                (row["count"].as_f64().unwrap() / PERSONS_PER_MOVER + seed).floor() as usize;
            let bin = row["half_hour"].as_i64().unwrap() as i32;
            for copy in 0..copies {
                let offset = departure_offset(seed as f32, copy);
                let start = bin as f32 * BIN + offset;
                let plan = RiderPlan::build(&path, path.legs.len(), start, copy, routes);
                let mut board_age = 0.;
                for step in plan.steps {
                    if let RiderAction::Ride(RideKey::Surface { template, start }) = step.action {
                        if routes.riders.surface[template].vehicle == Vehicle::Bus {
                            let day = start.div_euclid(DAY as i32);
                            runs.entry((template, start.rem_euclid(DAY as i32)))
                                .or_default()
                                .push(OnBoard {
                                    bin: bin - 48 * day,
                                    offset,
                                    board_age,
                                    alight_age: step.end_age,
                                });
                        }
                    }
                    board_age = step.end_age;
                }
            }
        }
        Self(runs)
    }

    pub(crate) fn count(&self, template: usize, start: i32, now: f32) -> usize {
        let Some(intervals) = self.0.get(&(template, start.rem_euclid(DAY as i32))) else {
            return 0;
        };
        let day_bins = 48 * start.div_euclid(DAY as i32);
        intervals
            .iter()
            .filter(|rider| {
                let age = (now - (rider.bin + day_bins) as f32 * BIN) - rider.offset;
                (rider.board_age..rider.alight_age).contains(&age)
            })
            .count()
    }
}

pub(crate) fn interior_mesh() -> Mesh {
    block([7.6, 0.05, 2.0], [0.; 3], PAPER)
}

pub(crate) fn passenger_mesh() -> Mesh {
    // Shoulders and head in plan view through the roof cutaway.
    shape([
        oval([0.30, 0.08, 0.40], [0., 0.09, 0.], [255; 3]),
        oval([0.21, 0.16, 0.21], [0.10, 0.20, 0.], [255; 3]),
    ])
}

pub(crate) fn draw(
    count: usize,
    base: usize,
    front: Vec3,
    dir: Vec3,
    scale: f32,
    place: &mut impl FnMut(Model, Paint, Transform),
) {
    let roof = front + Vec3::Y * (27. * ART_PIXEL + 0.02) * scale;
    place(
        Model::BusInterior,
        Paint::Plain,
        pose(roof, dir, Vec3::splat(scale)),
    );
    // The cutaway fits the assigned riders; its area does not imply vehicle capacity.
    let columns = count.div_ceil(2).max(5);
    let step = 7.2 / columns as f32;
    let breadth = right(dir);
    for index in 0..count {
        let x = (index / 2) as f32 * step - 3.6 + step / 2.;
        let z = if index % 2 == 0 { -0.46 } else { 0.46 };
        let at = roof + (dir * x + breadth * z) * scale;
        let tint = if index < base {
            Paint::Plain
        } else {
            Paint::Gain
        };
        place(
            Model::BusPassenger,
            tint,
            pose(at, dir, Vec3::new(step.min(1.), 1., 1.) * scale),
        );
    }
}
