use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    text::FontSize,
    ui::RelativeCursorPosition,
    window::PrimaryWindow,
};
use serde::Deserialize;
use std::{env, fs, io::ErrorKind, path::PathBuf, time::Duration};

const PAPER: Color = Color::srgb(0.97, 0.95, 0.88);
const MUTED: Color = Color::srgb(0.75, 0.81, 0.88);
const ACCENT: Color = Color::srgb(0.55, 0.85, 0.76);

pub(crate) struct ResultDisplayPlugin;

impl Plugin for ResultDisplayPlugin {
    fn build(&self, app: &mut App) {
        if let Some(path) = env::var_os("SANTIAGO_RESULT_DISPLAY_FILE") {
            app.insert_resource(Feed {
                path: path.into(),
                timer: Timer::new(Duration::from_millis(250), TimerMode::Repeating),
                previous: None,
                dismissed: false,
            })
            .add_systems(Update, (read_receipt, interact).chain());
        }
    }
}

#[derive(Component)]
pub(crate) struct ResultSurface;

#[derive(Component)]
struct Dismiss;

#[derive(Component)]
struct Body;

#[derive(Resource)]
struct Feed {
    path: PathBuf,
    timer: Timer,
    previous: Option<Result<Vec<u8>, ErrorKind>>,
    dismissed: bool,
}

#[derive(Deserialize)]
struct Receipt {
    receipt_id: String,
    cards: Vec<Card>,
}

impl Receipt {
    fn valid(&self) -> bool {
        !self.receipt_id.is_empty()
            && self.cards.iter().all(|card| {
                let has_values = matches!(card.status, Status::Result | Status::Partial);
                card.rows
                    .iter()
                    .all(|row| has_values && row.values.len() == card.columns.len().max(1))
            })
    }
}

#[derive(Deserialize)]
struct Card {
    title: String,
    source: String,
    status: Status,
    #[serde(default)]
    unit: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    coverage: String,
    #[serde(default)]
    columns: Vec<String>,
    #[serde(default)]
    rows: Vec<Row>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Pending,
    Result,
    Partial,
    Unavailable,
}

impl Status {
    fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Por confirmar",
            Self::Result => "Resultado",
            Self::Partial => "Cobertura parcial",
            Self::Unavailable => "Sin datos",
        }
    }
}

#[derive(Deserialize)]
struct Row {
    label: String,
    values: Vec<String>,
    bar: Option<f32>,
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

fn read_receipt(
    mut commands: Commands,
    time: Res<Time>,
    mut feed: ResMut<Feed>,
    old: Query<Entity, With<ResultSurface>>,
) {
    if !feed.timer.tick(time.delta()).just_finished() {
        return;
    }
    let snapshot = fs::read(&feed.path).map_err(|error| error.kind());
    if feed.previous.as_ref() == Some(&snapshot) {
        return;
    }
    // Every publication carries an ID, including a repeat of the same query.
    let cards = match &snapshot {
        Ok(bytes) => serde_json::from_slice::<Receipt>(bytes)
            .ok()
            .filter(Receipt::valid)
            .map_or_else(unreadable_receipt, |receipt| receipt.cards),
        Err(ErrorKind::NotFound) => Vec::new(),
        Err(_) => unreadable_receipt(),
    };
    feed.previous = Some(snapshot);
    feed.dismissed = false;
    for entity in &old {
        commands.entity(entity).despawn();
    }
    if cards.is_empty() {
        return;
    }

    commands
        .spawn((
            ResultSurface,
            RelativeCursorPosition::default(),
            GlobalZIndex(18),
            BackgroundColor(crate::label::PANEL_INK),
            BorderColor::all(Color::srgb(0.34, 0.44, 0.53)),
            Node {
                position_type: PositionType::Absolute,
                top: px(156),
                left: px(24),
                width: vw(31),
                min_width: px(290),
                max_width: px(470),
                max_height: vh(48),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(8)),
                padding: UiRect::all(px(15)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
        ))
        .with_children(|panel| {
            panel
                .spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|header| {
                    header.spawn(text("Consulta", 21., PAPER));
                    header
                        .spawn((
                            Dismiss,
                            BackgroundColor(Color::srgb(0.19, 0.25, 0.35)),
                            RelativeCursorPosition::default(),
                            Node {
                                padding: UiRect::axes(px(9), px(4)),
                                border_radius: BorderRadius::all(px(5)),
                                ..default()
                            },
                        ))
                        .with_children(|button| {
                            button.spawn(text("Cerrar", 16., MUTED));
                        });
                });
            panel
                .spawn((
                    Body,
                    ScrollPosition::default(),
                    Node {
                        min_height: px(0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll_y(),
                        row_gap: px(13),
                        ..default()
                    },
                ))
                .with_children(|body| {
                    for card in cards {
                        body.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            flex_shrink: 0.,
                            row_gap: px(5),
                            ..default()
                        })
                        .with_children(|section| render_card(section, card));
                    }
                });
        });
}

