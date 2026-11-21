use super::{Buildings, NamedBuilding, SubwayView, Surface, ground::ribbon, world};
use crate::net::Scenario;
use bevy::prelude::*;
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

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

fn closure_ends(commands: &mut Commands, geometry: &Value) {
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
        commands.spawn((
            EditMesh,
            ClosureEnd(world(point[0].as_f64().unwrap(), point[1].as_f64().unwrap())),
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

fn draw(feature: &Value, kind: &str, removed: bool) -> Surface {
    let mut mesh = Surface::default();
    let geometry = &feature["geometry"];
    // The transparent pass lays engine edits over the printed city.
    let color = [0.85, 0.13, 0.09, 1.];
    match kind {
        "building" => mesh.polygon(
            &rings(&geometry["coordinates"]),
            0.3,
            if removed {
                0.
            } else {
                feature["properties"]["height_m"].as_f64().unwrap_or(0.) as f32
            },
            color,
            color,
        ),
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
                ribbon(
                    &mut mesh,
                    &points,
                    match (kind, removed) {
                        ("road", true) => 4.,
                        ("road", false) => 5.,
                        (_, true) => 2.,
                        _ => 3.,
                    },
                    if removed { 0.69 } else { 0.82 },
                    color,
                );
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
    mut named: Query<(Entity, &NamedBuilding, &mut Transform, &mut Visibility)>,
    mut origins: Local<std::collections::HashMap<Entity, Vec3>>,
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
    let mut moved_landmarks = HashSet::new();
    for (entity, key, mut transform, mut visibility) in &mut named {
        let original = *origins.entry(entity).or_insert(transform.translation);
        let id = format!("mesh:{}:{}", key.0, key.1);
        let offset = final_state
            .get(&("building".to_owned(), id.clone()))
            .filter(|_| requested_moves.contains(id.as_str()))
            .and_then(|(before, after)| landmark_offset(before, after));
        transform.translation = original + offset.unwrap_or_default();
        *visibility = if offset.is_some() {
            moved_landmarks.insert(id);
            Visibility::Inherited
        } else if buildings.hidden.contains(&(key.0, key.1)) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
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
                closure_ends(&mut commands, &before["geometry"]);
            }
            let shape = draw(&before, &kind, true);
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
        if !after.is_null() && !(kind == "building" && moved_landmarks.contains(&id)) {
            let shape = draw(&after, &kind, false);
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
