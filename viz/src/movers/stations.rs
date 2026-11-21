use super::*;

pub(super) struct Station {
    entrance: Vec3,
    sides: Vec<Side>,
    entries: Vec<Entry>,
    longest_visit: f32,
}

struct Side {
    base: Vec3,
    dir: Vec3,
}

struct Entry {
    start: f32,
    ready: f32,
    departure: f32,
    side: usize,
    wait_at: Vec3,
    paint: Paint,
}

impl Station {
    pub(super) fn new(row: &Value, tracks: &[RailLine]) -> Self {
        let id = row["id"].as_str().unwrap();
        let entrance = world(
            row["longitude"].as_f64().unwrap(),
            row["latitude"].as_f64().unwrap(),
        );
        let mut sides = Vec::new();
        let mut timetables = Vec::new();
        for line in tracks {
            let Some(i) = line.stations.iter().position(|s| s == id) else {
                continue;
            };
            let stop = &line.stops[i];
            if stop.departures.is_empty() {
                continue;
            }
            let (rail, dir) = line.coach(stop.distance);
            sides.push(Side {
                base: rail + right(dir) * 4.9,
                dir,
            });
            timetables.push(&stop.departures);
        }

        // The engine supplies entries per half hour. Space the represented arrivals over that
        // interval. Without a passenger destination, show the first departing service they can reach.
        let mut entries = Vec::new();
        let mut cumulative = 0.;
        let mut next = PERSONS_PER_MOVER * 0.5;
        let mut longest_visit = 0.0_f32;
        for (bin, volume) in row["scenario_half_hour"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            let people = volume.as_f64().unwrap();
            let end = cumulative + people;
            while next <= end {
                let start = (bin as f32 + ((next - cumulative) / people) as f32) * BIN;
                let n = (next / PERSONS_PER_MOVER) as usize;
                let along = ((n * 17) % 21) as f32 * 2.2 - 22.;
                let across = ((n * 5) % 3) as f32 * 0.8 - 0.8;
                let board = sides
                    .iter()
                    .enumerate()
                    .filter_map(|(i, side)| {
                        let wait_at = side.base
                            + side.dir * along
                            + right(side.dir) * across
                            + Vec3::Y * 0.58;
                        let ready = start + entrance.distance(wait_at) / Vehicle::Walk.speed();
                        let times = timetables[i];
                        let departure = *times.get(times.partition_point(|&t| t < ready))?;
                        Some((i, ready, departure, wait_at))
                    })
                    .min_by(|a, b| a.2.total_cmp(&b.2));
                if let Some((side, ready, departure, wait_at)) = board {
                    let delta = row["delta"][bin].as_f64().unwrap_or(0.);
                    let paint = if delta > 0. {
                        Paint::Gain
                    } else if delta < 0. {
                        Paint::Loss
                    } else {
                        Paint::Plain
                    };
                    longest_visit = longest_visit.max(departure - start);
                    entries.push(Entry {
                        start,
                        ready,
                        departure,
                        side,
                        wait_at,
                        paint,
                    });
                }
                next += PERSONS_PER_MOVER;
            }
            cumulative = end;
        }
        Self {
            entrance,
            sides,
            entries,
            longest_visit,
        }
    }

    pub(super) fn draw(
        &self,
        now: f32,
        subway: bool,
        visible: &impl Fn(Vec3, f32) -> bool,
        place: &mut impl FnMut(Model, Paint, Transform),
    ) {
        if !visible(self.entrance, 180.) {
            return;
        }
        let dir = self.sides.first().map_or(Vec3::X, |s| s.dir);
        place(
            Model::Entrance,
            Paint::Plain,
            pose(self.entrance, dir, Vec3::ONE),
        );
        for side in &self.sides {
            if !subway && side.base.y < terrain_from_xz(side.base.x, side.base.z) - 0.6 {
                continue;
            }
            place(
                Model::Platform,
                Paint::Plain,
                pose(side.base, side.dir, Vec3::new(58., 1., 4.2)),
            );
        }
        let entered = self.entries.partition_point(|e| e.start <= now);
        let first = self
            .entries
            .partition_point(|e| e.start < now - self.longest_visit);
        for e in &self.entries[first..entered] {
            if now >= e.departure {
                continue;
            }
            let side = &self.sides[e.side];
            let (p, dir) = if now < e.ready {
                let progress = (now - e.start) / (e.ready - e.start);
                (
                    self.entrance.lerp(e.wait_at, progress),
                    e.wait_at - self.entrance,
                )
            } else {
                let inward = -right(side.dir);
                let boarding_time = (e.departure - e.ready).min(2.);
                let progress =
                    ((now - e.departure + boarding_time) / boarding_time.max(0.001)).clamp(0., 1.);
                (e.wait_at + inward * 2.2 * progress, inward)
            };
            if !subway && p.y < terrain_from_xz(p.x, p.z) - 0.6 {
                continue;
            }
            place(Model::Person, e.paint, pose(p, dir, Vec3::splat(1.2)));
        }
    }
}
