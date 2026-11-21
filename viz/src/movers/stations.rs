use super::*;

#[derive(Clone, Copy)]
pub(super) struct Entrance {
    pub(super) position: Vec3,
    pub(super) sign: Vec3,
    pub(super) outward: Vec3,
}

pub(super) type Entrances = HashMap<String, Vec<Entrance>>;

pub(super) fn entrances() -> Entrances {
    gzip_json("web/data/metro_entrances.json.gz")["stations"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(id, rows)| {
            let gates = rows
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    let position = point(&r["position"]);
                    // Keep the surveyed access point for routes. A short cartographic porch
                    // puts the ground-level marker outside the mapped facade.
                    let (sign, outward) = r.get("porch").map_or((position, Vec3::X), |porch| {
                        let dir = &porch["outward"];
                        (
                            point(&porch["position"]),
                            Vec3::new(
                                dir[0].as_f64().unwrap() as f32,
                                0.,
                                -dir[1].as_f64().unwrap() as f32,
                            ),
                        )
                    });
                    Entrance {
                        position,
                        sign,
                        outward,
                    }
                })
                .collect();
            (id.clone(), gates)
        })
        .collect()
}

// A screen-sized no-entry sign stays anchored to the closed station even behind a tower.
#[derive(Component)]
pub(super) struct ClosedSign(Vec3);

const SIGN_SIZE: f32 = 40.;

pub(super) fn spawn_closed_sign(commands: &mut Commands, station: &Value) {
    let centre = world(station["longitude"].as_f64().unwrap(), station["latitude"].as_f64().unwrap());
    commands.spawn((
        ClosedSign(centre),
        Node {
            position_type: PositionType::Absolute,
            width: px(SIGN_SIZE),
            height: px(SIGN_SIZE),
            border_radius: BorderRadius::MAX,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb_u8(198, 34, 44)),
        Visibility::Hidden,
        children![(
            Node { width: px(26), height: px(7), ..default() },
            BackgroundColor(Color::WHITE),
        )],
    ));
}

pub(super) fn position_closed_signs(
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
    canvas: Query<&Sprite, With<crate::pixels::CityCanvas>>,
    mut signs: Query<(&ClosedSign, &mut Node, &mut Visibility)>,
) {
    let (camera, transform) = *camera;
    for (sign, mut node, mut visibility) in &mut signs {
        if let Ok(mut point) = camera.world_to_viewport(transform, sign.0) {
            // Pixel mode centres and scales the 3D target on a window-sized UI camera.
            if let Ok(sprite) = canvas.single()
                && let (Some(size), Some(viewport)) = (sprite.custom_size, camera.logical_viewport_size())
            {
                point = window.size() * 0.5 + (point / viewport - Vec2::splat(0.5)) * size;
            }
            node.left = px(point.x - SIGN_SIZE / 2.);
            node.top = px(point.y - SIGN_SIZE / 2.);
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub(super) struct Station {
    street_people: street_people::StreetPeople,
    centre: Vec3,
    entrances: Vec<Entrance>,
    radius: f32,
    sides: Vec<Side>,
    // Cumulative engine boardings at the start of each half hour, plus the daily total.
    entries: Vec<f64>,
}

struct Side {
    base: Vec3,
    dir: Vec3,
}

impl Station {
    pub(super) fn new(row: &Value, routes: &Routes) -> Self {
        let id = row["id"].as_str().unwrap();
        let centre = world(
            row["longitude"].as_f64().unwrap(),
            row["latitude"].as_f64().unwrap(),
        );
        let entrances = routes.entrances[id].clone();
        let radius = entrances
            .iter()
            .map(|e| e.sign.distance(centre) + 55.)
            .fold(180., f32::max);
        let mut sides = Vec::new();
        for line in &routes.tracks {
            let Some(i) = line.stations.iter().position(|s| s == id) else {
                continue;
            };
            let Some(distance) = line.stops[i] else { continue };
            let (rail, dir) = line.coach(distance);
            sides.push(Side {
                base: rail + right(dir) * 4.9,
                dir,
            });
        }

        let mut entries = vec![0.];
        for people in row["scenario_half_hour"].as_array().unwrap() {
            entries.push(entries.last().unwrap() + people.as_f64().unwrap());
        }
        Self {
            street_people: street_people::StreetPeople::new(row, routes),
            centre,
            entrances,
            radius,
            sides,
            entries,
        }
    }

    pub(super) fn draw(
        &self,
        now: f32,
        subway: bool,
        eye: Vec3,
        visible: &impl Fn(Vec3, f32) -> bool,
        place: &mut impl FnMut(Model, Paint, Transform),
    ) {
        let drew_arrivals = self.street_people.draw(now, eye, visible, place);
        if !visible(self.centre, self.radius) {
            return;
        }
        for entrance in &self.entrances {
            place(
                Model::Entrance,
                Paint::Plain,
                pose(entrance.sign, -entrance.outward, Vec3::ONE),
            );
        }
        for side in &self.sides {
            if !subway && side.base.y < terrain_from_xz(side.base.x, side.base.z) - 0.6 {
                continue;
            }
            place(
                Model::Platform,
                Paint::Plain,
                pose(side.base, side.dir, Vec3::new(58., 1., 4.2)),
            );
        }
        if drew_arrivals {
            return;
        }
        // Close to an entrance, show one person per modeled boarding. The overview
        // samples those same boardings. The engine does not identify individual transfers.
        let distance = eye.distance(self.centre);
        let scale = if distance <= 2_000. { 1. } else { PERSONS_PER_MOVER };
        let person_size = (distance * 0.003).clamp(1.8, 3.6);
        let speed = Vehicle::Walk.speed();
        let approach = 18. / speed;
        let bin = (now / BIN) as usize;
        for slot in bin..=((now + approach) / BIN) as usize {
            let i = slot % 48;
            let total = self.entries[i + 1] - self.entries[i];
            if total <= 0. {
                continue;
            }
            let start = slot as f32 * BIN;
            let first_index = |time: f32| {
                let fraction = ((time - start) / BIN).clamp(0., 1.) as f64;
                ((self.entries[i] + total * fraction) / scale - 0.5).ceil() as usize
            };
            for j in first_index(now)..first_index(now + approach) {
                let fraction = ((j as f64 + 0.5) * scale - self.entries[i]) / total;
                let remaining = (start + fraction as f32 * BIN - now) * speed;
                // DTPM measures the station total. Alternate between its mapped gates.
                let gate = &self.entrances[j % self.entrances.len()];
                // Leave one shoulder width between lanes at the 3.2 m entrance.
                let lane = ((j / self.entrances.len() % 3) as f32 - 1.) * 1.35;
                let p = gate.sign + gate.outward * remaining + right(gate.outward) * lane;
                place(Model::Person, Paint::Plain, pose(p, -gate.outward, Vec3::splat(person_size)));
            }
        }
    }
}
