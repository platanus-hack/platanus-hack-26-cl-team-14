//! Final replay chart: stacked bars from the `chart` block that `run_replay` adds to a cached result.
use crate::net::Scenario;
use crate::label::{INK_MUTED, PANEL_INK, PANEL_PAD, TYPE_HEAD, TYPE_NOTE};
use bevy::{prelude::*, text::FontSize, time::Real};

pub struct ChartPlugin;
impl Plugin for ChartPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_font)
            .add_systems(Update, (draw.run_if(resource_changed::<Scenario>), reveal).chain());
    }
}

#[derive(Resource)]
struct ChartFont(Handle<Font>);

// The chart appears at this elapsed time, so the map plays first.
#[derive(Component)]
struct Chart(f32);

// Numbers on the bright bars use the darkest map ink.
const ON_BAR: Color = Color::srgb_u8(0x24, 0x29, 0x42);

// Stack order and colors. Transit leads, so both bars compare it from the same left edge.
// The first three match the fan colors on the map (movers/events.rs).
const MODES: [(&str, Color); 5] = [
    ("Transporte público", Color::srgb(0.09, 0.90, 0.82)),
    ("Auto", Color::srgb(1., 0.69, 0.19)),
    ("Otro", Color::srgb(0.83, 0.64, 1.)),
    ("A pie", Color::srgb(0.98, 0.45, 0.45)),
    ("Bicicleta", Color::srgb(0.55, 0.85, 0.45)),
];

fn rank(key: &str) -> usize {
    MODES.iter().position(|(name, _)| *name == key).unwrap_or(MODES.len())
}

// A single "total" key takes the transit color.
fn color(key: &str) -> Color {
    MODES.get(rank(key)).map_or(MODES[0].1, |mode| mode.1)
}

fn load_font(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(
        include_bytes!("../assets/inter/Inter-Regular.ttf").to_vec(),
    ));
    commands.insert_resource(ChartFont(font));
}

fn thousands(value: f64) -> String {
    let digits = (value.round() as i64).to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('.');
        }
        out.push(c);
    }
    out
}

fn reveal(time: Res<Time<Real>>, mut charts: Query<(&Chart, &mut Visibility)>) {
    for (chart, mut visibility) in &mut charts {
        if *visibility == Visibility::Hidden && time.elapsed_secs() >= chart.0 {
            *visibility = Visibility::Inherited;
        }
    }
}

fn draw(
    mut commands: Commands,
    time: Res<Time<Real>>,
    scenario: Res<Scenario>,
    font: Res<ChartFont>,
    old: Query<Entity, With<Chart>>,
) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let chart = &scenario.0["chart"];
    let Some(bars) = chart["bars"].as_array() else { return };
    let ink = |value: String, size: f32, color: Color| {
        (
            Text::new(value),
            TextFont { font: font.0.clone().into(), font_size: FontSize::Px(size), ..default() },
            TextColor(color),
        )
    };
    let text = |value: String, size: f32| ink(value, size, Color::WHITE);
    let bars: Vec<(String, Vec<(String, f64)>)> = bars
        .iter()
        .map(|bar| {
            let values = bar["values"].as_object().unwrap();
            let mut values: Vec<_> = values.iter().map(|(k, v)| (k.clone(), v.as_f64().unwrap())).collect();
            values.sort_by_key(|(k, _)| rank(k));
            (bar["label"].as_str().unwrap().to_owned(), values)
        })
        .collect();
    let max = bars
        .iter()
        .map(|(_, values)| values.iter().map(|(_, v)| v).sum::<f64>())
        .fold(0., f64::max);
    let keys: Vec<String> = bars[0].1.iter().map(|(k, _)| k.clone()).collect();

    commands
        .spawn((
            Chart(time.elapsed_secs() + chart["show_after_seconds"].as_f64().unwrap_or(0.) as f32),
            Visibility::Hidden,
            BackgroundColor(PANEL_INK),
            Node {
                // Under the clock, in the same left column: the embedded chat covers the right side.
                position_type: PositionType::Absolute,
                left: px(24),
                top: px(104),
                width: px(520),
                max_width: percent(40),
                padding: UiRect::all(PANEL_PAD.left),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn(text(chart["title"].as_str().unwrap().to_owned(), TYPE_HEAD));
            panel.spawn(ink(chart["unit"].as_str().unwrap().to_owned(), TYPE_NOTE, INK_MUTED));
            for (label, values) in &bars {
                let total: f64 = values.iter().map(|(_, v)| v).sum();
                // Each label sits on its bar, with air before the next comparison.
                panel
                    .spawn(Node { margin: UiRect::top(px(10)), justify_content: JustifyContent::SpaceBetween, ..default() })
                    .with_children(|row| {
                        row.spawn(text(label.clone(), TYPE_HEAD));
                        row.spawn(text(thousands(total), TYPE_HEAD));
                    });
                panel
                    .spawn(Node { width: percent(100), height: px(36), column_gap: px(3), ..default() })
                    .with_children(|row| {
                        for (key, value) in values {
                            let share = (value / max * 100.) as f32;
                            row.spawn((
                                BackgroundColor(color(key)),
                                Node {
                                    width: percent(share),
                                    height: percent(100),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                            ))
                            .with_children(|segment| {
                                // A narrow segment has no room for its number.
                                if keys.len() > 1 && share >= 25. {
                                    segment.spawn(ink(thousands(*value), TYPE_NOTE, ON_BAR));
                                }
                            });
                        }
                    });
            }
            // A single "total" key needs no legend.
            if keys.len() > 1 {
                panel
                    .spawn(Node { margin: UiRect::top(px(12)), flex_wrap: FlexWrap::Wrap, column_gap: px(20), row_gap: px(6), ..default() })
                    .with_children(|legend| {
                        for key in &keys {
                            legend
                                .spawn(Node { column_gap: px(8), align_items: AlignItems::Center, ..default() })
                                .with_children(|item| {
                                    item.spawn((BackgroundColor(color(key)), Node { width: px(20), height: px(20), ..default() }));
                                    item.spawn(text(key.clone(), TYPE_NOTE));
                                });
                        }
                    });
            }
        });
}
