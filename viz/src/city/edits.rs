use super::{Buildings, NamedBuilding, SubwayView, Surface, ground::ribbon, world};
use crate::net::Scenario;
use bevy::prelude::*;
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

#[derive(Component)]
pub(crate) struct EditMesh;

#[derive(Component)]
pub(crate) struct SurfaceEdit;

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
    let color = if removed {
        [0.9, 0.08, 0.05, 1.]
    } else if kind == "line" {
        [0.02, 0.75, 0.95, 1.]
    } else {
        [0.95, 0.55, 0.07, 1.]
    };
    match kind {
        "building" => mesh.polygon(
            &rings(&geometry["coordinates"]),
            0.3,
            feature["properties"]["height_m"].as_f64().unwrap_or(0.) as f32,
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

// Every result is a full scenario. Keep only the last state of each edited object.
pub(crate) fn refresh(
    mut commands: Commands,
    scenario: Res<Scenario>,
    mut buildings: ResMut<Buildings>,
    mode: Res<SubwayView>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    old: Query<Entity, With<EditMesh>>,
    mut named: Query<(&NamedBuilding, &mut Visibility)>,
    mut material: Local<Option<Handle<StandardMaterial>>>,
) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let mut final_state = BTreeMap::<(String, String), (Value, Value)>::new();
    for edit in scenario.0["city_edits"].as_array().into_iter().flatten() {
        let (Some(kind), Some(id)) = (edit["kind"].as_str(), edit["id"].as_str()) else {
            continue;
        };
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
    for (key, mut visibility) in &mut named {
        *visibility = if buildings.hidden.contains(&(key.0, key.1)) {
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
                ..default()
            })
        })
        .clone();
    for ((kind, _), (before, after)) in final_state {
        if !before.is_null()
            && (after.is_null()
                || before["geometry"] != after["geometry"]
                || (kind == "line"
                    && before["properties"]["mode"] != after["properties"]["mode"]))
            && kind != "building"
        {
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
        if !after.is_null() {
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
