//! The launcher opens chat when the player leaves the native city title.
use crate::{Rig, city, net::Scenario};
use bevy::{
    app::{Main, MainScheduleOrder},
    camera::visibility::RenderLayers,
    ecs::schedule::ScheduleLabel,
    prelude::*,
    render::{Render, RenderApp, RenderSystems, render_resource::PipelineCache},
    text::FontSize,
    ui_render::{ExtractedUiItem, ExtractedUiNodes},
    window::{PrimaryWindow, WindowPosition},
};
use std::{
    env, fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Resource, Default)]
pub(crate) struct TitleLight(pub Option<f32>);

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct SplashStartup;

#[derive(Resource)]
struct DeferredStartup(MainScheduleOrder, Arc<AtomicBool>);

#[derive(Resource)]
struct Entry {
    path: PathBuf,
    target: Vec3,
    direction: Vec3,
    distance: f32,
    fixed: bool,
    stage: Stage,
}

enum Stage {
    Loading { total: usize },
    Title { since: f32 },
    Leaving { since: f32, night: f32 },
}

#[derive(Resource)]
struct WindowPlacement(PathBuf);

pub(crate) struct CityBoot;

impl Plugin for CityBoot {
    fn build(&self, app: &mut App) {
        app.init_resource::<TitleLight>();
        let Some(path) = env::var_os("SANTIAGO_BOOT_SIGNAL") else {
            return;
        };
        let mut rig = app.world_mut().resource_mut::<Rig>();
        let entry = Entry {
            path: path.into(),
            target: rig.target,
            direction: rig.eye_direction,
            distance: rig.distance,
            fixed: rig.fixed,
            stage: Stage::Loading { total: 0 },
        };
        rig.target = city::world(-70.6505, -33.462);
        rig.distance = 36_500.;
        rig.eye_direction = panorama_direction();
        rig.fixed = true;
        // Present the real title before synchronous terrain and route decoding.
        let presented = Arc::new(AtomicBool::new(false));
        let ack = presented.clone();
        app.sub_app_mut(RenderApp).add_systems(Render,
            (move |nodes: Res<ExtractedUiNodes>, cache: Res<PipelineCache>| {
                if !ack.load(Ordering::Relaxed) && cache.waiting_pipelines().next().is_none()
                    && nodes.uinodes.values().any(|(_, group)| group.values()
                        .any(|n| matches!(&n.item, ExtractedUiItem::Glyphs { glyphs } if !glyphs.is_empty())))
                {
                    ack.store(true, Ordering::Release);
                }
            }).after(RenderSystems::Render).before(RenderSystems::Cleanup)
        );
        let normal = app
            .world_mut()
            .remove_resource::<MainScheduleOrder>()
            .unwrap();
        app.insert_resource(DeferredStartup(normal, presented))
            .insert_resource(MainScheduleOrder {
                startup_labels: vec![SplashStartup.intern()],
                labels: vec![
                    First.intern(),
                    PreUpdate.intern(),
                    PostUpdate.intern(),
                    Last.intern(),
                ],
            })
            .insert_resource(entry)
            .insert_resource(TitleLight(Some(0.)))
            .add_systems(SplashStartup, splash)
            .add_systems(
                Main,
                start_city
                    .before(Main::run_main)
                    .run_if(resource_exists::<DeferredStartup>),
            )
            .add_systems(
                Update,
                advance
                    .after(city::draw_buildings)
                    .run_if(resource_exists::<Entry>),
            )
            .add_systems(
                PostUpdate,
                hide_hud
                    .before(bevy::ui::UiSystems::Layout)
                    .run_if(resource_exists::<Entry>),
            )
            .add_systems(
                Update,
                place_window.run_if(resource_exists::<WindowPlacement>),
            );
    }
}

// Look east across the basin from above the western foothills.
fn panorama_direction() -> Vec3 {
    let (az, el) = (-78_f32.to_radians(), 18_f32.to_radians());
    Vec3::new(az.sin() * el.cos(), el.sin(), az.cos() * el.cos())
}

