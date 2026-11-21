//! Open a captured local bundle. No city or simulation systems are installed.
use bevy::{
    asset::{AssetPlugin, UntypedAssetId},
    core_pipeline::tonemapping::Tonemapping,
    gltf::{GltfPlugin, convert_coordinates::GltfConvertCoordinates},
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    math::DMat4,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::PrimaryWindow,
    world_serialization::{WorldInstance, WorldInstanceSpawner},
};
use serde::Deserialize;
use std::{f32::consts::FRAC_PI_4, path::PathBuf, time::Instant};

#[derive(Deserialize, Resource)]
struct Bundle {
    tiles: Vec<Tile>,
    attribution: Vec<String>,
    google_logo: String,
}

#[derive(Deserialize)]
struct Tile {
    path: String,
    // Capture has composed Earth placement and glTF axis conversion in f64.
    // This column-major matrix already maps to local east/up/south metres.
    transform: [f64; 16],
}

#[derive(Resource)]
struct Orbit {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

fn pose(orbit: &Orbit) -> Transform {
    let rotation = Quat::from_rotation_y(orbit.yaw) * Quat::from_rotation_x(-orbit.pitch);
    Transform::from_translation(orbit.target + rotation * Vec3::Z * orbit.distance)
        .looking_at(orbit.target, Vec3::Y)
}

#[derive(Component)]
struct TileScene;

#[derive(Resource)]
struct LogoAsset(Handle<Image>);

#[derive(Resource)]
struct Capture {
    path: Option<PathBuf>,
    exit_after: bool,
    started: Instant,
    ready_at: Option<Instant>,
    warm_updates: u32,
    requested: bool,
    finished: bool,
}

fn main() -> AppExit {
    let started = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    let camera_value = |flag: &str, default: f32| {
        args.iter()
            .position(|arg| arg == flag)
            .map(|index| args[index + 1].parse::<f32>().unwrap())
            .unwrap_or(default)
    };
    let capture_path = args.iter().position(|arg| arg == "--capture").map(|i| {
        let path = PathBuf::from(args.get(i + 1).expect("--capture needs a PNG path"));
        assert!(
            path.extension().is_some_and(|e| e == "png"),
            "--capture needs a .png path"
        );
        assert!(!path.exists(), "capture output already exists");
        path
    });
    let exit_after = args.iter().any(|arg| arg == "--exit-after-capture");
    assert!(
        !exit_after || capture_path.is_some(),
        "--exit-after-capture requires --capture"
    );
    let directory = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Use: view_demo_tiles <bundle-directory>"),
    )
    .canonicalize()
    .unwrap();
    let bundle: Bundle =
        serde_json::from_slice(&std::fs::read(directory.join("manifest.json")).unwrap()).unwrap();
    println!(
        "Opening {} local tiles from {}",
        bundle.tiles.len(),
        directory.display()
    );
    App::new()
        .insert_resource(Capture {
            path: capture_path,
            exit_after,
            started,
            ready_at: None,
            warm_updates: 0,
            requested: false,
            finished: false,
        })
        .insert_resource(bundle)
        .insert_resource(ClearColor(Color::srgb(0.70, 0.78, 0.85)))
        .insert_resource(Orbit {
            target: Vec3::new(0., camera_value("--target-y", 120.), 0.),
            yaw: FRAC_PI_4,
            pitch: 0.48,
            distance: camera_value("--distance", 850.),
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: directory.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(GltfPlugin {
                    convert_coordinates: GltfConvertCoordinates {
                        rotate_scene_entity: false,
                        rotate_meshes: false,
                    },
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Demo · local Google 3D Tiles".into(),
                        resolution: (1920, 1080).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (navigate, capture_when_ready))
        .run()
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, bundle: Res<Bundle>, orbit: Res<Orbit>) {
    let logo: Handle<Image> = assets.load(bundle.google_logo.clone());
    commands.insert_resource(LogoAsset(logo.clone()));
    for tile in &bundle.tiles {
        // Earth-scale subtraction was completed in f64 by capture. Only local
        // metres become f32 here; preserve each GLB's own node hierarchy.
        let local = DMat4::from_cols_array(&tile.transform).as_mat4();
        commands.spawn((
            TileScene,
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(tile.path.clone()))),
            Transform::from_matrix(local),
        ));
    }
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            near: 0.2,
            far: 20_000.,
            ..default()
        }),
        AmbientLight {
            brightness: 600.,
            ..default()
        },
        Tonemapping::None,
        pose(&orbit),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.,
            ..default()
        },
        Transform::from_xyz(400., 900., 300.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(12),
                left: px(12),
                right: px(12),
                padding: UiRect::all(px(12)),
                align_items: AlignItems::Center,
                column_gap: px(18),
                ..default()
            },
            BackgroundColor(Color::srgba(1., 1., 1., 0.94)),
        ))
        .with_children(|bar| {
            bar.spawn((
                ImageNode::new(logo.clone()),
                Node {
                    width: px(110),
                    flex_shrink: 0.,
                    ..default()
                },
            ));
            bar.spawn((
                Text::new(bundle.attribution.join(" · ")),
                TextFont {
                    font_size: FontSize::Px(18.),
                    ..default()
                },
                TextColor(Color::srgb(0.10, 0.12, 0.16)),
                Node {
                    flex_shrink: 1.,
                    ..default()
                },
            ));
        });
}

