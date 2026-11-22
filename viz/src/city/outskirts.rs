// A wash of paper over the surveyed boundary. Its width varies along each
// side, while the innermost edge stays clear of the central street map.
use super::{Surface, terrain};
use bevy::prelude::*;

pub(super) fn wash() -> Surface {
    let bounds = terrain::extent();
    let half = bounds.half_size();
    let center = bounds.center();
    let corners = [
        Vec2::new(-1., -1.),
        Vec2::new(1., -1.),
        Vec2::ONE,
        Vec2::new(-1., 1.),
    ];
    let mut surface = Surface::default();
    let steps = 72;
    // The wash reaches full opacity before the DEM stops. Outside it, the
    // matching sky is already visible and the veil can clear over the Andes.
    let rings = [
        (-1., 0.),
        (-0.66, 0.12),
        (-0.35, 0.63),
        (-0.10, 1.),
        (0.12, 1.),
        (0.55, 0.),
    ];
    for (ring, (inset, opacity)) in rings.into_iter().enumerate() {
        for side in 0..4 {
            for i in 0..steps {
                let unit = corners[side].lerp(corners[(side + 1) % 4], i as f32 / steps as f32);
                let edge = center + half * unit;
                let width = 3_850.
                    + 850. * (edge.x * 0.00048 + edge.y * 0.00068).sin()
                    + 570. * (edge.x * 0.00133 - edge.y * 0.00116).cos();
                let p = edge + unit * width * inset;
                let height = terrain::terrain_from_xz(p.x, p.y) + 170.;
                surface.vertex(Vec3::new(p.x, height, p.y), Vec3::Y, [1., 1., 1., opacity]);
                if ring > 0 {
                    let stride = 4 * steps;
                    let a = (ring * stride + side * steps + i) as u32;
                    let b = (ring * stride + (side * steps + i + 1) % stride) as u32;
                    surface.indices.extend_from_slice(&[
                        a,
                        b,
                        a - stride as u32,
                        b,
                        b - stride as u32,
                        a - stride as u32,
                    ]);
                }
            }
        }
    }
    surface
}
