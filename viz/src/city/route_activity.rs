//! Play saved route counts at the activity's step boundaries.
use super::{SubwayView, Surface, ground::ribbon, world};
use crate::{
    movers::clock::{CityClock, UpdateClock},
    net::Scenario,
};
use bevy::prelude::*;
use serde::Deserialize;

pub(crate) struct RouteActivityPlugin;

impl Plugin for RouteActivityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Playback>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (refresh.run_if(resource_changed::<Scenario>), draw)
                    .chain()
                    .after(UpdateClock),
            );
    }
}

#[derive(Deserialize)]
struct Activity {
    playback_revision: u64,
    segments: Vec<Segment>,
    steps: Vec<Step>,
}

#[derive(Deserialize)]
struct Segment {
    id: String,
    geometry: Vec<[f64; 2]>,
}

#[derive(Deserialize)]
struct Step {
    minute: u16,
    open: bool,
    segments: Vec<Counts>,
}

#[derive(Deserialize)]
struct Counts {
    id: String,
    moving: u64,
    queued: u64,
    visiting: u64,
}

#[derive(Resource, Default)]
struct Playback(Option<(Activity, f32)>);

#[derive(Resource)]
struct Art {
    route: Handle<StandardMaterial>,
    moving: Handle<StandardMaterial>,
    queue: Handle<StandardMaterial>,
    visit: Handle<StandardMaterial>,
    marker: Handle<Mesh>,
    fixed_clock: bool,
}

#[derive(Component)]
struct SegmentMark {
    index: usize,
    kind: Mark,
}

enum Mark {
    Route,
    Queue,
    Visit,
}

#[derive(Component)]
struct ActivityLabel;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fonts: ResMut<Assets<Font>>,
) {
    let mut ink = |r, g, b| {
        materials.add(StandardMaterial {
            base_color: Color::srgb_u8(r, g, b),
            unlit: true,
            ..default()
        })
    };
    commands.insert_resource(Art {
        route: ink(72, 81, 114),
        moving: ink(0, 125, 111),
        queue: ink(255, 192, 66),
        visit: ink(161, 105, 211),
        marker: meshes.add(Cylinder::new(5., 0.5)),
        fixed_clock: std::env::var_os("SANTIAGO_CLOCK_SECONDS").is_some(),
    });
    let font = fonts.add(Font::from_bytes(
        include_bytes!("../../assets/barlow/Barlow-Regular.ttf").to_vec(),
    ));
    commands.spawn((
        ActivityLabel,
        Text::default(),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(32.),
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(crate::label::PANEL_INK),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(34),
            left: px(24),
            padding: UiRect::axes(px(18), px(9)),
            ..default()
        },
        Visibility::Hidden,
    ));
}

fn refresh(
    mut commands: Commands,
    scenario: Res<Scenario>,
    clock: Res<CityClock>,
    art: Res<Art>,
    mut playback: ResMut<Playback>,
    mut meshes: ResMut<Assets<Mesh>>,
    old: Query<Entity, With<SegmentMark>>,
) {
    let value = &scenario.0["activity"];
    if value["playback_revision"].as_u64()
        == playback
            .0
            .as_ref()
            .map(|(activity, _)| activity.playback_revision)
    {
        return;
    }
    for entity in &old {
        commands.entity(entity).despawn();
    }
    playback.0 = None;
    if value.is_null() {
        return;
    }
    let activity: Activity = serde_json::from_value(value.clone()).unwrap();
    for (index, segment) in activity.segments.iter().enumerate() {
        let points: Vec<_> = segment
            .geometry
            .iter()
            .map(|&[lon, lat]| world(lon, lat))
            .collect();
        let mut surface = Surface::default();
        ribbon(&mut surface, &points, 3., 0.55, [1.; 4]);
        commands.spawn((
            SegmentMark {
                index,
                kind: Mark::Route,
            },
            Mesh3d(meshes.add(surface.finish())),
            MeshMaterial3d(art.route.clone()),
            Visibility::Hidden,
        ));
        for (kind, point, material) in [
            (Mark::Queue, points[0], &art.queue),
            (Mark::Visit, *points.last().unwrap(), &art.visit),
        ] {
            commands.spawn((
                SegmentMark { index, kind },
                Mesh3d(art.marker.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(point + Vec3::Y),
                Visibility::Hidden,
            ));
        }
    }
    playback.0 = Some((activity, clock.0.unwrap()));
}

fn draw(
    playback: Res<Playback>,
    clock: Res<CityClock>,
    subway: Res<SubwayView>,
    art: Res<Art>,
    mut displayed: Local<Option<(u64, usize, bool)>>,
    mut marks: Query<
        (
            &SegmentMark,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Visibility,
        ),
        Without<ActivityLabel>,
    >,
    mut label: Single<(&mut Text, &mut Visibility), With<ActivityLabel>>,
) {
    let Some((activity, origin)) = &playback.0 else {
        label.1.set_if_neq(Visibility::Hidden);
        *displayed = None;
        return;
    };
    let now = clock.0.unwrap();
    let minute = if art.fixed_clock {
        now / 60.
    } else {
        f32::from(activity.steps[0].minute) + (now - origin) / 60.
    };
    let current = activity
        .steps
        .partition_point(|step| f32::from(step.minute) <= minute)
        .saturating_sub(1);
    let key = (activity.playback_revision, current, subway.0);
    if *displayed == Some(key) {
        return;
    }
    *displayed = Some(key);
    let step = &activity.steps[current];
    for (mark, mut material, mut visible) in &mut marks {
        let id = &activity.segments[mark.index].id;
        let counts = step
            .segments
            .iter()
            .find(|counts| counts.id == *id)
            .unwrap();
        let show = match mark.kind {
            Mark::Route => {
                let ink = if counts.moving > 0 {
                    &art.moving
                } else {
                    &art.route
                };
                material.set_if_neq(MeshMaterial3d(ink.clone()));
                true
            }
            Mark::Queue => counts.queued > 0,
            Mark::Visit => counts.visiting > 0,
        };
        visible.set_if_neq(if show && !subway.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let (moving, queued, visiting) = step.segments.iter().fold((0, 0, 0), |(m, q, v), segment| {
        (m + segment.moving, q + segment.queued, v + segment.visiting)
    });
    let phase = if step.open { "Abierta" } else { "Cerrada" };
    label.0.set_if_neq(Text::new(format!(
        "Actividad {:02}:{:02} · {phase}\nEn ruta {moving} · En espera {queued} · En visita {visiting}",
        step.minute / 60,
        step.minute % 60,
    )));
    label.1.set_if_neq(if subway.0 {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
}
