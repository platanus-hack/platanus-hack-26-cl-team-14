use super::{Buildings, NamedBuilding, SubwayView, Surface, ground::ribbon, world};
use super::{ground::ribbon_faces, street_mask::StreetMask, streets};
use crate::net::Scenario;
use bevy::prelude::*;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Component)]
pub(crate) struct EditMesh;

#[derive(Component)]
pub(crate) struct SurfaceEdit;

#[derive(Component)]
pub(crate) struct FerryRoute;

// The wide replay introduces the new service with a route line. At the pier,
// the vessel, gangway and passengers identify the intervention at street scale.
pub(crate) fn ferry_routes(
    rig: Res<crate::Rig>,
    mode: Res<SubwayView>,
    mut lines: Query<&mut Visibility, With<FerryRoute>>,
) {
    let visible = if mode.0 || rig.distance < 1_200. {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut line in &mut lines {
        line.set_if_neq(visible);
    }
}

#[derive(Component)]
pub(crate) struct ClosureEnd(Vec3);

fn closure_ends(commands: &mut Commands, geometry: &Value, mask: &StreetMask) {
    let lines = &geometry["coordinates"];
    fn ends(line: &Value) -> [&Value; 2] {
        let points = line.as_array().unwrap();
        [points.first().unwrap(), points.last().unwrap()]
    }
    let mut endpoints = Vec::new();
    if geometry["type"] == "MultiLineString" {
        for line in lines.as_array().unwrap() {
            endpoints.extend(ends(line));
        }
    } else {
        endpoints.extend(ends(lines));
    }
    // Shared endpoints join parts of the same feature. Mark its exposed ends.
    for point in &endpoints {
        if endpoints.iter().filter(|p| **p == *point).count() != 1 {
            continue;
        }
        let position = world(point[0].as_f64().unwrap(), point[1].as_f64().unwrap());
        if mask.contains(position) {
            continue;
        }
        commands.spawn((
            EditMesh,
            ClosureEnd(position),
            Node {
                position_type: PositionType::Absolute,
                width: px(16),
                height: px(16),
                border_radius: BorderRadius::MAX,
                border: UiRect::all(px(1)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(Color::srgb_u8(164, 53, 47)),
            Visibility::Hidden,
            children![(
                Node { width: px(10), height: px(3), ..default() },
                BackgroundColor(Color::WHITE),
            )],
        ));
    }
}

pub(crate) fn place_closure_ends(
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
    canvas: Query<&Sprite, With<crate::pixels::CityCanvas>>,
    mode: Res<SubwayView>,
    mut ends: Query<(&ClosureEnd, &mut Node, &mut Visibility)>,
) {
    let (camera, transform) = *camera;
    for (end, mut node, mut visible) in &mut ends {
        let Ok(mut p) = camera.world_to_viewport(transform, end.0) else {
            *visible = Visibility::Hidden;
            continue;
        };
        if let Ok(sprite) = canvas.single()
            && let (Some(size), Some(viewport)) = (sprite.custom_size, camera.logical_viewport_size())
        {
            p = window.size() * 0.5 + (p / viewport - Vec2::splat(0.5)) * size;
        }
        node.left = px(p.x - 8.);
        node.top = px(p.y - 8.);
        *visible = if mode.0 { Visibility::Hidden } else { Visibility::Inherited };
    }
}

fn rings(value: &Value) -> Vec<Vec<Vec3>> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|ring| {
            let mut points: Vec<_> = ring
                .as_array()
                .unwrap()
                .iter()
                .map(|pair| world(pair[0].as_f64().unwrap(), pair[1].as_f64().unwrap()))
                .collect();
            points.pop();
            points
        })
        .collect()
}

fn draw(feature: &Value, kind: &str, removed: bool, mask: &StreetMask) -> Surface {
    let mut mesh = Surface::default();
    let geometry = &feature["geometry"];
    // The transparent pass lays engine edits over the printed city.
    let color = [0.85, 0.13, 0.09, 1.];
    match kind {
        "building" => {
            let footprint = rings(&geometry["coordinates"]);
            if removed && mask.drape_if_cut(&mut mesh, &footprint, 0.3, color) {
                return mesh;
            }
            let height = if removed {
                0.
            } else {
                feature["properties"]["height_m"].as_f64().unwrap_or(0.) as f32
            };
            mesh.polygon(&footprint, 0.3, height, color, color);
        }
        "road" | "line" => {
            let lines = if geometry["type"] == "MultiLineString" {
                geometry["coordinates"].as_array().unwrap().clone()
            } else {
                vec![geometry["coordinates"].clone()]
            };
            for line in lines {
                let points: Vec<_> = line
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|pair| world(pair[0].as_f64().unwrap(), pair[1].as_f64().unwrap()))
                    .collect();
                let points = if kind == "line" && feature["properties"]["mode"] == "boat" {
                    let water = super::water::route(points.iter().map(|p| p.xz()).collect());
                    let Some(water) = water else { continue };
                    water
                        .into_iter()
                        .map(|p| Vec3::new(p.x, super::terrain_from_xz(p.x, p.y), p.y))
                        .collect()
                } else {
                    points
                };
                let lift = if removed { 0.69 } else { 0.82 };
                if kind == "road" {
                    let width = if removed { 4. } else { 5. };
                    ribbon_faces(&points, width, |ring| {
                        mask.drape(&mut mesh, &[ring], lift, color)
                    });
                } else {
                    ribbon(&mut mesh, &points, if removed { 2. } else { 3. }, lift, color);
                }
            }
        }
        _ => {}
    }
    mesh
}