fn navigate(
    capture: Res<Capture>,
    mut orbit: ResMut<Orbit>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    // Keep the CLI camera fixed while producing the optional artifact.
    if capture.path.is_some() && !capture.finished {
        motion.clear();
        wheel.clear();
        return;
    }
    let delta: Vec2 = motion.read().map(|event| event.delta).sum();
    if mouse.pressed(MouseButton::Left) && !keys.pressed(KeyCode::ShiftLeft) {
        orbit.yaw -= delta.x * 0.006;
        orbit.pitch = (orbit.pitch + delta.y * 0.006).clamp(0.05, 1.50);
    }
    if mouse.pressed(MouseButton::Right)
        || mouse.pressed(MouseButton::Middle)
        || (mouse.pressed(MouseButton::Left) && keys.pressed(KeyCode::ShiftLeft))
    {
        let scale = 2. * orbit.distance * (FRAC_PI_4 * 0.5).tan() / window.height();
        let right = Vec3::new(orbit.yaw.cos(), 0., -orbit.yaw.sin());
        let forward = Vec3::new(-orbit.yaw.sin(), 0., -orbit.yaw.cos());
        orbit.target += (-right * delta.x + forward * delta.y) * scale;
    }
    let scroll: f32 = wheel
        .read()
        .map(|event| {
            event.y
                * match event.unit {
                    MouseScrollUnit::Line => 0.15,
                    MouseScrollUnit::Pixel => 0.002,
                }
        })
        .sum();
    orbit.distance = (orbit.distance * (-scroll).exp()).clamp(10., 5_000.);
    **camera = pose(&orbit);
}

// Dependency readiness alone does not mean that a world instance has spawned.
// Check both, then allow render preparation before requesting a GPU screenshot.
fn capture_when_ready(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    assets: Res<AssetServer>,
    logo: Res<LogoAsset>,
    bundle: Res<Bundle>,
    scenes: Query<(&WorldAssetRoot, Option<&WorldInstance>), With<TileScene>>,
    meshes: Query<(), With<Mesh3d>>,
    spawner: Res<WorldInstanceSpawner>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.path.is_none() || capture.finished {
        return;
    }
    if capture.started.elapsed().as_secs_f64() > 180. {
        error!("PNG export timed out before successful save; no completed export is claimed");
        capture.finished = true;
        exit.write(AppExit::error());
        return;
    }
    if capture.requested {
        return;
    }
    let ids: Vec<UntypedAssetId> = scenes
        .iter()
        .map(|(root, _)| root.id().into())
        .chain([logo.0.id().into()])
        .collect();
    for id in &ids {
        if let Some((load, dependencies, recursive)) = assets.get_load_states(*id) {
            if load.is_failed() || dependencies.is_failed() || recursive.is_failed() {
                error!("PNG export cancelled: tile or logo dependency failed: {id:?}");
                capture.finished = true;
                exit.write(AppExit::error());
                return;
            }
        }
    }
    let ready = !bundle.tiles.is_empty()
        && scenes.iter().count() == bundle.tiles.len()
        && ids.iter().all(|id| assets.is_loaded_with_dependencies(*id))
        && scenes.iter().all(|(_, instance)| {
            instance.is_some_and(|instance| spawner.instance_is_ready(**instance))
        })
        && !meshes.is_empty();
    if !ready {
        capture.ready_at = None;
        capture.warm_updates = 0;
        return;
    }
    if capture.ready_at.is_none() {
        let now = Instant::now();
        capture.ready_at = Some(now);
        info!(
            "PNG export assets and instances ready: {} tiles, {} mesh entities, startup {:.3}s",
            bundle.tiles.len(),
            meshes.iter().count(),
            now.duration_since(capture.started).as_secs_f64()
        );
    }
    capture.warm_updates += 1;
    if capture.warm_updates < 120 || capture.ready_at.unwrap().elapsed().as_secs_f64() < 4. {
        return;
    }
    capture.requested = true;
    let path = capture.path.clone().unwrap();
    info!(
        "Requesting renderer PNG after {} warm-up updates, startup {:.3}s",
        capture.warm_updates,
        capture.started.elapsed().as_secs_f64()
    );
    commands.spawn(Screenshot::primary_window()).observe(
        move |event: On<ScreenshotCaptured>, mut capture: ResMut<Capture>, mut exit: MessageWriter<AppExit>| {
            let saved = event.image.clone().try_into_dynamic().map_err(|e| format!("{e:?}")).and_then(|image| {
                let image = image.into_rgb8();
                image.save(&path).map(|()| (image.width(), image.height())).map_err(|e| e.to_string())
            });
            capture.finished = true;
            match saved {
                Ok((width, height)) => {
                    info!("Saved renderer PNG {width}x{height} to {}: startup-to-save {:.3}s (includes warm-up and PNG encoding)", path.display(), capture.started.elapsed().as_secs_f64());
                    if capture.exit_after { exit.write(AppExit::Success); }
                }
                Err(error) => {
                    error!("Could not save renderer PNG {}: {error}", path.display());
                    exit.write(AppExit::error());
                }
            }
        },
    );
}
