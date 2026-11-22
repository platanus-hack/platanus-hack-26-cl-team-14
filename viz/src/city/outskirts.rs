//! The survey ends at a rectangle. Mirrored copies of the terrain continue the land
//! past each side, and the print pass dissolves them into the sky (`pixels/vote.wgsl`).
use bevy::{
    camera::primitives::Aabb,
    light::NotShadowCaster,
    prelude::*,
};

/// Metres past the surveyed edge at which the mirrored land has become sky.
pub(crate) const REACH: f32 = 30_000.;

pub(super) fn spawn(
    commands: &mut Commands,
    parent: Entity,
    land: Rect,
    bounds: Aabb,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
) {
    // A mirror about an edge keeps the ground continuous there. Scale and offset per axis.
    let mirrors = |low: f32, high: f32| [(1., 0.), (-1., 2. * low), (-1., 2. * high)];
    for (sx, tx) in mirrors(land.min.x, land.max.x) {
        for (sz, tz) in mirrors(land.min.y, land.max.y) {
            let center = Vec2::new(tx + sx * bounds.center.x, tz + sz * bounds.center.z);
            let half = Vec2::new(bounds.half_extents.x, bounds.half_extents.z);
            let gap = (land.min - center - half).max(center - half - land.max).max(Vec2::ZERO);
            if (sx > 0. && sz > 0.) || gap.length() > REACH {
                continue;
            }
            commands.spawn((
                ChildOf(parent),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(tx, 0., tz).with_scale(Vec3::new(sx, 1., sz)),
                NotShadowCaster,
            ));
        }
    }
}
