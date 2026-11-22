//! Surface service lines for city overviews and the matched Metro replay.
use super::{SubwayView, Surface, ground::ribbon, world};
use crate::{
    Rig,
    movers::clock::{CityClock, UpdateClock},
    net::Scenario,
};
use bevy::prelude::*;
use serde_json::Value;
use std::{fs::File, ops::Range};

pub(crate) struct MetroServicePlugin;

impl Plugin for MetroServicePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, fade.after(UpdateClock));
    }
}

#[derive(Component)]
struct ServiceLines(f32);

fn half_width(result: &Value, distance: f32) -> Option<f32> {
    if distance >= 26_000. {
        Some(115.)
    } else if distance >= 7_000. {
        Some(34.)
    } else if result["scene"]["layer"] == "metro_service" {
        Some(8.)
    } else {
        None
    }
}

// Build every width at boot. A lazy build parses the routes and stalls a camera flight.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let routes: Vec<Value> =
        serde_json::from_reader(File::open("web/data/transit_paths.json").unwrap()).unwrap();
    let lines: Vec<_> = routes
        .iter()
        .filter(|route| route["mode"] == "metro" && route["direction"] == 0)
        .map(|route| {
            let points: Vec<_> = route["path"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| world(p[0].as_f64().unwrap(), p[1].as_f64().unwrap()))
                .collect();
            let color = Srgba::hex(route["color"].as_str().unwrap()).unwrap();
            (points, Color::from(color).to_linear().to_f32_array())
        })
        .collect();
    for width in [115., 34., 8.] {
        let mut mesh = Surface::default();
        for (points, color) in &lines {
            ribbon(&mut mesh, points, width, 1.2, *color);
        }
        commands.spawn((
            ServiceLines(width),
            Mesh3d(meshes.add(mesh.finish())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: service_color(1.0),
                unlit: true,
                cull_mode: None,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Visibility::Hidden,
        ));
    }
}

fn service_color(alpha: f32) -> Color {
    Color::linear_rgba(1., 1., 1., alpha)
}

#[derive(Default)]
struct FadeState {
    closure: Option<Range<i32>>,
    started: Option<f64>,
}

fn fade(
    scenario: Res<Scenario>,
    clock: Res<CityClock>,
    rig: Res<Rig>,
    subway: Res<SubwayView>,
    time: Res<Time<Real>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<FadeState>,
    mut lines: Query<(
        &ServiceLines,
        &MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
    )>,
) {
    if scenario.is_changed() {
        let result = &scenario.0;
        let closure = result["applied"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|change| {
                change["kind"] == "line"
                    && change["op"] == "remove"
                    && change["target"]["type"] == "mode"
                    && change["target"]["id"] == "metro"
            });
        *state = FadeState {
            closure: closure.then(|| match result["closure_window"]["half_hours"].as_array() {
                Some(window) => {
                    window[0].as_u64().unwrap() as i32..window[1].as_u64().unwrap() as i32
                }
                None => 0..48,
            }),
            started: None,
        };
    }

    let closed = clock.0.is_some_and(|seconds| {
        let slot = (seconds.rem_euclid(86_400.) / 1800.) as i32;
        state
            .closure
            .as_ref()
            .is_some_and(|bins| bins.contains(&slot))
    });
    let alpha = if closed {
        let now = time.elapsed_secs_f64();
        let start = *state.started.get_or_insert(now);
        (1.0 - (now - start) as f32).max(0.0)
    } else {
        state.started = None;
        1.0
    };
    let active_width = half_width(&scenario.0, rig.distance);
    for (line, handle, mut visibility) in &mut lines {
        if subway.0 || alpha == 0.0 || Some(line.0) != active_width {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }
        let color = service_color(alpha);
        if materials.get(&handle.0).unwrap().base_color != color {
            materials.get_mut(&handle.0).unwrap().base_color = color;
        }
        visibility.set_if_neq(Visibility::Inherited);
    }
}
