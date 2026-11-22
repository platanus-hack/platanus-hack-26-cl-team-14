//! A turn radius for the drawing. GTFS supplies the centreline and timetable, not steering geometry.
use super::{BUS_FRONT_TO_HINGE, BUS_TAIL_HALF, Call, Path, RailRun, Vec2, Vec3, terrain_from_xz};

const RADIUS: f32 = 12.;

struct Vertex {
    point: Vec2,
    gtfs: f32,
}

struct Segment {
    from: Vec2,
    to: Vec2,
    // Centre and signed sweep for a tangent circular arc; straight otherwise.
    arc: Option<(Vec2, f32)>,
    end: f32,
    gtfs_end: f32,
}

pub(super) struct BusCurve {
    segments: Vec<Segment>,
    run: RailRun,
}

fn bend(nodes: &[Vertex], i: usize) -> (f32, f32) {
    if i == 0 || i + 1 == nodes.len() {
        return (0., 0.);
    }
    let enter = nodes[i].point - nodes[i - 1].point;
    let leave = nodes[i + 1].point - nodes[i].point;
    let sweep = enter.perp_dot(leave).atan2(enter.dot(leave));
    (sweep, RADIUS * (0.5 * sweep.abs()).tan())
}

impl BusCurve {
    pub(super) fn new(path: &Path, schedule: &RailRun) -> Self {
        let mut nodes: Vec<_> = path
            .points
            .iter()
            .zip(&path.metres)
            .map(|(p, &gtfs)| Vertex {
                point: Vec2::new(p.x, p.z),
                gtfs,
            })
            .collect();
        nodes.dedup_by(|a, b| a.point == b.point);
        // Two tangent arcs can overlap on a short GTFS segment. Absorb the smaller
        // change of direction into the stronger corner, then recompute both tangents.
        let bends = loop {
            let bends: Vec<_> = (0..nodes.len()).map(|i| bend(&nodes, i)).collect();
            let conflict = nodes.windows(2).enumerate().position(|(i, pair)| {
                bends[i].1 + bends[i + 1].1 > pair[0].point.distance(pair[1].point)
            });
            let Some(i) = conflict else { break bends };
            let remove = if i == 0 || (i + 2 < nodes.len() && bends[i + 1].1 < bends[i].1) {
                i + 1
            } else {
                i
            };
            nodes.remove(remove);
        };
        let mut segments = Vec::new();
        let mut from = nodes[0].point;
        let mut end = 0.;
        let mut push = |to: Vec2, arc: Option<(Vec2, f32)>, gtfs_end: f32| {
            let length = arc.map_or_else(|| from.distance(to), |(_, sweep)| RADIUS * sweep.abs());
            if length > 0. {
                end += length;
                segments.push(Segment {
                    from,
                    to,
                    arc,
                    end,
                    gtfs_end,
                });
                from = to;
            }
        };
        for i in 1..nodes.len() - 1 {
            let (sweep, trim) = bends[i];
            if trim < 0.001 {
                push(nodes[i].point, None, nodes[i].gtfs);
                continue;
            }
            let prev = &nodes[i - 1];
            let corner = &nodes[i];
            let next = &nodes[i + 1];
            let incoming = corner.point - prev.point;
            let outgoing = next.point - corner.point;
            let entry_frac = trim / incoming.length();
            let exit_frac = trim / outgoing.length();
            let entry = corner.point - incoming * entry_frac;
            let exit = corner.point + outgoing * exit_frac;
            let centre = entry + incoming.normalize().perp() * (RADIUS * sweep.signum());
            push(
                entry,
                None,
                corner.gtfs - (corner.gtfs - prev.gtfs) * entry_frac,
            );
            push(
                exit,
                Some((centre, sweep)),
                corner.gtfs + (next.gtfs - corner.gtfs) * exit_frac,
            );
        }
        let last = nodes.last().unwrap();
        push(last.point, None, last.gtfs);
        // Convert each timetable stop once. The existing dwell and acceleration law
        // then runs on the real length of the drawn curve, between the same times.
        let calls = schedule
            .calls
            .iter()
            .map(|c| {
                let i = segments
                    .partition_point(|s| s.gtfs_end < c.distance)
                    .min(segments.len() - 1);
                let s = &segments[i];
                let (start, gtfs_start) = i
                    .checked_sub(1)
                    .map_or((0., 0.), |j| (segments[j].end, segments[j].gtfs_end));
                let fraction = (c.distance - gtfs_start) / (s.gtfs_end - gtfs_start);
                Call {
                    distance: start + fraction * (s.end - start),
                    station: c.station,
                    arrival: c.arrival,
                    departure: c.departure,
                }
            })
            .collect();
        Self {
            segments,
            run: RailRun { calls },
        }
    }

    fn point(&self, distance: f32) -> Vec2 {
        let i = self
            .segments
            .partition_point(|s| s.end < distance)
            .min(self.segments.len() - 1);
        let s = &self.segments[i];
        let start = i.checked_sub(1).map_or(0., |j| self.segments[j].end);
        let fraction = (distance - start) / (s.end - start);
        match s.arc {
            Some((centre, sweep)) => {
                centre + Vec2::from_angle(sweep * fraction).rotate(s.from - centre)
            }
            // Linear extrapolation at the terminal puts the whole rigid bus on its approach.
            None => s.from.lerp(s.to, fraction),
        }
    }

    pub(super) fn pose(&self, age: f32) -> Option<(Vec3, Vec3, Vec3)> {
        let d = self.run.distance(age)?;
        let hinge = self.point(d - BUS_FRONT_TO_HINGE);
        // Orient the front along the chord to its leading axle (3 m ahead of the
        // body centre), and the tail along the chord from its rear end to the joint.
        let front_dir = (self.point(d + 3.) - hinge).normalize();
        let rear_dir =
            (hinge - self.point(d - BUS_FRONT_TO_HINGE - 2. * BUS_TAIL_HALF)).normalize();
        let front = hinge + front_dir * BUS_FRONT_TO_HINGE;
        let horizontal = |v: Vec2| Vec3::new(v.x, 0., v.y);
        Some((
            Vec3::new(front.x, terrain_from_xz(front.x, front.y), front.y),
            horizontal(front_dir),
            horizontal(rear_dir),
        ))
    }
}
