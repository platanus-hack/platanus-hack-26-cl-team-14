//! A hypothetical event supplies totals for the whole return period. The route,
//! departure order and spacing illustrate those totals during the event replay.
use super::*;
use crate::{Rig, city::SubwayView};

// Eight event seconds per screen second. A fixed capture clock stays exact.
const REPLAY_SPEED: f32 = 8.;
const ATTENDEES_PER_FIGURE: f64 = 48.;

pub(crate) struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            refresh
                .after(crate::city::draw_buildings)
                .run_if(resource_changed::<Scenario>),
        )
        .add_systems(
            Update,
            animate
                .after(refresh)
                .run_if(resource_exists::<EventPlayback>),
        );
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Slot {
    venue: Vec3,
    bin: u8,
}

#[derive(Resource)]
struct EventPlayback(Vec<(Slot, f32)>);

#[derive(Component)]
struct Venue;

#[derive(Component)]
struct Attendee {
    path: Arc<Path>,
    bin_start: f32,
    playback_start: f32,
    delay: f32,
    car: Model,
    car_mark: Option<Entity>,
}

struct EventAssets {
    stops: Vec<Vec3>,
    ring: Handle<Mesh>,
    amber: Handle<StandardMaterial>,
}

impl EventAssets {
    fn load(meshes: &mut Assets<Mesh>, mats: &mut Assets<StandardMaterial>) -> Self {
        // Only rows that DTPM joined to a GTFS stop have mapped coordinates.
        let stops = fs::read_to_string("data/mvp/bus_activity.csv")
            .unwrap()
            .lines()
            .skip(1)
            .filter_map(|row| {
                let mut columns = row.split(',').skip(4);
                let lat = columns.next()?.parse().ok()?;
                let lon = columns.next()?.parse().ok()?;
                Some(world(lon, lat))
            })
            .collect();
        Self {
            stops,
            ring: meshes.add(Torus::new(27., 32.)),
            amber: mats.add(StandardMaterial {
                base_color: Color::srgb_u8(255, 180, 49),
                unlit: true,
                ..default()
            }),
        }
    }
}

