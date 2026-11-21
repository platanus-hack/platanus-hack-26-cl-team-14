//! Open a captured local bundle. No city or simulation systems are installed.
use bevy::{
    asset::AssetPlugin,
    core_pipeline::tonemapping::Tonemapping,
    gltf::{GltfPlugin, convert_coordinates::GltfConvertCoordinates},
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    math::DMat4,
    prelude::*,
    window::PrimaryWindow,
};
use serde::Deserialize;
use std::{f32::consts::FRAC_PI_4, path::PathBuf};

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let camera_value = |flag: &str, default: f32| {
        args.iter()
            .position(|arg| arg == flag)
            .map(|index| args[index + 1].parse::<f32>().unwrap())
            .unwrap_or(default)
    };
    let directory = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Use: view_costanera <bundle-directory>"),
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
                        title: "Costanera · local Google 3D Tiles".into(),
                        resolution: (1920, 1080).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, navigate)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, bundle: Res<Bundle>, orbit: Res<Orbit>) {
    for tile in &bundle.tiles {
        // Earth-scale subtraction was completed in f64 by capture. Only local
        // metres become f32 here; preserve each GLB's own node hierarchy.
        let local = DMat4::from_cols_array(&tile.transform).as_mat4();
        commands.spawn((
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
                ImageNode::new(assets.load(bundle.google_logo.clone())),
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
    mut orbit: ResMut<Orbit>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
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
