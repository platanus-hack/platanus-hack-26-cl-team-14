//! Stadium arrivals use the same weighted journeys and city clock as their vehicles.
//! Other neighborhood events retain their schematic local animation.
#[path = "events/walk.rs"]
mod walk;
use super::*;
use super::{
    clock::{CityClock, UpdateClock},
    fan_travel::{FanJourneys, FanMode, RefreshFanJourneys},
};
use crate::{
    Rig,
    city::{StadiumSeating, SubwayView},
};
use bevy::{text::FontSize, time::Real};
use super::silhouette::{Silhouette, Silhouettes, face};
use walk::{OriginWalk, WalkPath};

const PHASE_SECONDS: f32 = 10.;
// These are display transitions in replay seconds, not a measured gate capacity.
const GATE_FADE_SECONDS: f32 = 30.;
const SOLID: usize = 12;
const GROUP_SIZE: f64 = 192.;
const COLORS: [Color; 3] = [
    Color::srgb(0.09, 0.90, 0.82),
    Color::srgb(1., 0.69, 0.19),
    Color::srgb(0.83, 0.64, 1.),
];

pub(crate) struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                refresh
                    .after(crate::city::draw_buildings)
                    .after(RefreshFanJourneys)
                    .run_if(resource_changed::<Scenario>),
            )
            .add_systems(Update, animate.after(refresh).after(UpdateClock));
    }
}

#[derive(Resource)]
struct Art {
    figure: Handle<Mesh>,
    modes: [Handle<Mesh>; 2],
    glyph: [Handle<StandardMaterial>; 2],
    paint: [[Handle<StandardMaterial>; SOLID + 1]; 3],
}

#[derive(Component)]
struct Key;
#[derive(Component)]
struct Caption;
#[derive(Component)]
struct Group {
    venue: usize,
    mode: usize,
    path: GroupPath,
    delay: f32,
}

enum GroupPath {
    Local {
        from: Vec3,
        to: Vec3,
    },
    Arrival {
        origin: Option<OriginWalk>,
        walk: WalkPath,
        alight: f32,
        gate: f32,
        seat: Vec3,
        facing: Vec3,
    },
}

struct Event {
    center: Vec3,
    end: f32,
    started: f32,
    captions: [String; 2],
    routed: bool,
    progress: Option<(usize, f32)>,
}

