mod city;
mod net;
mod movers;
use std::{env, time::Instant};
use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    light::{CascadeShadowConfig, CascadeShadowConfigBuilder},
    post_process::dof::{DepthOfField, DepthOfFieldMode},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    text::FontSize,
    window::{PrimaryWindow, WindowResolution},
    winit::WinitSettings,
};
use crate::city::world;

pub struct MiniatureCamera;

impl Plugin for MiniatureCamera {
    fn build(&self, app: &mut App) {
        app.init_resource::<Rig>()
            .insert_resource(WinitSettings::continuous())
            .insert_resource(ClearColor(Color::srgb_u8(185, 202, 205)))
            .add_systems(Startup, setup)
            .add_systems(Update, (navigate, aim).chain())
            .add_systems(Last, measure);
    }
}

/// Use with DefaultPlugins.set(WindowPlugin { primary_window: Some(window()), ..default() }).
pub fn window() -> Window {
    Window {
        title: "Santiago".into(),
        resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
        ..default()
    }
}

#[derive(Resource)]
pub struct Rig {
    pub target: Vec3,
    pub distance: f32,
}

impl Default for Rig {
    fn default() -> Self {
        let focus = env::var("SANTIAGO_FOCUS").unwrap_or_else(|_| "-70.6505,-33.443".into());
        let (lon, lat) = focus.split_once(',').unwrap();
        Self {
            target: world(lon.trim().parse().unwrap(), lat.trim().parse().unwrap()),
            distance: env::var("SANTIAGO_DISTANCE").map_or(1650.0, |s| s.parse().unwrap()),
        }
    }
}

const FOV: f32 = 35.0 * std::f32::consts::PI / 180.0;

fn pose(rig: &Rig) -> Transform {
    let offset = Vec3::new(0.32, 0.79, 0.51).normalize() * rig.distance;
    Transform::from_translation(rig.target + offset).looking_at(rig.target, Vec3::Y)
}

fn lens(distance: f32) -> DepthOfField {
    DepthOfField {
        mode: DepthOfFieldMode::Gaussian,
        focal_distance: distance,
        // Keep the virtual model's magnification as the camera crosses city-scale distances.
        sensor_height: distance * 0.014,
        aperture_f_stops: 0.85,
        max_circle_of_confusion_diameter: 12.0,
        max_depth: distance * 2.4,
    }
}

fn shadows(distance: f32) -> CascadeShadowConfig {
    CascadeShadowConfigBuilder {
        num_cascades: 2,
        minimum_distance: 0.0,
        first_cascade_far_bound: distance * 0.95,
        maximum_distance: distance * 2.0,
        ..default()
    }.build()
}

fn setup(mut commands: Commands, rig: Res<Rig>) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV, near: 0.5, far: 160_000.0, ..default()
        }),
        pose(&rig),
        lens(rig.distance),
        Tonemapping::ReinhardLuminance,
        AmbientLight {
            color: Color::srgb_u8(195, 218, 255),
            brightness: 950.0,
            ..default()
        },
    ));
    commands.spawn((
        DirectionalLight {
            color: Color::srgb_u8(255, 245, 231),
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-0.65, 0.95, 0.48).looking_at(Vec3::ZERO, Vec3::Y),
        shadows(rig.distance),
    ));
    commands.spawn((
        Text::new("© OpenStreetMap contributors"),
        TextFont { font_size: FontSize::Px(14.0), ..default() },
        TextColor(Color::WHITE),
        TextShadow::default(),
        Node { position_type: PositionType::Absolute, bottom: px(8), right: px(12), ..default() },
    ));
}

fn navigate(
    mut rig: ResMut<Rig>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let scroll: f32 = wheel.read().map(|event| event.y * match event.unit {
        MouseScrollUnit::Line => 0.15,
        MouseScrollUnit::Pixel => 0.002,
    }).sum();
    if scroll != 0.0 {
        rig.distance = (rig.distance * (-scroll).exp()).clamp(55.0, 95_000.0);
    }
    let delta: Vec2 = motion.read().map(|ev| ev.delta).sum();
    if mouse.pressed(MouseButton::Left) && delta != Vec2::ZERO {
        let camera = pose(&rig);
        let right = *camera.right();
        let forward = Vec3::new(-0.32, 0., -0.51).normalize();
        let meters_per_pixel = 2.0 * rig.distance * (FOV * 0.5).tan() / window.height();
        rig.target += (-right * delta.x + forward * delta.y / camera.forward().y.abs()) * meters_per_pixel;
        rig.target.y=city::terrain_from_xz(rig.target.x,rig.target.z);
    }
}

fn aim(
    rig: Res<Rig>,
    mut camera: Single<(&mut Transform, &mut DepthOfField), With<Camera3d>>,
    mut shadow: Single<&mut CascadeShadowConfig, With<DirectionalLight>>,
) {
    if rig.is_changed() {
        let (pose_component, dof) = &mut *camera;
        **pose_component = pose(&rig);
        **dof = lens(rig.distance);
        **shadow = shadows(rig.distance);
    }
}

#[derive(Default)]
struct Sample {
    start: Option<Instant>,
    frames: u64,
    warmed: bool,
    finished: bool,
}

fn measure(
    mut commands: Commands,
    mut sample: Local<Sample>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if sample.finished {
        return;
    }
    let now = Instant::now();
    let started = *sample.start.get_or_insert(now);
    let elapsed = now.duration_since(started).as_secs_f64();
    sample.frames += 1;
    if !sample.warmed {
        if elapsed >= 4. && sample.frames >= 30 {
            sample.warmed = true;
            sample.frames = 0;
            sample.start = Some(now);
        }
        return;
    }
    if elapsed < 6.0 {
        return;
    }
    let fps = sample.frames as f64 / elapsed;
    info!("{}x{}: {} frames / {:.3} s = {:.2} FPS ({:.2} ms/frame), vsync {:?}",
        window.physical_width(), window.physical_height(), sample.frames, elapsed, fps, 1000.0 / fps, window.present_mode);
    sample.finished = true;
    if let Ok(path) = env::var("SANTIAGO_CAPTURE") {
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                let image = event.image.clone().try_into_dynamic().unwrap().into_rgb8();
                match image.save(&path) {
                    Ok(()) => { info!("Wrote {}x{} to {path}", image.width(), image.height()); exit.write(AppExit::Success); }
                    Err(error) => { error!("Could not write {path}: {error}"); exit.write(AppExit::error()); }
                }
            },
        );
    }
}

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {primary_window: Some(window()), ..default()}))
        .add_plugins((MiniatureCamera,net::NetPlugin,movers::MoversPlugin))
        .add_systems(Startup,(city::load_buildings,city::setup_ground))
        .add_systems(Update,city::draw_buildings)
        .run()
}
