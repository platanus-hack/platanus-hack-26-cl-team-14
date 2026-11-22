//! Weighted groups travel between a venue and the general direction of their comuna.
//! The geometry and playback duration are schematic. The result supplies the counts.
use super::*;
use crate::{
    Rig,
    city::{StadiumSeating, SubwayView},
};
use bevy::{text::FontSize, time::Real};

const PHASE_SECONDS: f32 = 10.;
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
                    .run_if(resource_changed::<Scenario>),
            )
            .add_systems(Update, animate.after(refresh));
    }
}

#[derive(Resource)]
struct Art {
    figure: Handle<Mesh>,
    modes: [Handle<Mesh>; 3],
    paint: [Handle<StandardMaterial>; 3],
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
    Stadium {
        entrance: f32,
        seat_turn: f32,
        row: f32,
        approach: f32,
    },
}

struct Event {
    center: Vec3,
    stadium: Option<Entity>,
    end: f32,
    started: f32,
    captions: [String; 2],
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
) {
    let head = Mesh::from(Sphere::new(1.))
        .scaled_by(Vec3::splat(0.65))
        .translated_by(Vec3::Y * 2.35);
    let mut figure = Mesh::from(Capsule3d::new(0.63, 0.8))
        .scaled_by(Vec3::new(1., 1., 0.75))
        .translated_by(Vec3::Y);
    figure.merge(&head).unwrap();
    let figure = meshes.add(figure);
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
            figure.clone(),
        ],
        figure,
        paint: COLORS.map(|color| {
            materials.add(StandardMaterial {
                base_color: color,
                // Draw the legend colors in the city pass after ink quantization.
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
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
            BackgroundColor(Color::srgba(0.035, 0.055, 0.08, 0.90)),
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                bottom: px(48),
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

// Keep the same crowd for both directions. The fan model supplies joint counts;
// generic events supply return marginals, so each mode shares the overall home mix.
fn spawn_groups(
    commands: &mut Commands,
    art: &Art,
    zones: &[Value],
    event: &Value,
    center: Vec3,
    venue: usize,
    at_stadium: bool,
) -> [String; 2] {
    let fan = &event["fan_base"];
    let fans = fan["by_home"].is_array();
    let census = &event["attendees_by_census_mode"];
    let totals = if fans {
        ["public", "car", "other"].map(|key| fan["by_mode"][key].as_f64().unwrap_or_default())
    } else {
        let mut counts = [0.; 3];
        for (key, count) in census.as_object().into_iter().flatten() {
            counts[match key.as_str() {
                "2" => 0,
                "1" => 1,
                _ => 2,
            }] += count.as_f64().unwrap_or_default();
        }
        counts
    };
    let total: f64 = totals.iter().sum();
    if total <= 0. {
        return Default::default();
    }
    let groups = (total / GROUP_SIZE).round().clamp(1., 512.) as usize;
    let mut mass = 0.;
    let mut used = 0;
    let fan_rows: HashMap<_, _> = fan["by_home"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| Some((row["cut"].as_str()?, row)))
        .collect();
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
                let weight = if fans {
                    fan_rows.get(id)?[["public", "car", "other"][mode]].as_f64()?
                } else {
                    event["attendees_by_return_comuna"][id].as_f64()?
                };
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
            let path = if at_stadium {
                // Each group enters at a schematic point on the outer rim. Seats are
                // distributed independently of home and mode; the engine supplies no seating plan.
                let entrance = direction.z.atan2(direction.x) / std::f32::consts::TAU;
                let rank = (boundary - n + i) as f32;
                let seat = (rank + 0.5) * 0.618034;
                GroupPath::Stadium {
                    entrance,
                    seat_turn: (seat - entrance + 0.5).rem_euclid(1.) - 0.5,
                    row: 0.32 + 0.50 * (rank * 0.414214).fract(),
                    approach: 34. + 37. * (q as f32 * 23.).fract(),
                }
            } else {
                GroupPath::Local {
                    from: center + direction * (122. + 47. * (q as f32 * 23.).fract()),
                    to: center + direction * (18. + 24. * (q as f32 * 13.).fract()),
                }
            };
            commands.spawn((
                Group {
                    venue,
                    mode,
                    path,
                    delay: (i as f32 * 0.414214 + mode as f32 * 0.15).fract() * 0.49,
                },
                Mesh3d(art.figure.clone()),
                MeshMaterial3d(art.paint[mode].clone()),
                Visibility::Hidden,
            ));
        }
    }
    let public = totals[0].round() as u64;
    let formatted = if public >= 1000 {
        format!("{}.{:03}", public / 1000, public % 1000)
    } else {
        public.to_string()
    };
    let labels = if fans {
        ["Llegada estimada", "Regreso ilustrativo"]
    } else {
        ["Llegada ilustrativa", "Regreso estimado"]
    };
    labels.map(|label| format!(
        "{label} · {formatted} en transporte público\nFlujo esquemático · 1 figura ≈ {:.0} personas",
        total / groups as f64
    ))
}

fn refresh(
    mut commands: Commands,
    scenario: Res<Scenario>,
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
        let captions = spawn_groups(
            &mut commands,
            &art,
            zones,
            row,
            center,
            playback.events.len(),
            stadium.is_some(),
        );
        let started = previous
            .iter()
            .find(|e| !replay_restarted && e.center == center && e.end == end)
            .map_or(time.elapsed_secs(), |e| e.started);
        playback.events.push(Event {
            center,
            stadium,
            end,
            started,
            captions,
            progress: None,
        });
    }
}

fn animate(
    art: Res<Art>,
    real: Res<Time<Real>>,
    fixed: Res<FixedClock>,
    mut playback: ResMut<Playback>,
    subway: Res<SubwayView>,
    rig: Res<Rig>,
    mut groups: Query<(&Group, &mut Transform, &mut Visibility, &mut Mesh3d), Without<Key>>,
    bowls: Query<(&StadiumSeating, &Transform), Without<Group>>,
    mut panel: Single<&mut Visibility, With<Key>>,
    mut caption: Single<&mut Text, With<Caption>>,
) {
    for event in &mut playback.events {
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
    for (group, mut transform, mut visibility, mut mesh) in &mut groups {
        let Some((phase, t)) = playback.events[group.venue].progress else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let time = if phase == 0 { t } else { 1. - t };
        let map_scale = (rig.distance * 0.0035).clamp(1.1, 3.8);
        let (point, heading, transit, scale) = match group.path {
            GroupPath::Stadium {
                entrance,
                seat_turn,
                row,
                approach,
            } => {
                let u = ((time - group.delay) / 0.40).min(1.);
                if u < 0. {
                    visibility.set_if_neq(Visibility::Hidden);
                    continue;
                }
                let (bowl, landmark) = bowls
                    .get(playback.events[group.venue].stadium.unwrap())
                    .unwrap();
                let (z, x) = (entrance * std::f32::consts::TAU).sin_cos();
                let outward = Vec3::new(x, 0., z);
                let rim = landmark.transform_point(bowl.terrace(entrance, 1.));
                let point = if u < 0.50 {
                    let walk = u / 0.50;
                    let mut p = rim + outward * approach * (1. - walk);
                    let ground = terrain_from_xz(p.x, p.z) + 0.3;
                    p.y = ground + (rim.y - ground) * walk.powi(4);
                    p
                } else {
                    // Follow the curved terrace, not a chord between stands. Keep
                    // each group in its seat until its turn to leave on the return.
                    let seated = (u - 0.50) * 2.;
                    landmark.transform_point(
                        bowl.terrace(entrance + seat_turn * seated, 1. + (row - 1.) * seated),
                    )
                };
                let size = if u < 0.35 {
                    map_scale
                } else {
                    map_scale.min(1.15)
                };
                (point, -outward, u < 0.35, size)
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
                (point, to - from, u < 0.56, map_scale)
            }
        };
        let glyph = if transit {
            &art.modes[group.mode]
        } else {
            &art.figure
        };
        mesh.set_if_neq(Mesh3d(glyph.clone()));
        transform.rotation = Quat::from_rotation_y(heading.x.atan2(heading.z));
        transform.translation = point;
        transform.scale = Vec3::splat(scale);
        visibility.set_if_neq(Visibility::Inherited);
    }
}