#[derive(Default, Resource)]
struct Playback {
    events: Vec<Event>,
    scene_revision: Option<u64>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fonts: ResMut<Assets<Font>>,
    people: Res<Silhouettes>,
) {
    // A single transit glyph covers pooled public modes. Vehicle symbols carry
    // the same weight as walking figures; their number is not a fleet estimate.
    let white = [255, 255, 255];
    let vehicle = |length, roof_length| {
        shape([
            block([1.35, 0.65, length], [0., 0.53, 0.], white),
            block([1.15, 0.55, roof_length], [0., 1.12, 0.], GLASS),
            block([1.25, 0.17, roof_length], [0., 1.47, 0.], white),
        ])
    };
    commands.insert_resource(Art {
        modes: [
            meshes.add(vehicle(3.6, 3.15)),
            meshes.add(vehicle(2.2, 1.15)),
        ],
        glyph: [0, 1].map(|mode| {
            materials.add(StandardMaterial {
                base_color: COLORS[mode],
                unlit: true,
                ..default()
            })
        }),
        figure: people.stand.clone(),
        paint: COLORS.map(|color| {
            std::array::from_fn(|step| {
                materials.add(StandardMaterial {
                    base_color: color.with_alpha(step as f32 / SOLID as f32),
                    base_color_texture: Some(people.atlas.clone()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })
            })
        }),
    });
    commands.init_resource::<Playback>();
    let font = fonts.add(Font::from_bytes(
        include_bytes!("../assets/barlow/Barlow-Regular.ttf").to_vec(),
    ));
    commands
        .spawn((
            Key,
            Visibility::Hidden,
            BackgroundColor(crate::label::PANEL_INK),
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                bottom: px(240),
                padding: UiRect::axes(px(14), px(8)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((
                Caption,
                Text::default(),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(27.),
                    ..default()
                },
            ));
            panel
                .spawn(Node {
                    column_gap: px(16),
                    ..default()
                })
                .with_children(|row| {
                    for (name, color) in ["Público", "Auto", "Otros"].into_iter().zip(COLORS) {
                        row.spawn((
                            Text::new(name),
                            TextColor(color),
                            TextFont {
                                font: font.clone().into(),
                                font_size: FontSize::Px(21.),
                                ..default()
                            },
                        ));
                    }
                });
        });
}

fn spawn_local_groups(
    commands: &mut Commands,
    art: &Art,
    zones: &[Value],
    event: &Value,
    center: Vec3,
    venue: usize,
) -> [String; 2] {
    let mut totals = [0.; 3];
    for (key, count) in event["attendees_by_census_mode"]
        .as_object()
        .into_iter()
        .flatten()
    {
        totals[match key.as_str() {
            "2" => 0,
            "1" => 1,
            _ => 2,
        }] += count.as_f64().unwrap_or_default();
    }
    let total: f64 = totals.iter().sum();
    if total <= 0. {
        return Default::default();
    }
    let groups = (total / GROUP_SIZE).round().clamp(1., 512.) as usize;
    let mut mass = 0.;
    let mut used = 0;
    for (mode, count) in totals.into_iter().enumerate() {
        mass += count;
        let boundary = (mass / total * groups as f64).round() as usize;
        let n = boundary - used;
        used = boundary;
        let mut home_mass = 0.;
        let homes: Vec<_> = zones
            .iter()
            .filter_map(|zone| {
                let id = zone["id"].as_str()?;
                let weight = event["attendees_by_return_comuna"][id].as_f64()?;
                let home = world(zone["longitude"].as_f64()?, zone["latitude"].as_f64()?);
                let bearing = (home.x - center.x).atan2(home.z - center.z);
                home_mass += weight;
                Some((home_mass, bearing))
            })
            .collect();
        for i in 0..n {
            let q = (i as f64 + 0.5) / n as f64;
            let home = homes.partition_point(|(upper, _)| *upper <= q * home_mass);
            // Eight broad sectors preserve direction without suggesting individual itineraries.
            let sector = std::f32::consts::FRAC_PI_4;
            let scatter = ((i as f32 * 0.618034 + mode as f32 * 0.31).fract() - 0.5) * 0.67;
            let theta = homes
                .get(home)
                .map_or(q as f32 * std::f32::consts::TAU, |&(_, b)| {
                    (b / sector).round() * sector
                })
                + scatter;
            let direction = Vec3::new(theta.sin(), 0., theta.cos());
            let path = GroupPath::Local {
                from: center + direction * (122. + 47. * (q as f32 * 23.).fract()),
                to: center + direction * (18. + 24. * (q as f32 * 13.).fract()),
            };
            commands.spawn((
                Group {
                    venue,
                    mode,
                    path,
                    delay: (i as f32 * 0.414214 + mode as f32 * 0.15).fract() * 0.49,
                },
                Mesh3d(art.figure.clone()),
                MeshMaterial3d(art.paint[mode][SOLID].clone()),
                Visibility::Hidden,
            ));
        }
    }
    let formatted = people_label(totals[0]);
    let labels = ["Llegada ilustrativa", "Regreso estimado"];
    labels.map(|label| format!(
        "{label} · {formatted} en transporte público\nFlujo esquemático · 1 figura ≈ {:.0} personas",
        total / groups as f64
    ))
}

fn people_label(people: f64) -> String {
    let n = people.round() as u64;
    if n >= 1000 {
        format!("{}.{:03}", n / 1000, n % 1000)
    } else {
        n.to_string()
    }
}

fn refresh(
    mut commands: Commands,
    scenario: Res<Scenario>,
    journeys: Res<FanJourneys>,
    art: Res<Art>,
    mut playback: ResMut<Playback>,
    time: Res<Time<Real>>,
    mut rig: ResMut<Rig>,
    old: Query<Entity, With<Group>>,
    bowls: Query<(Entity, &StadiumSeating, &Transform, &Visibility)>,
) {
    for e in &old {
        commands.entity(e).despawn();
    }
    let previous = std::mem::take(&mut playback.events);
    let result = &scenario.0;
    let scene_revision = result["scene"]["revision"].as_u64();
    let replay_restarted = scene_revision != playback.scene_revision;
    playback.scene_revision = scene_revision;
    let zones = result["zones"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    for row in result["events"].as_array().into_iter().flatten() {
        let center = world(
            row["longitude"].as_f64().unwrap(),
            row["latitude"].as_f64().unwrap(),
        );
        let end = row["assumed_end_half_hour"].as_f64().unwrap() as f32 * BIN;
        if rig.event_focus == Some(center) {
            rig.event_focus = None;
            rig.fly_to(center, 650.);
        }
        let stadium = bowls.iter().find_map(|(id, bowl, transform, visibility)| {
            let local = transform
                .compute_affine()
                .inverse()
                .transform_point3(center);
            (*visibility != Visibility::Hidden && bowl.contains(local)).then_some(id)
        });
        let venue = playback.events.len();
        let routed = stadium.is_some() || row["fan_base"]["by_home"].is_array();
        let captions = if routed {
            let groups = journeys.for_venue(venue);
            let mut represented = 0.;
            let mut figures = 0usize;
            if let Some((_, bowl, landmark, _)) = stadium.and_then(|id| bowls.get(id).ok()) {
                for group in groups {
                    let Some(walk) = WalkPath::to_gate(group.walk_path.clone(), bowl, landmark)
                    else {
                        continue;
                    };
                    let mode = match group.mode {
                        FanMode::Public => 0,
                        FanMode::Car => 1,
                        FanMode::Other => 2,
                    };
                    let turn = ((group.index as f32 + 0.5) * 0.618034).fract();
                    let row = 0.32 + 0.50 * (group.index as f32 * 0.414214).fract();
                    let seat = landmark.transform_point(bowl.terrace(turn, row));
                    represented += group.people;
                    figures += 1;
                    commands.spawn((
                        Name::new(format!(
                            "Hinchas {} · grupo {}",
                            group.home_cut, group.index
                        )),
                        Group {
                            venue,
                            mode,
                            delay: 0.,
                            path: GroupPath::Arrival {
                                origin: group.origin_walk.as_ref().and_then(|leg| {
                                    Some(OriginWalk {
                                        path: WalkPath::along(leg.path.clone())?,
                                        start: leg.start_seconds,
                                        end: leg.end_seconds,
                                    })
                                }),
                                walk,
                                alight: group.alight_seconds,
                                gate: group.gate_seconds,
                                seat,
                                facing: center - seat,
                            },
                        },
                        Silhouette,
                        Mesh3d(art.figure.clone()),
                        MeshMaterial3d(art.paint[mode][SOLID].clone()),
                        Visibility::Hidden,
                    ));
                }
            }
            let public = row["fan_base"]["by_mode"]["public"]
                .as_f64()
                .or_else(|| row["attendees_by_census_mode"]["2"].as_f64())
                .unwrap_or_default();
            let coverage = if figures > 0 {
                format!(
                    "{} de {} con viaje · promedio de {} por figura\ningreso al estadio: esquemático",
                    people_label(represented),
                    people_label(row["attendance"].as_f64().unwrap_or_default()),
                    people_label(represented / figures as f64)
                )
            } else {
                "Sin recorridos de llegada en este replay".to_owned()
            };
            let caption = format!(
                "Llegada estimada · {} en transporte público\n{coverage}",
                people_label(public)
            );
            [caption.clone(), caption]
        } else {
            spawn_local_groups(&mut commands, &art, zones, row, center, venue)
        };
        let started = previous
            .iter()
            .find(|e| !replay_restarted && e.center == center && e.end == end)
            .map_or(time.elapsed_secs(), |e| e.started);
        playback.events.push(Event {
            center,
            end,
            started,
            captions,
            routed,
            progress: None,
        });
    }
}

fn animate(
    art: Res<Art>,
    real: Res<Time<Real>>,
    fixed: Res<FixedClock>,
    city_clock: Res<CityClock>,
    mut playback: ResMut<Playback>,
    subway: Res<SubwayView>,
    rig: Res<Rig>,
    people: Res<Silhouettes>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    mut groups: Query<
        (
            &Group,
            &mut Transform,
            &mut Visibility,
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Without<Key>,
    >,
    mut panel: Single<&mut Visibility, With<Key>>,
    mut caption: Single<&mut Text, With<Caption>>,
) {
    for event in &mut playback.events {
        if event.routed {
            event.progress = (!subway.0 && city_clock.0.is_some()).then_some((0, 0.));
            continue;
        }
        // The event sequence reconstructs the two journeys within the presentation shot.
        // Only a fixed capture clock selects a specific time relative to the assumed end.
        let t = fixed.0.map_or_else(
            || ((real.elapsed_secs() - event.started) / PHASE_SECONDS).rem_euclid(2.),
            |s| {
                let delta = (s - event.end + DAY / 2.).rem_euclid(DAY) - DAY / 2.;
                1. + delta / BIN
            },
        );
        event.progress = (!subway.0 && (0.0..2.0).contains(&t)).then_some((t as usize, t.fract()));
    }
    let label = playback
        .events
        .iter()
        .filter_map(|e| {
            Some((
                e.center.distance_squared(rig.target),
                &e.captions[e.progress?.0],
            ))
        })
        .filter(|(distance, text)| *distance < (rig.distance * 0.8).powi(2) && !text.is_empty())
        .min_by(|(a, _), (b, _)| a.total_cmp(b));
    panel.set_if_neq(if label.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if let Some((_, text)) = label
        && caption.0 != *text
    {
        caption.0.clone_from(text);
    }
    for (group, mut transform, mut visibility, mut mesh, mut paint) in &mut groups {
        if rig.distance >= FIGURE_MAX_DISTANCE_M {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }
        let Some((phase, t)) = playback.events[group.venue].progress else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let map_scale = (rig.distance * 0.0035).clamp(1.1, 3.8);
        let (point, heading, figure, scale) = match group.path {
            GroupPath::Arrival {
                ref origin,
                ref walk,
                alight,
                gate,
                seat,
                facing,
            } => {
                let now = city_clock.0.unwrap().rem_euclid(DAY);
                let admitted = now - gate;
                // Walking fans step at 1.35 m/s with a 1.48 m stride; each group has its own phase.
                let frame = ((now * 1.35 / 1.48 + (seat.x * 0.37).fract().abs()).fract() * people.walk.len() as f32) as usize;
                let figure = if admitted < 0. {
                    &people.walk[frame]
                } else if admitted < GATE_FADE_SECONDS {
                    &people.stand
                } else {
                    &people.sit
                };
                let ((point, heading), opacity) = if now < alight {
                    let Some(pose) = origin.as_ref().and_then(|leg| leg.at(now)) else {
                        visibility.set_if_neq(Visibility::Hidden);
                        continue;
                    };
                    (pose, 1.)
                } else if admitted < 0. {
                    (walk.walk((now - alight) / (gate - alight)), 1.)
                } else if admitted < GATE_FADE_SECONDS {
                    (walk.walk(1.), 1. - admitted / GATE_FADE_SECONDS)
                } else {
                    // The admitted group fills its seat without a fictitious interior walk.
                    (
                        (seat, facing),
                        ((admitted - GATE_FADE_SECONDS) / GATE_FADE_SECONDS).min(1.),
                    )
                };
                let step = (opacity * SOLID as f32).round() as usize;
                paint.set_if_neq(MeshMaterial3d(art.paint[group.mode][step].clone()));
                // Beyond 300 m a fan keeps the screen height of the former 3 m glyph.
                let legible = 1. + (3. / 1.75 - 1.) * ((rig.distance - 300.) / 200.).clamp(0., 1.);
                let scale = legible * if admitted < GATE_FADE_SECONDS {
                    pedestrian_scale(rig.distance, map_scale)
                } else {
                    map_scale.min(1.15)
                };
                (point, heading, figure, scale)
            }
            GroupPath::Local { from, to } => {
                let p = (t - group.delay) / 0.51;
                if !(0.0..=1.).contains(&p) {
                    visibility.set_if_neq(Visibility::Hidden);
                    continue;
                }
                let u = if phase == 0 { p } else { 1. - p };
                let mut point = from.lerp(to, u);
                point.y = terrain_from_xz(point.x, point.z)
                    + 21.
                    + 20. * (u * std::f32::consts::PI).sin();
                let transit = u < 0.56 && group.mode < 2;
                paint.set_if_neq(MeshMaterial3d(if transit {
                    art.glyph[group.mode].clone()
                } else {
                    art.paint[group.mode][SOLID].clone()
                }));
                (point, to - from, if transit { &art.modes[group.mode] } else { &art.figure }, map_scale)
            }
        };
        mesh.set_if_neq(Mesh3d(figure.clone()));
        transform.translation = point;
        transform.scale = Vec3::splat(scale);
        let local = matches!(group.path, GroupPath::Local { .. });
        if *figure != art.figure && local {
            transform.rotation = Quat::from_rotation_y(heading.x.atan2(heading.z));
        } else if local {
            face(&mut transform, heading, camera.translation());
        } else {
            // Silhouette turns this walk direction to the camera after this system.
            transform.rotation = Quat::from_rotation_y((-heading.z).atan2(heading.x));
        }
        visibility.set_if_neq(Visibility::Inherited);
    }
}
