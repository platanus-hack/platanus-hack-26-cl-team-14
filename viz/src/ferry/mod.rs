//! Native ferry playback. A saved boat line supplies the timetable and a saved
//! line_riders row supplies representative passengers. No simulation runs here.
mod access;
mod passengers;
mod underpass;
mod vessel;
use underpass::{FerryMaterial, RiverDepth};

use crate::{
    city::SubwayView,
    movers::{
        Model, Models,
        clock::{CityClock, UpdateClock},
    },
    net::Scenario,
};
use bevy::{light::NotShadowCaster, prelude::*};
use passengers::{DAY, Sailing, Trip};
use std::collections::HashMap;
use vessel::Line;

const WALK_SPEED: f32 = 1.33;
const CROSSING: f32 = 4.;

pub(super) struct FerryPlugin;
impl Plugin for FerryPlugin {
    fn build(&self, app: &mut App) {
        underpass::install(app);
        app.init_resource::<Service>()
            .add_systems(PostStartup, setup)
            .add_systems(PostUpdate, crate::city::edits::ferry_routes)
            .add_systems(
                Update,
                (refresh.run_if(resource_changed::<Scenario>), animate)
                    .chain()
                    .after(UpdateClock),
            );
    }
}

struct Direction {
    line: Line,
    runs: Vec<Sailing>,
}

#[derive(Resource, Default)]
struct Service {
    directions: Vec<Direction>,
    docks: HashMap<[i32; 4], access::AccessPath>,
}

fn dock_key(stop: &vessel::Stop) -> [i32; 4] {
    [stop.location.x, stop.location.y, stop.pier.x, stop.pier.z].map(|x| x.round() as i32)
}

#[derive(Resource)]
struct Symbols {
    access: access::AccessMap,
    boat: Handle<Mesh>,
    person: Handle<Mesh>,
    plank: Handle<Mesh>,
    font: Handle<Font>,
    hull: Handle<FerryMaterial>,
    coat: Handle<FerryMaterial>,
    deck_coat: Handle<FerryMaterial>,
    timber: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut ferries: ResMut<Assets<FerryMaterial>>,
    mut images: ResMut<Assets<Image>>,
    models: Res<Models>,
    mut fonts: ResMut<Assets<Font>>,
) {
    let mut person = meshes.get(&models.mesh[&Model::Person]).unwrap().clone();
    person.remove_attribute(Mesh::ATTRIBUTE_COLOR);
    let material = |color| StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    };
    let (dem, grid) = crate::city::ferry_depth_grid();
    let dem = images.add(dem);
    let mut ferry = |color, aboard| {
        ferries.add(FerryMaterial {
            base: material(color),
            extension: RiverDepth {
                dem: dem.clone(),
                grid,
                aboard,
            },
        })
    };
    let coat = Color::srgb_u8(111, 60, 115);
    commands.insert_resource(Symbols {
        access: access::AccessMap::load(),
        font: fonts.add(Font::from_bytes(
            include_bytes!("../../assets/barlow/Barlow-Regular.ttf").to_vec(),
        )),
        boat: meshes.add(vessel::mesh()),
        person: meshes.add(person),
        plank: meshes.add(Cuboid::from_size(Vec3::ONE)),
        hull: ferry(Color::WHITE, 1),
        coat: ferry(coat, 0),
        deck_coat: ferry(coat, 1),
        timber: mats.add(material(Color::srgb_u8(108, 117, 129))),
    });
}

#[derive(Component)]
struct FerryEntity;

#[derive(Component)]
struct FerryLegend;

#[derive(Component)]
enum Actor {
    Boat {
        line: usize,
        run: usize,
    },
    Person {
        line: usize,
        run: usize,
        trip: usize,
    },
    Pier,
}

fn refresh(
    mut commands: Commands,
    scenario: Res<Scenario>,
    symbols: Res<Symbols>,
    mut service: ResMut<Service>,
    old: Query<Entity, With<FerryEntity>>,
) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    service.directions.clear();
    service.docks.clear();
    for line in Line::from_result(&scenario.0) {
        let ix = service.directions.len();
        let demand = scenario.0["line_riders"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|r| r["id"].as_str() == Some(&line.id));
        let runs = passengers::schedule(&line, demand);
        for stop in &line.stops {
            let std::collections::hash_map::Entry::Vacant(slot) =
                service.docks.entry(dock_key(stop))
            else {
                continue;
            };
            let (_, heading) = line.sail(stop.arrival);
            let path = slot.insert(symbols.access.path(stop.location, stop.pier, heading));
            let a = path.gangway_start() + Vec3::Y * 0.85;
            let b = path.at(1.) + Vec3::Y * 0.85;
            let span = b - a;
            let t = Transform::from_translation(a.midpoint(b))
                .looking_to(span, Vec3::Y)
                .with_scale(Vec3::new(2.4, 0.50, span.length()));
            commands.spawn((
                FerryEntity,
                Actor::Pier,
                Mesh3d(symbols.plank.clone()),
                MeshMaterial3d(symbols.timber.clone()),
                t,
            ));
        }
        for (run, sailing) in runs.iter().enumerate() {
            commands.spawn((
                FerryEntity,
                Actor::Boat { line: ix, run },
                NotShadowCaster,
                Mesh3d(symbols.boat.clone()),
                MeshMaterial3d(symbols.hull.clone()),
                Visibility::Hidden,
            ));
            for trip in 0..sailing.trips.len() {
                commands.spawn((
                    FerryEntity,
                    Actor::Person {
                        line: ix,
                        run,
                        trip,
                    },
                    Mesh3d(symbols.person.clone()),
                    MeshMaterial3d(symbols.coat.clone()),
                    Visibility::Hidden,
                ));
            }
        }
        service.directions.push(Direction { line, runs });
    }
    if service
        .directions
        .iter()
        .any(|d| d.runs.iter().any(|s| !s.trips.is_empty()))
    {
        commands.spawn((
            FerryEntity,
            FerryLegend,
            Text::new(format!(
                "Ferry · 1 figura por cada {} viajes al trabajo",
                passengers::TRIPS_PER_FIGURE
            )),
            TextFont {
                font: symbols.font.clone().into(),
                font_size: bevy::text::FontSize::Px(21.),
                ..default()
            },
            TextColor(Color::WHITE),
            BackgroundColor(Color::srgba(0.05, 0.07, 0.09, 0.78)),
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                top: px(92),
                padding: UiRect::axes(px(10), px(4)),
                ..default()
            },
        ));
    }
}

