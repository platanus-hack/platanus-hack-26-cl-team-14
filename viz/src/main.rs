mod boot;
mod chart;
mod city;
mod focus;
mod label;
mod movers;
mod net;
mod pixels;
use crate::city::world;
use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    light::{CascadeShadowConfig, CascadeShadowConfigBuilder},
    post_process::dof::{DepthOfField, DepthOfFieldMode},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PrimaryWindow, WindowPosition, WindowResolution},
    winit::WinitSettings,
};
use std::{env, time::Instant};

const DAY_SKY: Color = Color::srgb_u8(249, 248, 244);

#[derive(Resource, Default, PartialEq)]
pub(crate) struct Night(bool);

pub struct MiniatureCamera;

impl Plugin for MiniatureCamera {
    fn build(&self, app: &mut App) {
        app.init_resource::<Rig>()
            .init_resource::<Night>()
            .insert_resource(WinitSettings::continuous())
            .insert_resource(ClearColor(DAY_SKY))
            .add_systems(Startup, setup)
            .add_systems(Update, city::paper_light)
            .add_systems(
                Update,
                (
                    focus::select.run_if(resource_changed::<net::Scenario>),
                    focus::cue,
                    navigate,
                    focus::advance,
                    aim,
                )
                    .chain()
                    .before(city::draw_buildings),
            )
            .add_plugins((movers::clock::ClockPlugin, movers::follow::FollowPlugin))
            .add_systems(Last, measure.run_if(resource_exists::<city::Buildings>));
    }
}

/// Use with DefaultPlugins.set(WindowPlugin { primary_window: Some(window()), ..default() }).
pub fn window() -> Window {
    let mut window = Window {
        title: "santiaguito.cl".into(),
        name: env::var("SANTIAGO_CITY_WINDOW_CLASS").ok(),
        resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
        ..default()
    };
    if let Ok(bounds) = env::var("SANTIAGO_WINDOW_RECT") {
        let values: Vec<i32> = bounds
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        let [x, y, width, height] = values[..] else {
            panic!("SANTIAGO_WINDOW_RECT requires x y width height")
        };
        window.position = WindowPosition::At(IVec2::new(x, y));
        window.resolution =
            WindowResolution::new(width as u32, height as u32).with_scale_factor_override(1.);
        window.focused = false;
    }
    if env::var_os("SANTIAGO_BOOT_SIGNAL").is_some() {
        window.set_maximized(true);
    }
    window
}

#[derive(Resource)]
pub struct Rig {
    pub target: Vec3,
    pub distance: f32,
    fixed: bool,
    flight: Option<focus::Flight>,
    event_focus: Option<Vec3>,
}

impl Default for Rig {
    fn default() -> Self {
        let focus = env::var("SANTIAGO_FOCUS").unwrap_or_else(|_| "-70.6505,-33.443".into());
        let (lon, lat) = focus.split_once(',').unwrap();
        Self {
            flight: None,
            event_focus: None,
            fixed: env::var_os("SANTIAGO_CAPTURE").is_some()
                || env::var_os("SANTIAGO_LOCK_CAMERA").is_some(),
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
    }
    .build()
}

fn setup(mut commands: Commands, rig: Res<Rig>) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV,
            near: 0.5,
            far: 160_000.0,
            ..default()
        }),
        pose(&rig),
        lens(rig.distance),
        Tonemapping::ReinhardLuminance,
        AmbientLight::default(),
    ));
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-0.65, 0.95, 0.48).looking_at(Vec3::ZERO, Vec3::Y),
        shadows(rig.distance),
    ));
}