fn start_city(world: &mut World) {
    if !world
        .resource::<DeferredStartup>()
        .1
        .load(Ordering::Acquire)
    {
        return;
    }
    let DeferredStartup(order, _) = world.remove_resource().unwrap();
    let startup = order.startup_labels.clone();
    world.insert_resource(order);
    info!("Boot: loading city data");
    for label in startup {
        let _ = world.try_run_schedule(label);
    }
}

#[derive(Component)]
struct Cover;
#[derive(Component)]
struct HiddenHud(Display);
#[derive(Component)]
struct BootText;
#[derive(Component)]
struct BootBackground(f32);

#[derive(Resource)]
struct Elements {
    cover: Entity,
    camera: Entity,
    track: Entity,
    fill: Entity,
    prompt: Entity,
}

fn splash(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let terrain: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../web/data/terrain.json")).unwrap();
    let credits = format!(
        "© OpenStreetMap contributors · openstreetmap.org/copyright · ODbL 1.0\nGoogle Research Open Buildings 2.5D Temporal (2023) · CC BY 4.0 · Sentinel-2\n{}",
        terrain["attribution"].as_str().unwrap()
    );
    let font = fonts.add(Font::from_bytes(
        include_bytes!("../assets/barlow/Barlow-Regular.ttf").to_vec(),
    ));
    let text = |value: &str, size: FontSize| {
        (
            BootText,
            Text::new(value),
            TextFont {
                font: font.clone().into(),
                font_size: size,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::justify(Justify::Center),
        )
    };
    let camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: 10,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            Msaa::Off,
            RenderLayers::none(),
        ))
        .id();
    let cover = commands
        .spawn((
            Cover,
            UiTargetCamera(camera),
            GlobalZIndex(100),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(ink(1.)),
        ))
        .id();
    commands.spawn((
        ChildOf(cover),
        text("santiaguito.cl", FontSize::VMin(12.)),
        TextShadow {
            offset: Vec2::new(1.5, 3.),
            color: ink(0.85),
        },
    ));
    let track = commands
        .spawn((
            ChildOf(cover),
            Node {
                width: percent(23),
                height: px(4),
                margin: UiRect::top(px(18)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.68, 0.72, 0.86, 0.32)),
        ))
        .id();
    let fill = commands
        .spawn((
            ChildOf(track),
            Node {
                width: percent(0),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::WHITE),
        ))
        .id();
    let prompt = commands
        .spawn((
            ChildOf(cover),
            BackgroundColor(ink(0.84)),
            BootBackground(0.84),
            text("Enter para empezar", FontSize::Vw(2.8)),
            Node {
                display: Display::None,
                margin: UiRect::top(px(12)),
                padding: UiRect::axes(px(22), px(4)),
                ..default()
            },
        ))
        .id();
    commands.spawn((
        ChildOf(cover),
        text(&credits, FontSize::Px(27.)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(0),
            width: percent(100),
            padding: UiRect::axes(px(24), px(8)),
            ..default()
        },
        BackgroundColor(ink(0.90)),
        BootBackground(0.90),
    ));
    commands.insert_resource(Elements {
        cover,
        camera,
        track,
        fill,
        prompt,
    });
}

fn ink(alpha: f32) -> Color {
    Color::srgba(0.055, 0.063, 0.108, alpha)
}

fn hide_hud(
    mut commands: Commands,
    mut roots: Query<(Entity, &mut Node), (Without<ChildOf>, Without<Cover>, Without<HiddenHud>)>,
) {
    for (entity, mut node) in &mut roots {
        commands.entity(entity).insert(HiddenHud(node.display));
        node.display = Display::None;
    }
}

