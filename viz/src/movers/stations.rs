use super::*;

#[derive(Clone)]
pub(super) struct Entrance {
    pub(super) gtfs: Option<Box<str>>,
    pub(super) position: Vec3,
    sign: Vec3,
    outward: Vec3,
}

impl Entrance {
    pub(super) fn arrival_porch(&self) -> (Vec3, Vec3) { (self.sign, self.outward) }
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
                        gtfs: r["gtfs"].as_str().map(Into::into),
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
pub(super) struct StationPin {
    centre: Vec3,
    name: bool,
}

pub(super) fn spawn_name(commands: &mut Commands, row: &Value) {
    let name = row["name"].as_str().unwrap();
    let centre = world(row["longitude"].as_f64().unwrap(), row["latitude"].as_f64().unwrap());
    commands.spawn((
        StationPin { centre, name: true },
        Text::new(name),
        TextFont { font_size: bevy::text::FontSize::Px(21.), ..default() },
        TextColor(Color::srgb_u8(51, 45, 64)),
        BackgroundColor(Color::srgba(0.99, 0.98, 0.96, 0.94)),
        Node { position_type: PositionType::Absolute, padding: UiRect::axes(px(5), px(1)), ..default() },
        Visibility::Hidden,
    ));
}

const SIGN_SIZE: f32 = 40.;

pub(super) fn spawn_closed_sign(commands: &mut Commands, station: &Value) {
    let centre = world(station["longitude"].as_f64().unwrap(), station["latitude"].as_f64().unwrap());
    commands.spawn((
        StationPin { centre, name: false },
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
    rig: Res<crate::Rig>,
    mut signs: Query<(&StationPin, &mut Node, &mut Visibility)>,
) {
    let (camera, transform) = *camera;
    for (sign, mut node, mut visibility) in &mut signs {
        if (!sign.name || (350. ..=2200.).contains(&rig.distance))
            && let Ok(mut point) = camera.world_to_viewport(transform, sign.centre) {
            // Pixel mode centres and scales the 3D target on a window-sized UI camera.
            if let Ok(sprite) = canvas.single()
                && let (Some(size), Some(viewport)) = (sprite.custom_size, camera.logical_viewport_size())
            {
                point = window.size() * 0.5 + (point / viewport - Vec2::splat(0.5)) * size;
            }
            let offset = if sign.name { Vec2::new(-32., -51.) } else { Vec2::splat(-SIGN_SIZE / 2.) };
            node.left = px(point.x + offset.x);
            node.top = px(point.y + offset.y);
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub(super) struct Station {
    street_people: street_people::AccessFlows,
    centre: Vec3,
    entrances: Vec<Entrance>,
    radius: f32,
    sides: Vec<Side>,
}

struct Side {
    base: Vec3,
    dir: Vec3,
}

impl Station {
    pub(super) fn new(row: &Value, base_origins: Option<&Value>, routes: &Routes) -> Self {
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

        Self {
            street_people: street_people::AccessFlows::new(row, base_origins, routes),
            centre,
            entrances,
            radius,
            sides,
        }
    }

    pub(super) fn draw(
        &self,
        now: f32,
        subway: bool,
        eye: Vec3,
        view_distance: f32,
        visible: &impl Fn(Vec3, f32) -> bool,
        place: &mut impl FnMut(Model, Paint, Transform),
    ) {
        self.street_people.draw(now, eye, view_distance, visible, place);
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
        if subway && let Some(side) = self.sides.first() {
            self.street_people.draw_station_entries(now, side.base + right(side.dir) * 23., place);
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
    }
}