fn refresh(
    mut commands: Commands,
    scenario: Res<Scenario>,
    time: Res<Time>,
    previous: Option<Res<EventPlayback>>,
    routes: Res<Routes>,
    models: Res<Models>,
    fleet: Res<fleet::Fleet>,
    mut rig: ResMut<Rig>,
    old: Query<Entity, Or<(With<Venue>, With<Attendee>)>>,
    mut cache: Local<Option<EventAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let result = &scenario.0;
    let events = result["events"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    if events.is_empty() {
        if previous.is_some() {
            commands.remove_resource::<EventPlayback>();
        }
        return;
    }
    let assets = cache.get_or_insert_with(|| EventAssets::load(&mut meshes, &mut mats));
    let slots: Vec<_> = events
        .iter()
        .map(|e| Slot {
            venue: world(
                e["longitude"].as_f64().unwrap(),
                e["latitude"].as_f64().unwrap(),
            ),
            bin: e["assumed_end_half_hour"].as_u64().unwrap() as u8,
        })
        .collect();
    let previous = previous
        .as_deref()
        .map(|p| p.0.as_slice())
        .unwrap_or_default();
    let focus = slots
        .iter()
        .rev()
        .find(|s| !previous.iter().any(|(old, _)| old.venue == s.venue))
        .copied();
    // Open large events mid-departure. Small events start with their first figure.
    // Adding a second venue preserves the playback at existing venues.
    let playback = EventPlayback(
        slots
            .into_iter()
            .zip(events)
            .map(|(slot, event)| {
                let lead = if event["attendance"].as_f64().unwrap() >= 96. * ATTENDEES_PER_FIGURE {
                    360.
                } else {
                    0.
                };
                let start = previous
                    .iter()
                    .find(|(old, _)| *old == slot)
                    .map_or(time.elapsed_secs() - lead / REPLAY_SPEED, |&(_, start)| {
                        start
                    });
                (slot, start)
            })
            .collect(),
    );
    let stations = result["stations"].as_array().unwrap();
    // The result already contains exactly the active new stations after line,
    // corridor and proposed-point edits. Only existing stations remain in the
    // result when closed. Their applied edits contain a canonical station name.
    let mut station_states = HashMap::new();
    for change in result["applied"].as_array().into_iter().flatten() {
        let understood = &change["understood"];
        if change["kind"] != "station" || understood.get("id").is_some() {
            continue;
        }
        let Some(name) = understood["station"].as_str() else {
            continue;
        };
        let target = change["target"]["id"].as_str().unwrap();
        let id = if routes.entrances.contains_key(target) {
            target
        } else {
            let Some(row) = stations.iter().find(|s| {
                s["name"] == name && routes.entrances.contains_key(s["id"].as_str().unwrap())
            }) else {
                continue;
            };
            row["id"].as_str().unwrap()
        };
        station_states.insert(id, understood["state"] == "open");
    }
    let open: Vec<_> = stations
        .iter()
        .filter(|s| station_states.get(s["id"].as_str().unwrap()) != Some(&false))
        .map(|s| {
            world(
                s["longitude"].as_f64().unwrap(),
                s["latitude"].as_f64().unwrap(),
            )
        })
        .collect();
    for (event, &(slot, playback_start)) in events.iter().zip(&playback.0) {
        commands.spawn((
            Venue,
            Mesh3d(assets.ring.clone()),
            MeshMaterial3d(assets.amber.clone()),
            Transform::from_translation(slot.venue + Vec3::Y * 60.),
        ));
        // Use a mapped street on the camera side of the venue for the illustrative exit.
        let offset = crate::pose(&rig).translation - rig.target;
        let street_focus = slot.venue + Vec3::new(offset.x, 0., offset.z).normalize() * 70.;
        let frontage = frontage(&routes.streets, street_focus);
        if !rig.fixed && Some(slot) == focus {
            let curb = frontage.nearest(street_focus).0;
            rig.target = slot.venue.midpoint(curb);
            rig.distance = 460_f32.max(220. + 2.5 * slot.venue.distance(curb));
        }
        spawn_crowd(
            &mut commands,
            event,
            result,
            slot,
            playback_start,
            &open,
            assets,
            &frontage,
            street_focus,
            &routes.streets,
            &models,
            &fleet,
        );
    }
    commands.insert_resource(playback);
}

fn nearest(points: &[Vec3], p: Vec3) -> Option<Vec3> {
    points
        .iter()
        .copied()
        .min_by(|a, b| a.distance_squared(p).total_cmp(&b.distance_squared(p)))
}

// Keep both actual segment endpoints for the onward road. The commuter paths
// clip their first segment, which can leave an event car visible for one frame.
fn frontage(streets: &StreetIndex, focus: Vec3) -> Arc<Path> {
    let nearest_segment = |best: Option<(f32, Vec3, Vec3)>, &(a, b): &(Vec3, Vec3)| {
        let d = foot(focus, a, b).distance_squared(focus);
        if best.is_none_or(|(old, _, _)| d < old) {
            Some((d, a, b))
        } else {
            best
        }
    };
    let (x, z) = cell(focus);
    let mut closest = None;
    for i in x - 1..=x + 1 {
        for j in z - 1..=z + 1 {
            if let Some(segments) = streets.0.get(&(i, j)) {
                closest = segments.iter().fold(closest, nearest_segment);
            }
        }
    }
    let (_, a, b) = closest
        .filter(|p| p.0 < 200_f32.powi(2))
        .unwrap_or_else(|| {
            streets
                .0
                .values()
                .flatten()
                .fold(None, nearest_segment)
                .unwrap()
        });
    Path::single(vec![a, b], Vehicle::Walk)
}

// Follow connected mapped segments in the initial direction for this local shot.
fn onward(streets: &StreetIndex, curb: Vec3, from: Vec3, to: Vec3) -> Vec<Vec3> {
    let mut points = vec![curb, to];
    let mut length = curb.distance(to);
    let mut previous = from;
    let mut at = to;
    while length < 450. {
        let heading = (at - previous).normalize();
        let next = streets
            .0
            .get(&cell(at))
            .into_iter()
            .flatten()
            .filter_map(|&(a, b)| {
                if a == at {
                    Some(b)
                } else if b == at {
                    Some(a)
                } else {
                    None
                }
            })
            .filter(|p| *p != previous && !points.contains(p))
            .max_by(|a, b| {
                ((*a - at).normalize().dot(heading))
                    .total_cmp(&((*b - at).normalize().dot(heading)))
            });
        let Some(next) = next else { break };
        length += at.distance(next);
        points.push(next);
        previous = at;
        at = next;
    }
    points
}

fn spawn_crowd(
    commands: &mut Commands,
    event: &Value,
    result: &Value,
    slot: Slot,
    playback_start: f32,
    stations: &[Vec3],
    assets: &EventAssets,
    frontage: &Path,
    street_focus: Vec3,
    streets: &StreetIndex,
    models: &Models,
    fleet: &fleet::Fleet,
) {
    let modes = &event["attendees_by_census_mode"];
    let count = |key: &str| modes[key].as_f64().unwrap_or_default();
    let public = count("2");
    let metro = event["metro_users_with_network_access"]
        .as_f64()
        .unwrap_or_default();
    let rest: f64 = modes
        .as_object()
        .unwrap()
        .iter()
        .filter(|(k, _)| *k != "1" && *k != "2")
        .map(|(_, n)| n.as_f64().unwrap())
        .sum();
    let total = count("1") + public + rest;
    if total == 0. {
        return;
    }

    // One figure per 48 modeled attendees, with one figure for a small event
    // and at most 1024 per venue. Cumulative rounding preserves the displayed total.
    let copies = (total / ATTENDEES_PER_FIGURE).round().clamp(1., 1024.) as usize;
    let boundary = |mass: f64| (mass / total * copies as f64).round() as usize;
    let metro_end = boundary(metro);
    let bus_end = boundary(public);
    let car_end = boundary(public + count("1"));
    let cars = fleet.cars(event["venue_cut"].as_str().unwrap(), 0, car_end - bus_end);
    // The engine supplies no event station or stop split. Use the nearest open station
    // and mapped bus stop. Show the other public modes walking toward the bus stop.
    let station = nearest(stations, slot.venue);
    let stop = nearest(&assets.stops, slot.venue);
    let (curb, along, _) = frontage.nearest(street_focus);
    let (a, b) = (frontage.points[0], frontage.points[1]);
    let roads = [onward(streets, curb, b, a), onward(streets, curb, a, b)];
    let side = right(along);
    let mut mass = 0.;
    let homes: Vec<_> = result["zones"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|z| {
            mass += event["attendees_by_return_comuna"][z["id"].as_str()?].as_f64()?;
            Some((
                mass,
                world(z["longitude"].as_f64()?, z["latitude"].as_f64()?),
            ))
        })
        .collect();
    for i in 0..copies {
        // The engine has no venue exits. Place the waiting crowd beside the
        // street so figures remain visible when the point lies under a roof.
        let along_offset = 65. * ((i as f32 * 0.41421357).fract() - 0.5);
        let side_offset = 2. + 4. * (i as f32 * 0.618034).fract();
        let mut start = curb + along * along_offset + side * side_offset;
        start.y = terrain_from_xz(start.x, start.z);
        let car = i >= bus_end && i < car_end;
        let path = if i < bus_end {
            let Some(end) = (if i < metro_end { station } else { stop }) else {
                continue;
            };
            Path::single(vec![start, end], Vehicle::Walk)
        } else {
            // Mode and return comuna are separate engine totals. Sample the comuna
            // distribution for the heading of the first street segment only.
            let q = ((i as f64 + 0.5) * 0.61803398875).fract() * mass;
            let home = homes[homes.partition_point(|(upper, _)| *upper <= q)].1;
            let mode = if car { Vehicle::Car } else { Vehicle::Walk };
            let side = usize::from(along.dot(home - curb) >= 0.);
            let road = if roads[side].iter().any(|p| p.distance_squared(curb) > 1.) {
                &roads[side]
            } else {
                &roads[1 - side]
            };
            let mut points = vec![start];
            points.extend_from_slice(road);
            let end = points.len() - 1;
            Path::new(
                points,
                vec![
                    Leg {
                        end: 1,
                        vehicle: Vehicle::Walk,
                        scheduled: None,
                    },
                    Leg {
                        end,
                        vehicle: mode,
                        scheduled: None,
                    },
                ],
            )
        };
        let model = if car {
            Model::Car(cars[i - bus_end])
        } else {
            Model::Person
        };
        // The same colored road marks identify changed cars in the normal movers.
        let car_mark = car.then(|| {
            commands
                .spawn((
                    Mesh3d(models.mesh[&Model::CarChange].clone()),
                    MeshMaterial3d(models.paint[&(Model::CarChange, Paint::Gain)].clone()),
                    Visibility::Hidden,
                ))
                .id()
        });
        let mut entity = commands.spawn((
            Attendee {
                path,
                bin_start: f32::from(slot.bin) * BIN,
                playback_start,
                delay: departure_offset(0., i),
                car: model,
                car_mark,
            },
            Mesh3d(models.mesh[&Model::Person].clone()),
            MeshMaterial3d(models.paint[&(Model::Person, Paint::Gain)].clone()),
            Transform::from_translation(start),
            Visibility::Hidden,
        ));
        if let Some(mark) = car_mark {
            entity.add_child(mark);
        }
    }
}

fn animate(
    time: Res<Time>,
    clock: Res<FixedClock>,
    event: Res<EventPlayback>,
    subway: Res<SubwayView>,
    models: Res<Models>,
    rig: Res<Rig>,
    mut next: Local<f32>,
    mut marks: Query<&mut Visibility, (Without<Attendee>, Without<Venue>)>,
    mut venues: Query<&mut Visibility, (With<Venue>, Without<Attendee>)>,
    mut people: Query<(
        &Attendee,
        &mut Transform,
        &mut Visibility,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    if time.elapsed_secs() < *next && !event.is_changed() && !subway.is_changed() {
        return;
    }
    *next = time.elapsed_secs() + 1. / 30.;
    for mut visible in &mut venues {
        visible.set_if_neq(if subway.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    for (person, mut transform, mut visible, mut mesh, mut material) in &mut people {
        let age = clock.0.map_or(
            (time.elapsed_secs() - person.playback_start) * REPLAY_SPEED,
            |now| now - person.bin_start,
        );
        let position = if subway.0 || age < 0. || age < person.delay - 90. {
            None
        } else if age < person.delay {
            Some((person.path.points[0], Vec3::X, Vehicle::Walk))
        } else {
            person.path.at(age - person.delay, &person.path.seconds)
        };
        let Some((point, direction, mode)) = position else {
            visible.set_if_neq(Visibility::Hidden);
            continue;
        };
        let (model, paint) = if mode == Vehicle::Car {
            (person.car, Paint::Plain)
        } else {
            (Model::Person, Paint::Gain)
        };
        if let Some(mark) = person.car_mark {
            marks
                .get_mut(mark)
                .unwrap()
                .set_if_neq(if mode == Vehicle::Car {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
        }
        mesh.set_if_neq(Mesh3d(models.mesh[&model].clone()));
        material.set_if_neq(MeshMaterial3d(models.paint[&(model, paint)].clone()));
        transform.set_if_neq(pose(
            point
                + Vec3::Y * 0.25
                + if mode == Vehicle::Car {
                    right(direction) * 1.6
                } else {
                    Vec3::ZERO
                },
            direction,
            Vec3::splat((rig.distance * 0.004).clamp(1.8, 11.)),
        ));
        visible.set_if_neq(Visibility::Inherited);
    }
}
