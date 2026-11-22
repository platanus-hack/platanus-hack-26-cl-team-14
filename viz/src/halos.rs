//! Soft ink glow on the ground where people gather, and dashed arrows on the fans' last walk to the gate.
use crate::{
    Rig,
    city::{SubwayView, terrain_from_xz, world},
    net::Scenario,
    movers::{
        clock::CityClock,
        fan_travel::{FanJourneys, FanMode, RefreshFanJourneys},
    },
};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use std::collections::HashMap;

const CELL: f32 = 24.;
const HALO: Color = Color::srgb_u8(126, 98, 186);
const FLOW: Color = Color::srgb_u8(88, 72, 150);
const DASH: f32 = 12.;
const GAP: f32 = 8.;
const WIDTH: f32 = 3.;
// At pitch playback speed the fans' own walk lasts seconds; keep the arrow up around it.
const LEAD: f32 = 1_800.;

pub(crate) struct HalosPlugin;

impl Plugin for HalosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                lay_flows
                    .after(RefreshFanJourneys)
                    .after(crate::city::draw_buildings)
                    .run_if(resource_changed::<FanJourneys>),
                (show_flows, glow).after(crate::movers::clock::UpdateClock),
            )
                .chain(),
        );
    }
}

#[derive(Resource)]
struct Art {
    quad: Handle<Mesh>,
    head: Handle<Mesh>,
    halo: Handle<StandardMaterial>,
    flow: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Halo;

/// One dash or head of an arrow, shown while any fan on its path is walking to the gate.
#[derive(Component)]
struct Flow {
    from: f32,
    to: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // White texel with a smooth radial falloff in alpha; the material tints it.
    let n = 64;
    let mut data = Vec::with_capacity(n * n * 4);
    for y in 0..n {
        for x in 0..n {
            let d = Vec2::new(x as f32, y as f32) + 0.5 - n as f32 / 2.;
            let r = (d.length() / (n as f32 / 2.)).min(1.);
            let a = (1. - r * r).powf(1.5);
            data.extend([255, 255, 255, (a * 255.) as u8]);
        }
    }
    let texture = images.add(Image::new(
        Extent3d { width: n as u32, height: n as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ));
    let flat = |color: Color, texture| StandardMaterial {
        base_color: color,
        base_color_texture: texture,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        depth_bias: 50.,
        cull_mode: None,
        ..default()
    };
    commands.insert_resource(Art {
        quad: meshes.add(Plane3d::default().mesh().size(1., 1.)),
        head: meshes.add(
            Mesh::from(Triangle2d::new(Vec2::new(1., 0.), Vec2::new(-1., 0.8), Vec2::new(-1., -0.8)))
                .rotated_by(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ),
        halo: materials.add(flat(HALO.with_alpha(0.8), Some(texture))),
        flow: materials.add(flat(FLOW, None)),
    });
}

fn glow(
    art: Res<Art>,
    subway: Res<SubwayView>,
    people: Query<(&Name, &Transform, &Visibility), Without<Halo>>,
    mut halos: Query<(&mut Transform, &mut Visibility), With<Halo>>,
    mut pool: Local<Vec<Entity>>,
    mut commands: Commands,
) {
    // shortcut: keys on the stadium fans' names, switch to Maqui's `Silhouette` marker when it lands.
    let mut cells = HashMap::<(i32, i32, i32), (Vec3, f32)>::new();
    if !subway.0 {
        for (name, transform, visibility) in &people {
            if *visibility == Visibility::Hidden || !name.starts_with("Hinchas") {
                continue;
            }
            let p = transform.translation;
            // Height bands keep one disc from cutting through stacked terraces.
            let key = ((p.x / CELL).floor() as i32, (p.z / CELL).floor() as i32, (p.y / 4.).floor() as i32);
            let cell = cells.entry(key).or_default();
            cell.0 += p;
            cell.1 += 1.;
        }
    }
    let mut used = 0;
    for (sum, count) in cells.into_values() {
        let transform = Transform::from_translation(sum / count + Vec3::Y * 0.2)
            .with_scale(Vec3::splat((16. + 10. * count.sqrt()).min(60.)));
        if let Some(&id) = pool.get(used) {
            let (mut t, mut v) = halos.get_mut(id).unwrap();
            t.set_if_neq(transform);
            v.set_if_neq(Visibility::Inherited);
        } else {
            pool.push(
                commands
                    .spawn((Halo, Mesh3d(art.quad.clone()), MeshMaterial3d(art.halo.clone()), transform))
                    .id(),
            );
        }
        used += 1;
    }
    for &id in &pool[used..] {
        halos.get_mut(id).unwrap().1.set_if_neq(Visibility::Hidden);
    }
}

fn lay_flows(art: Res<Art>, journeys: Res<FanJourneys>, scenario: Res<Scenario>, old: Query<Entity, With<Flow>>, mut commands: Commands) {
    for id in &old {
        commands.entity(id).despawn();
    }
    // One schematic arc per distinct last walk of public fans: from the stop they leave to the gate they reach,
    // active from the first alight to the last gate entry.
    let mut paths = HashMap::<(usize, [u32; 4]), (Vec3, Vec3, Vec3, f32, f32)>::new();
    for (venue, event) in scenario.0["events"].as_array().into_iter().flatten().enumerate() {
        let venue_at = world(event["longitude"].as_f64().unwrap(), event["latitude"].as_f64().unwrap());
        for g in journeys.for_venue(venue).iter().filter(|g| g.mode == FanMode::Public) {
            let (a, b) = (g.walk_path[0], *g.walk_path.last().unwrap());
            let path = paths.entry((venue, [a.x, a.z, b.x, b.z].map(f32::to_bits))).or_insert((a, b, venue_at, f32::MAX, f32::MIN));
            path.3 = path.3.min(g.alight_seconds);
            path.4 = path.4.max(g.gate_seconds);
        }
    }
    for (a, b, venue_at, from, to) in paths.into_values() {
        // Bow the arc away from the venue so the stands do not hide it.
        let side = Vec3::new(a.z - b.z, 0., b.x - a.x);
        let bend = side * 0.25 * -side.dot(venue_at - a.lerp(b, 0.5)).signum();
        let at = |t: f32| {
            let p = a.lerp(b, t) + bend * 4. * t * (1. - t);
            Vec3::new(p.x, terrain_from_xz(p.x, p.z) + 1.5, p.z)
        };
        let mut spawn = |mesh: &Handle<Mesh>, transform| {
            commands.spawn((Flow { from, to }, Mesh3d(mesh.clone()), MeshMaterial3d(art.flow.clone()), transform, Visibility::Hidden));
        };
        let yaw = |d: Vec3| Quat::from_rotation_y((-d.z).atan2(d.x));
        let steps = 64;
        let length = a.distance(b) * 1.1;
        let mut gap_left = 0.;
        let mut dash = DASH;
        for i in 0..steps {
            let (p, q) = (at(i as f32 / steps as f32), at((i + 1) as f32 / steps as f32));
            let piece = length / steps as f32;
            // Draw the step while inside a dash; the arc is fine enough that dashes need not split steps.
            if gap_left <= 0. {
                spawn(&art.quad, Transform::from_translation((p + q) * 0.5).with_rotation(yaw(q - p)).with_scale(Vec3::new(p.distance(q) + 0.3, 1., WIDTH)));
                dash -= piece;
                if dash <= 0. {
                    gap_left = GAP;
                    dash = DASH;
                }
            } else {
                gap_left -= piece;
            }
        }
        let (p, q) = (at(0.97), at(1.));
        spawn(&art.head, Transform::from_translation(q).with_rotation(yaw(q - p)).with_scale(Vec3::splat(WIDTH * 3.5)));
    }
}

fn show_flows(
    clock: Res<CityClock>,
    subway: Res<SubwayView>,
    rig: Res<Rig>,
    mut flows: Query<(&Flow, &mut Visibility)>,
) {
    let now = clock.0.map(|s| s.rem_euclid(86_400.));
    for (flow, mut visibility) in &mut flows {
        let on = !subway.0 && rig.distance < 3_000. && now.is_some_and(|t| (flow.from - LEAD..=flow.to + LEAD).contains(&t));
        visibility.set_if_neq(if on { Visibility::Inherited } else { Visibility::Hidden });
    }
}