fn advance(
    mut commands: Commands,
    mut entry: ResMut<Entry>,
    ui: Res<Elements>,
    buildings: Res<city::Buildings>,
    scenario: Res<Scenario>,
    time: Res<Time<Real>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut rig: ResMut<Rig>,
    mut light: ResMut<TitleLight>,
    mut nodes: Query<&mut Node>,
    mut backgrounds: Query<&mut BackgroundColor>,
    backing: Query<(Entity, &BootBackground)>,
    mut labels: Query<(&mut TextColor, Option<&mut TextShadow>), With<BootText>>,
    hidden: Query<(Entity, &HiddenHud)>,
    mut exit: MessageWriter<AppExit>,
    #[cfg(target_os = "macos")] display: (
        Query<&bevy::window::OnMonitor, With<PrimaryWindow>>,
        Query<&bevy::window::Monitor>,
    ),
) {
    let now = time.elapsed_secs();
    match entry.stage {
        Stage::Loading { ref mut total } => {
            *total = (*total).max(buildings.pending_tiles);
            let done = *total - buildings.pending_tiles + usize::from(!scenario.0.is_null());
            nodes.get_mut(ui.fill).unwrap().width =
                percent(100. * done as f32 / (*total + 1) as f32);
            if buildings.ready && !scenario.0.is_null() {
                nodes.get_mut(ui.track).unwrap().display = Display::None;
                nodes.get_mut(ui.prompt).unwrap().display = Display::Flex;
                backgrounds.get_mut(ui.cover).unwrap().0 = ink(0.46);
                entry.stage = Stage::Title { since: now };
                info!("Boot ready: full city panorama");
            }
        }
        Stage::Title { since } => {
            let night = (1. - ((now - since) * std::f32::consts::TAU / 16.).cos()) * 0.5;
            light.0 = Some(night);
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
                rig.fly_to(entry.target, entry.distance);
                entry.stage = Stage::Leaving { since: now, night };
            }
        }
        Stage::Leaving { since, night } => {
            let opacity = (1. - (now - since) / 0.75).clamp(0., 1.);
            let turn = 1. - opacity;
            rig.eye_direction = panorama_direction().slerp(entry.direction, turn * turn * (3. - 2. * turn));
            backgrounds.get_mut(ui.cover).unwrap().0 = ink(0.46 * opacity);
            for (entity, base) in &backing {
                backgrounds.get_mut(entity).unwrap().0 = ink(base.0 * opacity);
            }
            for (mut color, shadow) in &mut labels {
                color.0.set_alpha(opacity);
                if let Some(mut shadow) = shadow {
                    shadow.color = ink(0.85 * opacity);
                }
            }
            light.0 = Some(night * opacity);
            if opacity > 0. {
                return;
            }
            rig.fixed = entry.fixed;
            light.0 = None;
            for (entity, old) in &hidden {
                nodes.get_mut(entity).unwrap().display = old.0;
                commands.entity(entity).remove::<HiddenHud>();
            }
            commands.entity(ui.cover).despawn();
            commands.entity(ui.camera).despawn();
            commands.remove_resource::<Entry>();
            commands.remove_resource::<Elements>();
            #[cfg(target_os = "macos")]
            if env::var_os(crate::chat_terminal::ENABLE).is_some() {
                commands.insert_resource(crate::chat_terminal::ChatReady);
                return;
            }
            #[cfg(target_os = "macos")]
            let signal = {
                let (window, monitors) = &display;
                let monitor = monitors.get(window.single().unwrap().0).unwrap();
                // Winit's global monitor origin is physical, scaled by this monitor.
                // AppKit and Terminal use global desktop points instead.
                let origin = monitor.physical_position.as_dvec2() / monitor.scale_factor;
                format!("{} {}\n", origin.x.round() as i32, origin.y.round() as i32)
            };
            #[cfg(not(target_os = "macos"))]
            let signal = "ready\n";
            match fs::write(&entry.path, signal) {
                Ok(()) => {
                    let mut rect = entry.path.clone().into_os_string();
                    rect.push(".rect");
                    commands.insert_resource(WindowPlacement(rect.into()));
                    info!("Boot entered: launching chat");
                }
                Err(err) => {
                    error!("Cannot signal launcher: {err}");
                    exit.write(AppExit::error());
                }
            }
        }
    }
}

fn place_window(
    mut commands: Commands,
    path: Res<WindowPlacement>,
    mut restored: Local<bool>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(rect) = fs::read_to_string(&path.0) else {
        return;
    };
    // Winit applies maximization after size changes. Restore on its own frame.
    if !*restored {
        window.set_maximized(false);
        *restored = true;
        return;
    }
    let v: Vec<i32> = rect
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect();
    let [x, y, width, height] = v[..] else {
        panic!("Expected x y width height")
    };
    window.position = WindowPosition::At(IVec2::new(x, y));
    window
        .resolution
        .set_physical_resolution(width as u32, height as u32);
    commands.remove_resource::<WindowPlacement>();
}