// The detailed mesh has local heights but uses the city's horizontal coordinates.
// Reuse it only when every footprint ring has the same horizontal displacement.
fn landmark_offset(before: &Value, after: &Value) -> Option<Vec3> {
    if before.is_null()
        || after.is_null()
        || before["properties"]["height_m"] != after["properties"]["height_m"]
    {
        return None;
    }
    let source = rings(&before["geometry"]["coordinates"]);
    let target = rings(&after["geometry"]["coordinates"]);
    if source.len() != target.len() {
        return None;
    }
    let anchor = |ring: &[Vec3]| {
        let center = ring.iter().copied().sum::<Vec3>() / ring.len() as f32;
        Vec3::new(
            center.x,
            super::terrain_from_xz(center.x, center.z),
            center.z,
        )
    };
    let offset = anchor(target.first()?) - anchor(source.first()?);
    // A closed ring may start at any vertex and run in either direction.
    let same_ring = |a: &[Vec3], b: &[Vec3]| {
        let close = |i: usize, j: usize| (b[j] - a[i] - offset).xz().length_squared() < 0.01;
        let n = a.len();
        n == b.len()
            && (0..n).filter(|&start| close(0, start)).any(|start| {
                (0..n).all(|i| close(i, (start + i) % n))
                    || (0..n).all(|i| close(i, (start + n - i) % n))
            })
    };
    (same_ring(&source[0], &target[0])
        && source[1..]
            .iter()
            .all(|hole| target[1..].iter().any(|other| same_ring(hole, other))))
    .then_some(offset)
}

