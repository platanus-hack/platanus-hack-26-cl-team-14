//! A painted eastern skyline for the title. The mapped DEM ends at X = 18,583 m.
//! Tonal separation follows aerial perspective: distant planes approach the sky.
//! https://www.nationalgallery.org.uk/paintings/glossary/aerial-perspective
use super::Surface;
use crate::boot::TitleLight;
use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

pub(crate) struct HorizonPlugin;

impl Plugin for HorizonPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var_os("SANTIAGO_BOOT_SIGNAL").is_some() {
            app.add_systems(Startup, setup).add_systems(
                Update,
                tint.after(super::paper_light)
                    .run_if(resource_changed::<TitleLight>),
            );
        }
    }
}

#[derive(Component)]
enum Pigment {
    Ink { day: Srgba, night: Srgba },
    Paper,
}

impl Pigment {
    fn ink(day: [u8; 3], night: [u8; 3]) -> Self {
        Self::Ink {
            day: Srgba::rgb_u8(day[0], day[1], day[2]),
            night: Srgba::rgb_u8(night[0], night[1], night[2]),
        }
    }

    fn color(&self, darkness: f32, sky: Color) -> Color {
        match self {
            Self::Ink { day, night } => day.mix(night, darkness).into(),
            Self::Paper => sky,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Entity origins are on separate eastings so Bevy sorts the transparent planes
    // from the far peaks to the foothills. The city depth buffer still occludes them.
    let mut panel = |name, east, surface: Surface, pigment: Pigment| {
        let material = StandardMaterial {
            base_color: pigment.color(0., crate::DAY_SKY),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        };
        commands.spawn((
            Name::new(name),
            pigment,
            Mesh3d(meshes.add(surface.finish())),
            MeshMaterial3d(materials.add(material)),
            Transform::from_xyz(east, 0., 0.),
            NotShadowCaster,
            NotShadowReceiver,
        ));
    };

    panel(
        "Andes · high summits",
        45_000.,
        ridge(2),
        Pigment::ink([189, 196, 211], [77, 83, 108]),
    );
    panel(
        "Andes · high cloud",
        40_000.,
        clouds(&[
            (-36., 13., 7.1, 0.35),
            (-9., 8., 6.9, 0.16),
            (9., 15., 7.5, 0.26),
            (32., 9., 6.7, 0.22),
        ]),
        Pigment::ink([247, 245, 238], [121, 128, 156]),
    );
    panel(
        "Andes · middle ridge",
        31_000.,
        ridge(1),
        Pigment::ink([158, 168, 192], [54, 63, 85]),
    );
    panel(
        "Andes · valley mist",
        25_000.,
        clouds(&[
            (-27., 15., 2.8, 0.48),
            (-5., 12., 2.1, 0.31),
            (11., 7., 3.0, 0.22),
            (29., 13., 2.5, 0.38),
        ]),
        Pigment::ink([230, 231, 232], [95, 105, 130]),
    );
    panel(
        "Andes · foothills",
        21_500.,
        ridge(0),
        Pigment::ink([127, 143, 162], [39, 47, 66]),
    );
    panel(
        "Basin edge haze",
        0.,
        super::outskirts::wash(),
        Pigment::Paper,
    );
}

// Unequal saddles and summits form a broken skyline along the north–south axis.
// They are drawing coordinates, independent of named peaks or measured elevations.
const PEAKS: &[(f32, f32)] = &[
    (-100., 0.22),
    (-59., 0.42),
    (-47., 0.21),
    (-41., 0.68),
    (-35., 0.34),
    (-28., 0.80),
    (-24., 0.60),
    (-16., 0.32),
    (-12., 0.72),
    (-8., 0.57),
    (-3., 0.98),
    (0., 0.66),
    (6., 0.24),
    (10., 0.40),
    (17., 0.81),
    (21., 0.45),
    (29., 0.62),
    (36., 0.28),
    (43., 0.68),
    (51., 0.43),
    (64., 0.68),
    (100., 0.27),
];

fn skyline(z: f32, layer: usize) -> f32 {
    let t = [z * 0.88 + 11., -z * 1.08 - 5., z][layer];
    let i = PEAKS
        .partition_point(|&(p, _)| p < t)
        .clamp(1, PEAKS.len() - 1);
    let (a, b) = (PEAKS[i - 1], PEAKS[i]);
    let shape = a.1 + (b.1 - a.1) * ((t - a.0) / (b.0 - a.0)).clamp(0., 1.);
    let crags = (t * 1.26).sin().abs() * 0.08 + (t * 3.7 + 1.7).sin() * 0.026;
    [820., 1_580., 2_550.][layer] + [2_250., 3_310., 4_000.][layer] * (shape + crags)
}

fn add_vertex(mesh: &mut Surface, p: Vec3, tone: f32, alpha: f32) {
    mesh.vertex(p, Vec3::Y, [tone, tone, tone, alpha]);
}

fn ridge(layer: usize) -> Surface {
    let mut mesh = Surface::default();
    let bounds = super::terrain::extent();
    for i in 0..=320 {
        let z = -80. + i as f32 * 0.5; // kilometres along the range
        let top = skyline(z, layer);
        let slope = (skyline(z + 0.5, layer) - skyline(z - 0.5, layer)) / 1000.;
        let shade = (0.80 + 0.21 * slope).clamp(0.61, 0.98);
        let spur = (z * 0.49 + layer as f32 * 1.7).sin();
        let x = 610. * (z * 0.13).sin() + 340. * spur;
        let base = mesh.positions.len() as u32;
        // Beyond the mapped valley, let the foothills taper into the paper.
        let z_m = z * 1000.;
        let spread = if layer == 0 {
            let t = ((bounds.min.y - z_m).max(z_m - bounds.max.y) / 8000.).clamp(0., 1.);
            t * t * (3. - 2. * t) * (10_800. + 1_600. * spur)
        } else {
            0.
        };
        add_vertex(&mut mesh, Vec3::new(-2900. - spread, -500., z_m), 0.92, 0.);
        add_vertex(
            &mut mesh,
            Vec3::new(x - 1200., top * (0.46 + 0.09 * spur), z * 1000.),
            shade,
            1.,
        );
        // A small broken snow line on high summits. Lower crests retain their ink.
        let snow = if layer == 2 {
            ((top - 4550.) / 1700.).clamp(0., 1.)
        } else {
            0.
        };
        add_vertex(
            &mut mesh,
            Vec3::new(x - 40., top * (0.96 - 0.12 * snow), z * 1000.),
            shade,
            1.,
        );
        add_vertex(
            &mut mesh,
            Vec3::new(x, top, z * 1000.),
            1. + 0.48 * snow,
            1.,
        );
        if i > 0 {
            for row in 0..3 {
                let left = base - 4 + row;
                let right = base + row;
                mesh.indices
                    .extend([left, right, right + 1, left, right + 1, left + 1]);
            }
        }
    }
    mesh
}

// Each wisp tapers to zero alpha along all four edges; one draw per cloud layer.
fn clouds(wisps: &[(f32, f32, f32, f32)]) -> Surface {
    let mut mesh = Surface::default();
    for &(center, width, height, thickness) in wisps {
        for i in 0..=40 {
            let t = i as f32 / 40.;
            let taper = (std::f32::consts::PI * t).sin().max(0.);
            let z = center + width * (2. * t - 1.);
            let mid = height + 0.12 * (z * 0.43).sin() + 0.09 * (z * 0.67).cos();
            let radius = thickness * taper * (1. + 0.14 * (z * 1.18).sin());
            let base = mesh.positions.len() as u32;
            for (offset, opacity) in [(-1., 0.), (-0.2, 0.32), (0.24, 0.43), (1., 0.)] {
                add_vertex(
                    &mut mesh,
                    Vec3::new(0., 1000. * (mid + radius * offset), 1000. * z),
                    1.,
                    opacity * taper,
                );
            }
            if i > 0 {
                for row in 0..3 {
                    let a = base - 4 + row;
                    let b = base + row;
                    mesh.indices.extend([a, b, b + 1, a, b + 1, a + 1]);
                }
            }
        }
    }
    mesh
}

fn tint(
    mut commands: Commands,
    light: Res<TitleLight>,
    rig: Res<crate::Rig>,
    sky: Res<ClearColor>,
    layers: Query<(Entity, &Pigment, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(night) = light.0 else {
        for (entity, _, _) in &layers {
            commands.entity(entity).despawn();
        }
        return;
    };
    let opacity = ((rig.distance - 8_000.) / 10_000.).clamp(0., 1.);
    for (_, pigment, handle) in &layers {
        materials.get_mut(handle).unwrap().base_color =
            pigment.color(night, sky.0).with_alpha(opacity);
    }
}
