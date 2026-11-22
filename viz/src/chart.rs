//! Final replay chart: stacked bars from the `chart` block that `run_replay` adds to a cached result.
use crate::net::Scenario;
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

// High contrast on the dark panel, in stack order.
const COLORS: [Color; 6] = [
    Color::srgb(0.98, 0.76, 0.18),
    Color::srgb(0.30, 0.69, 0.98),
    Color::srgb(0.95, 0.36, 0.36),
    Color::srgb(0.45, 0.85, 0.50),
    Color::srgb(0.78, 0.56, 0.96),
    Color::srgb(0.85, 0.85, 0.85),
];

fn load_font(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(
        include_bytes!("../assets/barlow/Barlow-Regular.ttf").to_vec(),
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
            let values = values.iter().map(|(k, v)| (k.clone(), v.as_f64().unwrap())).collect();
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
            BackgroundColor(crate::label::PANEL_INK),
            Node {
                position_type: PositionType::Absolute,
                right: px(24),
                bottom: px(48),
                width: px(360),
                max_width: percent(29),
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn(text(chart["title"].as_str().unwrap().to_owned(), 30.));
            panel.spawn(text(chart["unit"].as_str().unwrap().to_owned(), 20.));
            for (label, values) in &bars {
                let total: f64 = values.iter().map(|(_, v)| v).sum();
                panel.spawn(text(format!("{label} · {}", thousands(total)), 24.));
                panel
                    .spawn(Node { width: percent(100), height: px(24), ..default() })
                    .with_children(|row| {
                        for (key, value) in values {
                            let color = COLORS[keys.iter().position(|k| k == key).unwrap_or(0) % COLORS.len()];
                            let share = (value / max * 100.) as f32;
                            row.spawn((
                                BackgroundColor(color),
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
                                    segment.spawn(ink(thousands(*value), 16., Color::BLACK));
                                }
                            });
                        }
                    });
            }
            // A single "total" key needs no legend.
            if keys.len() > 1 {
                panel
                    .spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: px(12), row_gap: px(4), ..default() })
                    .with_children(|legend| {
                        for (i, key) in keys.iter().enumerate() {
                            legend
                                .spawn(Node { column_gap: px(6), align_items: AlignItems::Center, ..default() })
                                .with_children(|item| {
                                    item.spawn((
                                        BackgroundColor(COLORS[i % COLORS.len()]),
                                        Node { width: px(14), height: px(14), ..default() },
                                    ));
                                    item.spawn(text(key.clone(), 18.));
                                });
                        }
                    });
            }
        });
}
