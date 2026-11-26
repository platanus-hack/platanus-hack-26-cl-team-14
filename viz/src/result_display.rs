use crate::chart::ChartFont;
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    text::FontSize,
    ui::RelativeCursorPosition,
    window::PrimaryWindow,
};
use serde::Deserialize;
use std::{env, io::ErrorKind, path::PathBuf, time::Duration};

// One style: white text on the print blue.
const PAPER: Color = Color::WHITE;
const MUTED: Color = Color::WHITE;
const ACCENT: Color = Color::WHITE;

pub(crate) struct ResultDisplayPlugin;

impl Plugin for ResultDisplayPlugin {
    fn build(&self, app: &mut App) {
        // Browser: no environment. The page keeps the file that the MCP server writes.
        let path = if cfg!(target_family = "wasm") {
            Some("result-display.json".into())
        } else {
            env::var_os("SANTIAGO_RESULT_DISPLAY_FILE")
        };
        if let Some(path) = path {
            app.insert_resource(Feed {
                path: path.into(),
                timer: Timer::new(Duration::from_millis(250), TimerMode::Repeating),
                previous: None,
                dismissed: false,
            })
            .add_systems(Update, (read_receipt, interact, show_more).chain());
        }
    }
}

#[derive(Component)]
pub(crate) struct ResultSurface;

#[derive(Component)]
struct Dismiss;

#[derive(Component)]
struct Body;

/// The line that tells the reader that the card has more rows below.
#[derive(Component)]
struct More;

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

fn text(font: &Handle<Font>, value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

fn read_receipt(
    mut commands: Commands,
    time: Res<Time>,
    font: Res<ChartFont>,
    mut feed: ResMut<Feed>,
    old: Query<Entity, With<ResultSurface>>,
) {
    if !feed.timer.tick(time.delta()).just_finished() {
        return;
    }
    let snapshot = crate::data::read(&feed.path).map_err(|error| error.kind());
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

    let font = &font.0;
    commands
        .spawn((
            ResultSurface,
            RelativeCursorPosition::default(),
            GlobalZIndex(18),
            BackgroundColor(crate::label::PANEL_INK),
            BorderColor::all(crate::label::GLASS_RIM),
            Node {
                position_type: PositionType::Absolute,
                top: px(156),
                left: px(24),
                width: vw(44),
                min_width: px(480),
                max_width: px(900),
                max_height: vh(80),
                border: UiRect::all(px(1)),
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
                    header.spawn(text(font, "Consulta", 38., PAPER));
                    header
                        .spawn((
                            Dismiss,
                            BackgroundColor(Color::WHITE),
                            RelativeCursorPosition::default(),
                            Node {
                                padding: UiRect::axes(px(9), px(4)),
                                ..default()
                            },
                        ))
                        .with_children(|button| {
                            button.spawn(text(font, "Cerrar", 34., crate::label::PANEL_INK));
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
                        .with_children(|section| render_card(section, font, card));
                    }
                });
            panel.spawn((More, Node { display: Display::None, ..default() }, text(font, "Desliza para ver más", 30., PAPER)));
        });
}

// The card scrolls with the wheel. Without this line a cut row looks like the end of the result.
fn show_more(body: Query<&ComputedNode, With<Body>>, mut more: Query<&mut Node, With<More>>) {
    let (Ok(body), Ok(mut more)) = (body.single(), more.single_mut()) else {
        return;
    };
    let display = if body.content_size().y > body.size().y + 1. { Display::Flex } else { Display::None };
    if more.display != display {
        more.display = display;
    }
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

fn render_card(section: &mut ChildSpawnerCommands, font: &Handle<Font>, card: Card) {
    section.spawn(text(
        font,
        format!("{} · {}", card.status.label(), card.source),
        32.,
        ACCENT,
    ));
    section.spawn(text(font, card.title, 46., PAPER));
    for detail in [card.unit, card.coverage] {
        if !detail.is_empty() {
            section.spawn(text(font, detail, 34., MUTED));
        }
    }
    if !card.columns.is_empty() {
        render_row(section, font, String::new(), card.columns, 32., MUTED);
    }
    for row in card.rows {
        render_row(section, font, row.label, row.values, 38., PAPER);
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
    if !card.scope.is_empty() {
        section.spawn(text(font, card.scope, 34., MUTED));
    }
}

fn render_row(
    section: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
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
                text(font, label, size, color),
                Node {
                    flex_grow: 2.,
                    flex_basis: px(0),
                    min_width: px(0),
                    ..default()
                },
            ));
            for value in values {
                row.spawn((
                    text(font, value, size, color),
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