// Times just before midnight can already belong to a sailing on the next day.
fn age(now: f32, start: f32) -> f32 {
    (now - start + DAY * 0.5).rem_euclid(DAY) - DAY * 0.5
}

fn boat_pose(line: &Line, age: f32) -> Transform {
    let (p, heading) = line.sail(age);
    Transform::from_translation(p).with_rotation(Quat::from_rotation_y(heading.x.atan2(heading.z)))
}

fn walk(path: &access::AccessPath, distance: f32, forward: bool, slot: usize) -> Vec3 {
    let t = distance / path.length();
    let p = path.at(t);
    let d = path.at(t + 0.01) - path.at(t - 0.01);
    let side = Vec3::new(d.z, 0., -d.x).normalize_or_zero();
    let lane = 0.25 + 0.16 * (slot % 3) as f32;
    p + Vec3::Y * 1.14 + side * if forward { lane } else { -lane }
}

fn person_pose(
    line: &Line,
    t: &Trip,
    age: f32,
    deck_rows: usize,
    paths: &HashMap<[i32; 4], access::AccessPath>,
) -> Option<Transform> {
    let from = paths.get(&dock_key(&line.stops[t.on]));
    let to = paths.get(&dock_key(&line.stops[t.off]));
    let seat = Vec3::new(
        -1.95 + (t.seat % 4) as f32 * 1.3,
        1.30,
        -1.9 + (t.seat / 4) as f32 * (7.0 / deck_rows.saturating_sub(1).max(1) as f32).min(1.3),
    );
    let deck = |time| boat_pose(line, time).transform_point(seat);
    let p = if age < t.board_at {
        let path = from?;
        let end = path.length() - (t.seat % 5) as f32 * 0.8;
        let d = end + (age - t.board_at + t.wait).min(0.) * WALK_SPEED;
        if d < 0. {
            return None;
        }
        walk(path, d, true, t.seat)
    } else if age < t.board_at + CROSSING {
        let path = from?;
        let d = path.length() - (t.seat % 5) as f32 * 0.8;
        walk(path, d, true, t.seat).lerp(deck(age), (age - t.board_at) / CROSSING)
    } else if age < t.leave_at {
        deck(age)
    } else if age < t.leave_at + CROSSING {
        let path = to?;
        deck(age).lerp(
            walk(path, path.length(), false, t.seat),
            (age - t.leave_at) / CROSSING,
        )
    } else {
        let path = to?;
        let d = path.length() - (age - t.leave_at - CROSSING) * WALK_SPEED;
        if d < 0. {
            return None;
        }
        walk(path, d, false, t.seat)
    };
    Some(Transform::from_translation(p).with_scale(Vec3::splat(1.45)))
}

fn animate(
    mut commands: Commands,
    clock: Res<CityClock>,
    mode: Res<SubwayView>,
    service: Res<Service>,
    symbols: Res<Symbols>,
    mut actors: Query<(
        Entity,
        &Actor,
        &mut Transform,
        &mut Visibility,
        Option<&mut MeshMaterial3d<FerryMaterial>>,
    )>,
    mut legends: Query<&mut Visibility, (With<FerryLegend>, Without<Actor>)>,
) {
    let Some(now) = clock.0 else {
        return;
    };
    for mut legend in &mut legends {
        legend.set_if_neq(if mode.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    for (entity, actor, mut position, mut visibility, material) in &mut actors {
        let pose = if mode.0 {
            None
        } else {
            match *actor {
                Actor::Pier => {
                    visibility.set_if_neq(Visibility::Inherited);
                    continue;
                }
                Actor::Boat { line, run } => {
                    let d = &service.directions[line];
                    let age = age(now, d.runs[run].start);
                    (-15. ..d.line.duration + CROSSING)
                        .contains(&age)
                        .then(|| boat_pose(&d.line, age))
                }
                Actor::Person { line, run, trip } => {
                    let d = &service.directions[line];
                    let sailing = &d.runs[run];
                    let trip = &sailing.trips[trip];
                    let age = age(now, sailing.start);
                    let aboard = (trip.board_at..trip.leave_at + CROSSING).contains(&age);
                    let coat = if aboard {
                        &symbols.deck_coat
                    } else {
                        &symbols.coat
                    };
                    if material.unwrap().set_if_neq(MeshMaterial3d(coat.clone())) {
                        // The shadow pass uses physical mesh heights. A symbol below a
                        // cartographic deck must not cast its silhouette on the pavement.
                        if aboard {
                            commands.entity(entity).insert(NotShadowCaster);
                        } else {
                            commands.entity(entity).remove::<NotShadowCaster>();
                        }
                    }
                    person_pose(&d.line, trip, age, sailing.deck_rows, &service.docks)
                }
            }
        };
        if let Some(pose) = pose {
            position.set_if_neq(pose);
            visibility.set_if_neq(Visibility::Inherited);
        } else {
            visibility.set_if_neq(Visibility::Hidden);
        }
    }
}
