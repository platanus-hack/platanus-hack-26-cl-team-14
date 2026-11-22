use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    text::FontSize,
    ui::RelativeCursorPosition,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    env,
    io::Write,
    process::{Command, Stdio},
    sync::{Mutex, mpsc},
    thread,
};

const INK: Color = Color::srgba(0.07, 0.12, 0.20, 0.94);
const PAPER: Color = Color::srgb(0.96, 0.94, 0.88);
const MUTED: Color = Color::srgb(0.74, 0.79, 0.85);
const BUTTON: Color = Color::srgb(0.14, 0.22, 0.33);
const HOVER: Color = Color::srgb(0.22, 0.34, 0.48);
const BORDER: Color = Color::srgb(0.38, 0.46, 0.55);

pub(crate) struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        let (request, requests) = mpsc::channel();
        let (replies, reply) = mpsc::channel();
        let engine = env::var("SANTIAGO_ENGINE").unwrap_or_else(|_| "http://127.0.0.1:8787".into());
        thread::spawn(move || {
            for request in requests {
                let reply = match request {
                    Request::Catalogue => Reply::Catalogue(
                        call(&engine, "list_replays", json!({})).and_then(|value| {
                            serde_json::from_value(value).map_err(|error| error.to_string())
                        }),
                    ),
                    Request::Play(id) => Reply::Playback(
                        call(&engine, "run_replay", json!({ "id": id })).map(|_| ()),
                    ),
                };
                if replies.send(reply).is_err() {
                    break;
                }
            }
        });
        app.insert_resource(Worker {
            request,
            reply: Mutex::new(reply),
        })
        .init_resource::<History>()
        .add_message::<ReplayStarted>()
        .add_systems(Startup, setup)
        .add_systems(Update, (interact, receive, scroll).chain());
    }
}

#[derive(Message)]
pub(crate) struct ReplayStarted;

#[derive(Component)]
pub(crate) struct HistorySurface;

#[derive(Component)]
struct Panel;
#[derive(Component)]
struct Catalogue;
#[derive(Component)]
struct Status;
#[derive(Component)]
struct ToggleLabel;
#[derive(Component)]
struct Detail(String);
#[derive(Component)]
enum Action {
    Toggle,
    Select(String),
    Play(String),
}

#[derive(Resource, Default)]
struct History {
    open: bool,
    playing: bool,
    selected: Option<String>,
}

#[derive(Resource)]
struct Worker {
    request: mpsc::Sender<Request>,
    reply: Mutex<mpsc::Receiver<Reply>>,
}

enum Request {
    Catalogue,
    Play(String),
}

enum Reply {
    Catalogue(Result<Vec<Replay>, String>),
    Playback(Result<(), String>),
}

#[derive(Deserialize)]
struct Replay {
    id: String,
    title: String,
    scene: Scene,
    duration_half_hours: u32,
    analyze: String,
}

#[derive(Deserialize)]
struct Scene {
    half_hour: u32,
}

fn call(engine: &str, name: &str, arguments: Value) -> Result<Value, String> {
    let mut child = Command::new("node")
        .arg("tools/santiago-mcp/server.mjs")
        .env("SANTIAGO_API_URL", engine)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let request = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": name, "arguments": arguments }
    });
    writeln!(child.stdin.take().unwrap(), "{request}").map_err(|error| error.to_string())?;
    let response = child
        .wait_with_output()
        .map_err(|error| error.to_string())?;
    if !response.status.success() {
        return Err(String::from_utf8_lossy(&response.stderr).into_owned());
    }
    let response: Value =
        serde_json::from_slice(&response.stdout).map_err(|error| error.to_string())?;
    let result = &response["result"];
    let content = result["content"][0]["text"]
        .as_str()
        .ok_or_else(|| response.to_string())?;
    if result["isError"] == true {
        return Err(content.to_owned());
    }
    serde_json::from_str(content).map_err(|error| error.to_string())
}

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Action::Toggle,
            HistorySurface,
            RelativeCursorPosition::default(),
            GlobalZIndex(20),
            BackgroundColor(BUTTON),
            BorderColor::all(BORDER),
            Node {
                position_type: PositionType::Absolute,
                top: px(106),
                left: px(24),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(7)),
                padding: UiRect::axes(px(13), px(8)),
                ..default()
            },
        ))
        .with_children(|button| {
            button.spawn((ToggleLabel, text("Historial", 19., PAPER)));
        });

    commands
        .spawn((
            Panel,
            HistorySurface,
            RelativeCursorPosition::default(),
            GlobalZIndex(20),
            BackgroundColor(INK),
            BorderColor::all(BORDER),
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                top: px(156),
                bottom: px(24),
                left: px(24),
                width: vw(27),
                min_width: px(290),
                max_width: px(460),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(9)),
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn(text("Experimentos", 24., PAPER));
            panel.spawn((Status, text("", 16., MUTED)));
            panel.spawn((
                Catalogue,
                RelativeCursorPosition::default(),
                ScrollPosition::default(),
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.,
                    min_height: px(0),
                    overflow: Overflow::scroll_y(),
                    row_gap: px(8),
                    ..default()
                },
            ));
        });
}

