use crate::{
    city::{StadiumSeating, terrain_from_xz},
    movers::{Path, Vehicle},
};
use bevy::prelude::*;
use std::sync::Arc;

pub(super) struct GateWalk {
    path: Arc<Path>,
    length: f32,
}

impl GateWalk {
    /// World-space walking points ending at the access declared by the journey.
    pub(super) fn new(
        mut points: Vec<Vec3>,
        bowl: &StadiumSeating,
        landmark: &Transform,
    ) -> Option<Self> {
        let first = *points.first()?;
        let inverse = landmark.compute_affine().inverse();
        // A stop under the relocated stadium cannot serve a surface arrival.
        if bowl.contains(inverse.transform_point3(first)) {
            return None;
        }
        let crossing = points.windows(2).enumerate().find_map(|(i, segment)| {
            let from = inverse.transform_point3(segment[0]);
            let to = inverse.transform_point3(segment[1]);
            bowl.first_entry(from, to)
                .map(|fraction| (i, segment[0].lerp(segment[1], fraction)))
        });
        if let Some((i, point)) = crossing {
            points.truncate(i + 1);
            points.push(point);
        }
        for point in &mut points {
            point.y = terrain_from_xz(point.x, point.z);
        }
        points.dedup();
        let path = Path::single(points, Vehicle::Walk);
        let length = *path.metres.last()?;
        Some(Self { path, length })
    }

    /// Position and forward heading along the supplied walk, by distance.
    pub(super) fn walk(&self, fraction: f32) -> (Vec3, Vec3) {
        self.path
            .at(self.length * fraction.clamp(0., 1.), &self.path.metres)
            .map(|(point, heading, _)| (point, heading))
            .unwrap_or((self.path.points[0], Vec3::ZERO))
    }
}
