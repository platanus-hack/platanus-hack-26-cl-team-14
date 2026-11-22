//! Geometry for accepted new Metro corridors and terminal extensions.
use super::*;

const METRO_INK: [f32; 4] = [0.035, 0.16, 0.65, 1.];

#[derive(Default)]
pub(super) struct Corridors {
    paths: BTreeMap<Vec<String>, Vec<[f64; 2]>>,
    stops: BTreeMap<String, [f64; 2]>,
}

fn coordinates(feature: &Value) -> Option<Vec<[f64; 2]>> {
    let geometry = &feature["geometry"];
    if geometry["type"] != "LineString" {
        return None;
    }
    serde_json::from_value(geometry["coordinates"].clone()).ok()
}

fn point([lon, lat]: [f64; 2]) -> Vec3 {
    world(lon, lat)
}

fn terminal_index(path: &[[f64; 2]], terminal: [f64; 2]) -> Option<usize> {
    let terminal = point(terminal).xz();
    path.iter()
        .enumerate()
        .map(|(i, &p)| (i, point(p).xz().distance_squared(terminal)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

impl Corridors {
    pub(super) fn add(&mut self, before: &Value, after: &Value) -> bool {
        if after["properties"]["mode"] != "metro" {
            return false;
        }
        let Some(mut path) = coordinates(after) else {
            return false;
        };
        let properties = &after["properties"];
        let Some(mut stops) = properties["stop_ids"].as_array().map(Vec::as_slice) else {
            return false;
        };
        let new_stops = &properties["new_stops"];

        if !before.is_null() {
            let Some(old_stops) = before["properties"]["stop_ids"].as_array() else {
                return false;
            };
            let Some(old_path) = coordinates(before) else {
                return false;
            };
            if old_stops.is_empty() || stops.len() <= old_stops.len() {
                return false;
            }
            // The engine publishes the whole service. Display only the added tail.
            let is_new = |stop: &Value| stop.as_str().is_some_and(|id| !new_stops[id].is_null());
            if stops.starts_with(old_stops) && stops.last().is_some_and(is_new) {
                let Some(join) = old_path.last().and_then(|&p| terminal_index(&path, p)) else {
                    return false;
                };
                path.drain(..join);
                stops = &stops[old_stops.len() - 1..];
            } else if stops.ends_with(old_stops) && stops.first().is_some_and(is_new) {
                let Some(join) = old_path.first().and_then(|&p| terminal_index(&path, p)) else {
                    return false;
                };
                path.truncate(join + 1);
                stops = &stops[..stops.len() - old_stops.len() + 1];
            } else {
                return false;
            }
        }
        if path.len() < 2 {
            return false;
        }

        // Existing tracks may end at different coordinates in each direction.
        // The ordered stop IDs identify their single schematic corridor.
        let forward: Vec<_> = stops
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        let reverse = forward.iter().rev().cloned().collect::<Vec<_>>();
        self.paths.entry(forward.min(reverse)).or_insert(path);
        for id in stops.iter().filter_map(Value::as_str) {
            if let Ok(position) = serde_json::from_value(new_stops[id].clone()) {
                self.stops.entry(id.to_owned()).or_insert(position);
            }
        }
        true
    }

    pub(super) fn spawn(
        self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        material: Handle<StandardMaterial>,
    ) {
        if self.paths.is_empty() {
            return;
        }
        let mut mesh = Surface::default();
        for path in self.paths.into_values() {
            let points: Vec<_> = path.into_iter().map(point).collect();
            ribbon(&mut mesh, &points, 8., 1.3, METRO_INK);
        }
        commands.spawn((
            EditMesh,
            MetroCorridor,
            Mesh3d(meshes.add(mesh.finish())),
            MeshMaterial3d(material),
        ));

        for coordinates in self.stops.into_values() {
            let position = point(coordinates);
            commands.spawn((
                EditMesh,
                MapPin {
                    position,
                    kind: PinKind::ProposedStation,
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: px(12.),
                    height: px(12.),
                    border_radius: BorderRadius::MAX,
                    border: UiRect::all(px(2.)),
                    ..default()
                },
                BorderColor::all(Color::srgb_u8(35, 72, 160)),
                BackgroundColor(Color::WHITE),
                Visibility::Hidden,
            ));
            // The accepted stop point is a schematic surface access, with no mapped entrance.
            commands.spawn((
                EditMesh,
                MapPin {
                    position,
                    kind: PinKind::ProposedAccess,
                },
                Text::new("M  Acceso propuesto"),
                TextFont {
                    font_size: FontSize::Px(16.),
                    ..default()
                },
                TextColor(Color::WHITE),
                BackgroundColor(Color::srgb_u8(35, 72, 160)),
                Node {
                    position_type: PositionType::Absolute,
                    padding: UiRect::axes(px(6.), px(3.)),
                    border_radius: BorderRadius::all(px(4.)),
                    ..default()
                },
                Visibility::Hidden,
            ));
        }
    }
}