fn interact(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut buttons: Query<(&RelativeCursorPosition, &Action, &mut BackgroundColor)>,
    mut panel: Single<&mut Node, (With<Panel>, Without<Detail>)>,
    mut details: Query<(&Detail, &mut Node), Without<Panel>>,
    catalogue: Single<Entity, With<Catalogue>>,
    mut label: Single<&mut Text, (With<ToggleLabel>, Without<Status>)>,
    mut status: Single<&mut Text, (With<Status>, Without<ToggleLabel>)>,
    mut history: ResMut<History>,
    worker: Res<Worker>,
) {
    for (cursor, action, mut color) in &mut buttons {
        let hovered = cursor.cursor_over();
        let selected =
            matches!(action, Action::Select(id) if history.selected.as_ref() == Some(id));
        let background = if hovered || selected { HOVER } else { BUTTON };
        if color.0 != background {
            color.0 = background;
        }
        if !hovered || !mouse.just_pressed(MouseButton::Left) {
            continue;
        }
        match action {
            Action::Toggle => {
                history.open = !history.open;
                panel.display = if history.open {
                    Display::Flex
                } else {
                    Display::None
                };
                label.0 = if history.open {
                    "Cerrar historial"
                } else {
                    "Historial"
                }
                .into();
                if history.open {
                    history.selected = None;
                    commands.entity(*catalogue).despawn_children();
                    status.0 = "Cargando historial…".into();
                    worker.request.send(Request::Catalogue).unwrap();
                }
            }
            Action::Select(id) if history.open => {
                history.selected = Some(id.clone());
                for (detail, mut node) in &mut details {
                    node.display = if &detail.0 == id {
                        Display::Flex
                    } else {
                        Display::None
                    };
                }
            }
            Action::Play(id)
                if history.open && !history.playing && history.selected.as_ref() == Some(id) =>
            {
                history.playing = true;
                status.0 = "Abriendo experimento…".into();
                worker.request.send(Request::Play(id.clone())).unwrap();
            }
            Action::Select(_) | Action::Play(_) => {}
        }
        break;
    }
}

fn receive(
    mut commands: Commands,
    worker: Res<Worker>,
    mut history: ResMut<History>,
    mut status: Single<&mut Text, With<Status>>,
    mut catalogue: Single<(Entity, &mut ScrollPosition), With<Catalogue>>,
    mut started: MessageWriter<ReplayStarted>,
) {
    for reply in worker.reply.lock().unwrap().try_iter() {
        match reply {
            Reply::Catalogue(Ok(replays)) => {
                status.0 = if replays.is_empty() {
                    "Aún no hay experimentos guardados.".into()
                } else {
                    String::new()
                };
                catalogue.1.0 = Vec2::ZERO;
                commands
                    .entity(catalogue.0)
                    .despawn_children()
                    .with_children(|list| {
                        for replay in replays {
                            let start = replay.scene.half_hour;
                            let timing = format!(
                                "{:02}:{:02} · {} min",
                                start / 2,
                                30 * (start % 2),
                                replay.duration_half_hours * 30
                            );
                            list.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                flex_shrink: 0.,
                                ..default()
                            })
                            .with_children(|card| {
                                card.spawn((
                                    Action::Select(replay.id.clone()),
                                    RelativeCursorPosition::default(),
                                    BackgroundColor(BUTTON),
                                    Node {
                                        flex_direction: FlexDirection::Column,
                                        padding: UiRect::axes(px(11), px(7)),
                                        border_radius: BorderRadius::all(px(6)),
                                        ..default()
                                    },
                                ))
                                .with_children(|label| {
                                    label.spawn(text(replay.title, 19., PAPER));
                                    label.spawn(text(timing, 16., MUTED));
                                });
                                card.spawn((
                                    Detail(replay.id.clone()),
                                    Node {
                                        display: Display::None,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Start,
                                        padding: UiRect::axes(px(11), px(8)),
                                        row_gap: px(7),
                                        ..default()
                                    },
                                ))
                                .with_children(|detail| {
                                    detail.spawn(text(replay.analyze, 16., MUTED));
                                    detail
                                        .spawn((
                                            Action::Play(replay.id),
                                            RelativeCursorPosition::default(),
                                            BackgroundColor(BUTTON),
                                            Node {
                                                padding: UiRect::axes(px(11), px(5)),
                                                border_radius: BorderRadius::all(px(6)),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|button| {
                                            button.spawn(text("Reproducir", 17., PAPER));
                                        });
                                });
                            });
                        }
                    });
            }
            Reply::Catalogue(Err(error)) => {
                status.0 = "No se pudo abrir el historial.".into();
                warn!("Experiment history: {error}");
            }
            Reply::Playback(result) => {
                history.playing = false;
                match result {
                    Ok(()) => {
                        status.0.clear();
                        started.write(ReplayStarted);
                    }
                    Err(error) => {
                        status.0 = "No se pudo reproducir el experimento.".into();
                        warn!("Experiment replay: {error}");
                    }
                }
            }
        }
    }
}

fn scroll(
    mut wheel: MessageReader<MouseWheel>,
    list: Single<(&RelativeCursorPosition, &ComputedNode, &mut ScrollPosition), With<Catalogue>>,
) {
    let (cursor, node, mut position) = list.into_inner();
    let delta: f32 = wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y * 33.,
            MouseScrollUnit::Pixel => event.y,
        })
        .sum();
    if cursor.cursor_over() && delta != 0. {
        let max = ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
        position.y = (position.y - delta).clamp(0., max);
    }
}