fn navigate(
    mut rig: ResMut<Rig>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<&Camera, With<Camera3d>>,
    canvas: Query<&Sprite, With<pixels::CityCanvas>>,
) {
    if rig.fixed {
        return;
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
    if scroll != 0.0 {
        rig.flight = None;
        rig.event_focus = None;
        let distance = (rig.distance * (-scroll).exp()).clamp(55.0, 95_000.0);
        if distance != rig.distance
            && let Some(cursor) = window.cursor_position()
            && let Some(viewport) = camera.logical_viewport_size()
        {
            let transform = pose(&rig);
            // Pixel mode centers a canvas rounded up to whole output pixels.
            let size = canvas
                .single()
                .ok()
                .and_then(|s| s.custom_size)
                .unwrap_or(window.size());
            let pixel = ((cursor - window.size() * 0.5) / size + Vec2::splat(0.5)) * viewport;
            if let Ok(ray) = camera.viewport_to_world(&transform.into(), pixel)
                && let Some(anchor) = city::cursor_ground(ray)
            {
                let offset = -(*transform.forward()) * distance;
                // The eye can slide along the cursor ray. Its target follows a
                // parallel line one camera offset away, until it meets the terrain.
                rig.target = city::ground_on_line(anchor - offset, ray.direction, f32::INFINITY);
            }
        }
        rig.distance = distance;
    }
    let delta: Vec2 = motion.read().map(|ev| ev.delta).sum();
    if mouse.pressed(MouseButton::Left) && delta != Vec2::ZERO {
        rig.flight = None;
        rig.event_focus = None;
        let camera = pose(&rig);
        let right = *camera.right();
        let forward = Vec3::new(-0.32, 0., -0.51).normalize();
        let meters_per_pixel = 2.0 * rig.distance * (FOV * 0.5).tan() / window.height();
        rig.target +=
            (-right * delta.x + forward * delta.y / camera.forward().y.abs()) * meters_per_pixel;
        rig.target.y = city::terrain_from_xz(rig.target.x, rig.target.z);
    }
}

fn aim(
    rig: Res<Rig>,
    mut camera: Single<(&mut Transform, Option<&mut DepthOfField>), With<Camera3d>>,
    mut shadow: Single<&mut CascadeShadowConfig, With<DirectionalLight>>,
) {
    if rig.is_changed() {
        let (pose_component, dof) = &mut *camera;
        **pose_component = pose(&rig);
        if let Some(dof) = dof {
            **dof = lens(rig.distance);
        }
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
    buildings: Res<city::Buildings>,
    mode: Res<city::SubwayView>,
    rig: Res<Rig>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if mode.is_changed() || !buildings.ready {
        *sample = Sample::default();
    }
    if !buildings.ready || sample.finished {
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
    info!(
        "Camera target {:?}, distance {} m, mode {}",
        rig.target,
        rig.distance,
        if mode.0 { "subway" } else { "surface" }
    );
    let fps = sample.frames as f64 / elapsed;
    info!(
        "{}x{}: {} frames / {:.3} s = {:.2} FPS ({:.2} ms/frame), vsync {:?}",
        window.physical_width(),
        window.physical_height(),
        sample.frames,
        elapsed,
        fps,
        1000.0 / fps,
        window.present_mode
    );
    sample.finished = true;
    if let Ok(path) = env::var("SANTIAGO_CAPTURE") {
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                let image = event.image.clone().try_into_dynamic().unwrap().into_rgb8();
                match image.save(&path) {
                    Ok(()) => {
                        info!("Wrote {}x{} to {path}", image.width(), image.height());
                        exit.write(AppExit::Success);
                    }
                    Err(error) => {
                        error!("Could not write {path}: {error}");
                        exit.write(AppExit::error());
                    }
                }
            },
        );
    }
}

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window()),
            ..default()
        }))
        .add_plugins((
            MiniatureCamera,
            net::NetPlugin,
            chart::ChartPlugin,
            label::LabelPlugin,
            movers::MoversPlugin,
            city::metro_service::MetroServicePlugin,
            movers::events::EventsPlugin,
            pixels::PixelMode,
        ))
        .init_resource::<city::SubwayView>()
        .add_systems(
            PreUpdate,
            city::toggle_subway.after(bevy::input::InputSystems).run_if(
                bevy::input::common_conditions::input_just_pressed(KeyCode::KeyM),
            ),
        )
        .add_systems(Startup, (city::load_buildings, city::setup_ground))
        .add_systems(
            Update,
            city::edits::refresh
                .run_if(resource_changed::<net::Scenario>)
                .before(city::draw_buildings),
        )
        .add_systems(
            PostUpdate,
            city::edits::place_closure_ends
                .after(TransformSystems::Propagate)
                .before(bevy::ui::UiSystems::Layout),
        )
        .add_systems(Update, city::draw_buildings)
        .add_plugins(boot::CityBoot)
        .run()
}
