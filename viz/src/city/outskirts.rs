// A wash of paper over the surveyed boundary. Its width varies along each
// side, while the innermost edge stays clear of the central street map.
use super::{Surface, terrain};
use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

#[derive(Component)]
pub(crate) struct PaperWash;

pub(super) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
) {
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
    commands.spawn((
        Name::new("Basin edge haze"),
        PaperWash,
        ChildOf(root),
        NotShadowCaster,
        NotShadowReceiver,
        Mesh3d(meshes.add(surface.finish())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: crate::DAY_SKY,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        })),
    ));
}

pub(crate) fn fade(
    rig: Res<crate::Rig>,
    sky: Res<ClearColor>,
    mut wash: Single<(&MeshMaterial3d<StandardMaterial>, &mut Visibility), With<PaperWash>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !rig.is_changed() && !sky.is_changed() {
        return;
    }
    let opacity = ((rig.distance - 8_000.) / 10_000.).clamp(0., 1.);
    *wash.1 = if opacity > 0. {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let mut material = materials.get_mut(&wash.0.0).unwrap();
    material.base_color = sky.0.with_alpha(opacity);
}