fn unreadable_receipt() -> Vec<Card> {
    vec![Card {
        title: "No se pudo leer el resultado".into(),
        source: "Consulta".into(),
        status: Status::Unavailable,
        unit: String::new(),
        scope: String::new(),
        coverage: String::new(),
        columns: Vec::new(),
        rows: Vec::new(),
    }]
}

fn render_card(section: &mut ChildSpawnerCommands, card: Card) {
    section.spawn(text(
        format!("{} · {}", card.status.label(), card.source),
        16.,
        ACCENT,
    ));
    section.spawn(text(card.title, 24., PAPER));
    for detail in [card.unit, card.scope, card.coverage] {
        if !detail.is_empty() {
            section.spawn(text(detail, 17., MUTED));
        }
    }
    if !card.columns.is_empty() {
        render_row(section, String::new(), card.columns, 15., MUTED);
    }
    for row in card.rows {
        render_row(section, row.label, row.values, 17., PAPER);
        if let Some(bar) = row.bar {
            section.spawn((
                BackgroundColor(ACCENT),
                Node {
                    width: percent(bar.clamp(0., 1.) * 100.),
                    height: px(5),
                    ..default()
                },
            ));
        }
    }
}

fn render_row(
    section: &mut ChildSpawnerCommands,
    label: String,
    values: Vec<String>,
    size: f32,
    color: Color,
) {
    section
        .spawn(Node {
            column_gap: px(8),
            padding: UiRect::vertical(px(2)),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                text(label, size, color),
                Node {
                    flex_grow: 2.,
                    flex_basis: px(0),
                    min_width: px(0),
                    ..default()
                },
            ));
            for value in values {
                row.spawn((
                    text(value, size, color),
                    Node {
                        flex_grow: 1.,
                        flex_basis: px(0),
                        min_width: px(0),
                        ..default()
                    },
                    TextLayout::justify(Justify::Right),
                ));
            }
        });
}

fn interact(
    mut feed: ResMut<Feed>,
    mut panel: Query<(&mut Node, &RelativeCursorPosition), With<ResultSurface>>,
    dismiss: Query<&RelativeCursorPosition, With<Dismiss>>,
    mut body: Query<&mut ScrollPosition, With<Body>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let Ok((mut node, cursor)) = panel.single_mut() else {
        wheel.clear();
        return;
    };
    if node.display != Display::None
        && mouse.just_pressed(MouseButton::Left)
        && dismiss.iter().any(RelativeCursorPosition::cursor_over)
    {
        feed.dismissed = true;
    }
    let display = if feed.dismissed {
        Display::None
    } else {
        Display::Flex
    };
    if node.display != display {
        node.display = display;
    }
    if display == Display::None || !cursor.cursor_over() {
        wheel.clear();
        return;
    }
    let delta = wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y * 27.,
            MouseScrollUnit::Pixel => event.y / window.resolution.base_scale_factor(),
        })
        .sum::<f32>();
    if delta != 0.
        && let Ok(mut position) = body.single_mut()
    {
        position.y = (position.y - delta).max(0.);
    }
}
