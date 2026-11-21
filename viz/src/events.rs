//! A hypothetical event supplies totals for the whole return period. The route,
//! departure order and spacing illustrate those totals during the event replay.
use super::*;
use crate::{Rig, city::SubwayView};

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
            animate.after(refresh).run_if(resource_exists::<EventClock>),
        );
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Slot {
    venue: Vec3,
    bin: u8,
}

#[derive(Resource)]
struct EventClock {
    slots: Vec<Slot>,
    anchor: Slot,
    offset: f32,
}

#[derive(Component)]
struct Venue;

#[derive(Component)]
struct Attendee {
    path: Arc<Path>,
    bin_start: f32,
    delay: f32,
    car: Model,
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
    previous: Option<Res<EventClock>>,
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
            commands.remove_resource::<EventClock>();
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
    let previous = previous.as_deref();
    let focus = slots
        .iter()
        .rev()
        .find(|s| !previous.is_some_and(|p| p.slots.iter().any(|old| old.venue == s.venue)))
        .copied();
    let anchor = slots
        .iter()
        .rev()
        .find(|s| !previous.is_some_and(|p| p.slots.contains(s)))
        .copied()
        .or_else(|| previous.map(|p| p.anchor).filter(|s| slots.contains(s)))
        .unwrap_or(slots[0]);
    // Start the event replay one minute into its end bin. Scheduled city traffic keeps its own time.
    let offset = previous.filter(|p| p.anchor == anchor).map_or_else(
        || f32::from(anchor.bin) * BIN + 60. - time.elapsed_secs(),
        |p| p.offset,
    );
    let mut station_states = HashMap::new();
    for change in result["applied"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| c["kind"] == "station")
    {
        let station = &change["understood"];
        station_states.insert(
            station["station"].as_str().unwrap(),
            station["state"] == "open",
        );
    }
    let open: Vec<_> = result["stations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| station_states.get(s["name"].as_str().unwrap()) != Some(&false))
        .map(|s| {
            world(
                s["longitude"].as_f64().unwrap(),
                s["latitude"].as_f64().unwrap(),
            )
        })
        .collect();
    for (event, &slot) in events.iter().zip(&slots) {
        commands.spawn((
            Venue,
            Mesh3d(assets.ring.clone()),
            MeshMaterial3d(assets.amber.clone()),
            Transform::from_translation(slot.venue + Vec3::Y * 60.),
        ));
        let frontage = frontage(&routes, slot.venue);
        if !rig.fixed && Some(slot) == focus {
            let curb = frontage.nearest(slot.venue).0;
            rig.target = slot.venue.midpoint(curb);
            rig.distance = 720_f32.max(200. + 2.5 * slot.venue.distance(curb));
        }
        spawn_crowd(
            &mut commands,
            event,
            result,
            slot,
            &open,
            assets,
            &frontage,
            &models,
            &fleet,
        );
    }
    commands.insert_resource(EventClock {
        slots,
        anchor,
        offset,
    });
}

fn nearest(points: &[Vec3], p: Vec3) -> Option<Vec3> {
    points
        .iter()
        .copied()
        .min_by(|a, b| a.distance_squared(p).total_cmp(&b.distance_squared(p)))
}

// Use the nearest mapped frontage when a roof or a field covers the event point.
fn frontage(routes: &Routes, venue: Vec3) -> Arc<Path> {
    if let Some(path) = routes
        .streets
        .local_path(venue, venue + Vec3::X, Vehicle::Walk)
    {
        return path;
    }
    let (a, b) = routes
        .streets
        .0
        .values()
        .flatten()
        .copied()
        .min_by(|&(a, b), &(c, d)| {
            foot(venue, a, b)
                .distance_squared(venue)
                .total_cmp(&foot(venue, c, d).distance_squared(venue))
                .then(a.x.total_cmp(&c.x))
                .then(a.z.total_cmp(&c.z))
        })
        .unwrap();
    Path::single(vec![a, b], Vehicle::Walk)
}

fn spawn_crowd(
    commands: &mut Commands,
    event: &Value,
    result: &Value,
    slot: Slot,
    stations: &[Vec3],
    assets: &EventAssets,
    frontage: &Path,
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

    // One figure per 96 modeled attendees, with one figure for a small event
    // and at most 512 per venue. Cumulative rounding preserves the displayed total.
    let copies = (total / PERSONS_PER_MOVER).round().clamp(1., 512.) as usize;
    let boundary = |mass: f64| (mass / total * copies as f64).round() as usize;
    let metro_end = boundary(metro);
    let bus_end = boundary(public);
    let car_end = boundary(public + count("1"));
    let cars = fleet.cars(event["venue_cut"].as_str().unwrap(), 0, car_end - bus_end);
    // The engine supplies no event station or stop split. Use the nearest open station
    // and mapped bus stop. Show the other public modes walking toward the bus stop.
    let station = nearest(stations, slot.venue);
    let stop = nearest(&assets.stops, slot.venue);
    let (curb, along, _) = frontage.nearest(slot.venue);
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
        let along_offset = 50. * ((i as f32 + 0.5) / copies as f32 - 0.5);
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
            let [a, b] = [frontage.points[0], *frontage.points.last().unwrap()];
            let mut end = if along.dot(home - curb) < 0. { a } else { b };
            if end.distance_squared(curb) < 1. {
                end = if end == a { b } else { a };
            }
            Path::new(
                vec![start, curb, end],
                vec![
                    Leg {
                        end: 1,
                        vehicle: Vehicle::Walk,
                        scheduled: None,
                    },
                    Leg {
                        end: 2,
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
        commands.spawn((
            Attendee {
                path,
                bin_start: f32::from(slot.bin) * BIN,
                delay: departure_offset(0., i),
                car: model,
            },
            Mesh3d(models.mesh[&Model::Person].clone()),
            MeshMaterial3d(models.paint[&(Model::Person, Paint::Gain)].clone()),
            Transform::from_translation(start),
            Visibility::Hidden,
        ));
    }
}

fn animate(
    time: Res<Time>,
    clock: Res<FixedClock>,
    event: Res<EventClock>,
    subway: Res<SubwayView>,
    models: Res<Models>,
    rig: Res<Rig>,
    mut next: Local<f32>,
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
    let now = clock.0.unwrap_or(event.offset + time.elapsed_secs());
    for mut visible in &mut venues {
        visible.set_if_neq(if subway.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    for (person, mut transform, mut visible, mut mesh, mut material) in &mut people {
        let age = now - person.bin_start;
        let position = if subway.0 || age < 0. {
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
        mesh.set_if_neq(Mesh3d(models.mesh[&model].clone()));
        material.set_if_neq(MeshMaterial3d(models.paint[&(model, paint)].clone()));
        transform.set_if_neq(pose(
            point + Vec3::Y * 0.25,
            direction,
            Vec3::splat((rig.distance * 0.0024).clamp(1.35, 11.)),
        ));
        visible.set_if_neq(Visibility::Inherited);
    }
}