// Every result is a full scenario. Keep only the last state of each edited object.
pub(crate) fn refresh(
    mut commands: Commands,
    scenario: Res<Scenario>,
    mut buildings: ResMut<Buildings>,
    mode: Res<SubwayView>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    old: Query<Entity, With<EditMesh>>,
    mut named: Query<(Entity, &NamedBuilding, Option<&super::StadiumSeating>, &mut Transform, &mut Visibility)>,
    mut road_tiles: Query<(&streets::StreetTile, &mut Mesh3d)>,
    mut previous_mask: Local<StreetMask>,
    mut origins: Local<HashMap<Entity, Vec3>>,
    mut material: Local<Option<Handle<StandardMaterial>>>,
) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let mut final_state = BTreeMap::<(String, String), (Value, Value)>::new();
    let mut requested_moves = HashSet::new();
    for edit in scenario.0["city_edits"].as_array().into_iter().flatten() {
        let (Some(kind), Some(id)) = (edit["kind"].as_str(), edit["id"].as_str()) else {
            continue;
        };
        if kind == "building" && edit["op"] == "move" {
            requested_moves.insert(id);
        }
        final_state
            .entry((kind.into(), id.into()))
            .and_modify(|state| state.1 = edit["after"].clone())
            .or_insert_with(|| (edit["before"].clone(), edit["after"].clone()));
    }
    let hidden: HashSet<_> = final_state
        .iter()
        .filter_map(|((kind, id), (before, _))| {
            if kind != "building" || before.is_null() {
                return None;
            }
            let pair = id.strip_prefix("mesh:")?.split_once(':')?;
            Some((pair.0.parse::<usize>().ok()?, pair.1.parse::<usize>().ok()?))
        })
        .collect();
    let changed: Vec<_> = buildings
        .hidden
        .symmetric_difference(&hidden)
        .copied()
        .collect();
    for (tile, _) in changed {
        buildings.dirty[tile] = true;
    }
    buildings.hidden = hidden;
    let mut moved_landmarks = HashMap::new();
    for (entity, key, seating, mut transform, mut visibility) in &mut named {
        let original = *origins.entry(entity).or_insert(transform.translation);
        let id = format!("mesh:{}:{}", key.0, key.1);
        let offset = final_state
            .get(&("building".to_owned(), id.clone()))
            .filter(|_| requested_moves.contains(id.as_str()))
            .and_then(|(before, after)| landmark_offset(before, after));
        transform.translation = original + offset.unwrap_or_default();
        *visibility = if offset.is_some() {
            moved_landmarks.insert(id, seating.is_some());
            Visibility::Inherited
        } else if buildings.hidden.contains(&(key.0, key.1)) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    let mask = StreetMask::from_footprints(final_state.iter().filter_map(|((kind, id), (before, after))| {
        if kind != "building" || after.is_null() || before["geometry"] == after["geometry"] {
            return None;
        }
        let mut footprint = rings(&after["geometry"]["coordinates"]);
        // The moved stadium's interior ring opens onto its pitch, which also occupies the site.
        if moved_landmarks.get(id) == Some(&true) {
            footprint.truncate(1);
        }
        Some(footprint)
    }));
    if mask != *previous_mask {
        streets::apply(&mask, &mut meshes, &mut road_tiles);
        *previous_mask = mask;
    }
    let mask = &*previous_mask;
    let path_material = material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                cull_mode: None,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })
        })
        .clone();
    for ((kind, id), (before, after)) in final_state {
        if !before.is_null()
            && (after.is_null()
                || before["geometry"] != after["geometry"]
                || (kind == "line" && before["properties"]["mode"] != after["properties"]["mode"]))
        {
            if kind == "road" && after.is_null() {
                closure_ends(&mut commands, &before["geometry"], mask);
            }
            let shape = draw(&before, &kind, true, mask);
            if !shape.indices.is_empty() {
                let mut entity = commands.spawn((
                    EditMesh,
                    Mesh3d(meshes.add(shape.finish())),
                    MeshMaterial3d(path_material.clone()),
                ));
                if kind == "road" || before["properties"]["mode"] != "metro" {
                    entity.insert((
                        SurfaceEdit,
                        if mode.0 {
                            Visibility::Hidden
                        } else {
                            Visibility::Inherited
                        },
                    ));
                }
            }
        }
        if !after.is_null() && !(kind == "building" && moved_landmarks.contains_key(&id)) {
            let shape = draw(&after, &kind, false, mask);
            if !shape.indices.is_empty() {
                let mut entity = commands.spawn((
                    EditMesh,
                    Mesh3d(meshes.add(shape.finish())),
                    MeshMaterial3d(if kind == "building" {
                        buildings.edit_material.clone()
                    } else {
                        path_material.clone()
                    }),
                ));
                if kind == "line" && after["properties"]["mode"] == "boat" {
                    entity.insert(FerryRoute);
                }
                if kind == "road" || (kind == "line" && after["properties"]["mode"] != "metro") {
                    entity.insert((
                        SurfaceEdit,
                        if mode.0 {
                            Visibility::Hidden
                        } else {
                            Visibility::Inherited
                        },
                    ));
                }
            }
        }
    }
}
