use crate::{
    city::{StadiumSeating, terrain_from_xz},
    movers::{Path, Vehicle},
};
use bevy::prelude::*;
use std::sync::Arc;

pub(super) struct GateWalk {
    path: Arc<Path>,
    length: f32,
    rim_turn: Option<f32>,
}

impl GateWalk {
    /// World-space walking points ending at the access declared by the journey.
    pub(super) fn new(
        mut points: Vec<Vec3>,
        bowl: &StadiumSeating,
        landmark: &Transform,
    ) -> Option<Self> {
        let end = *points.last()?;
        let inverse = landmark.compute_affine().inverse();
        // A stop under the relocated stadium cannot serve a surface arrival.
        if bowl.contains(inverse.transform_point3(points[0])) {
            return None;
        }
        let crossing = points.windows(2).enumerate().find_map(|(i, segment)| {
            let from = inverse.transform_point3(segment[0]);
            let to = inverse.transform_point3(segment[1]);
            let (fraction, turn) = bowl.first_entry(from, to)?;
            Some((i, segment[0].lerp(segment[1], fraction), turn))
        });

        let mut rim_turn = None;
        if let Some((i, point, turn)) = crossing {
            points.truncate(i + 1);
            points.push(point);
            rim_turn = Some(turn);
        } else if let Some((turn, point)) = nearby_rim(end, bowl, landmark) {
            points.push(point);
            rim_turn = Some(turn);
        }

        for point in &mut points {
            point.y = terrain_from_xz(point.x, point.z);
        }
        points.dedup();
        let path = Path::single(points, Vehicle::Walk);
        let length = *path.metres.last()?;
        Some(Self {
            path,
            length,
            rim_turn,
        })
    }

    /// Position and forward heading along the supplied walk, by distance.
    pub(super) fn walk(&self, fraction: f32) -> (Vec3, Vec3) {
        self.path
            .at(self.length * fraction.clamp(0., 1.), &self.path.metres)
            .map(|(point, heading, _)| (point, heading))
            .unwrap_or((self.path.points[0], Vec3::ZERO))
    }

    /// Follow the short arc to an absolute seat turn and depth in the terrace.
    /// An unconnected walk ends at its last point and has no seating segment.
    pub(super) fn seat(
        &self,
        fraction: f32,
        seat_turn: f32,
        row: f32,
        bowl: &StadiumSeating,
        landmark: &Transform,
    ) -> Option<(Vec3, Vec3)> {
        let entry = self.rim_turn?;
        let arc = (seat_turn - entry + 0.5).rem_euclid(1.) - 0.5;
        let at = |t: f32| {
            let depth = 1. + (row.clamp(0., 1.) - 1.) * t;
            let mut point = landmark.transform_point(bowl.terrace(entry + arc * t, depth));
            let ground = terrain_from_xz(point.x, point.z);
            // The schematic ramp starts at walking height and meets the terrace
            // during the first quarter of the seating segment.
            let ramp = (4. * t).min(1.);
            let rise = ramp * ramp * (3. - 2. * ramp);
            point.y = ground + (point.y - ground) * rise;
            point
        };
        let t = fraction.clamp(0., 1.);
        let heading = at((t + 0.001).min(1.)) - at((t - 0.001).max(0.));
        Some((at(t), heading.normalize_or_zero()))
    }
}

fn nearby_rim(end: Vec3, bowl: &StadiumSeating, landmark: &Transform) -> Option<(f32, Vec3)> {
    // stadium_bowl draws 96 straight rim edges. Project on the edges themselves,
    // so the turn and terrace(turn, 1.) describe the same horizontal position.
    const EDGES: usize = 96;
    // Allow a few metre-grid cells of registration error at the supplied endpoint.
    const MAX_JOIN_METRES: f32 = 3.;
    let rim = |i: usize| landmark.transform_point(bowl.terrace(i as f32 / EDGES as f32, 1.));
    let mut from = rim(0);
    let mut closest = None;
    let mut best = MAX_JOIN_METRES.powi(2);
    for i in 0..EDGES {
        let to = rim(i + 1);
        let edge = to.xz() - from.xz();
        let along = ((end.xz() - from.xz()).dot(edge) / edge.length_squared()).clamp(0., 1.);
        let point = from.lerp(to, along);
        let distance = end.xz().distance_squared(point.xz());
        if distance <= best {
            best = distance;
            closest = Some(((i as f32 + along) / EDGES as f32, point));
        }
        from = to;
    }
    closest
}
