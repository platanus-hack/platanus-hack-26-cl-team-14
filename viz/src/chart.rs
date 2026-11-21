//! Final replay chart: stacked bars from the `chart` block that `run_replay` adds to a cached result.
use crate::net::Scenario;
use bevy::{prelude::*, text::FontSize};

pub struct ChartPlugin;
impl Plugin for ChartPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_font)
            .add_systems(Update, draw.run_if(resource_changed::<Scenario>));
    }
}

#[derive(Resource)]
struct ChartFont(Handle<Font>);

#[derive(Component)]
struct Chart;

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

fn draw(
    mut commands: Commands,
    scenario: Res<Scenario>,
    font: Res<ChartFont>,
    old: Query<Entity, With<Chart>>,
) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let chart = &scenario.0["chart"];
    let Some(bars) = chart["bars"].as_array() else { return };
    let text = |value: String, size: f32| {
        (
            Text::new(value),
            TextFont { font: font.0.clone().into(), font_size: FontSize::Px(size), ..default() },
            TextColor(Color::WHITE),
        )
    };
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
            Chart,
            BackgroundColor(Color::srgba(0.05, 0.07, 0.09, 0.85)),
            Node {
                position_type: PositionType::Absolute,
                right: px(24),
                bottom: px(48),
                width: px(780),
                padding: UiRect::all(px(28)),
                flex_direction: FlexDirection::Column,
                row_gap: px(18),
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn(text(chart["title"].as_str().unwrap().to_owned(), 46.));
            panel.spawn(text(chart["unit"].as_str().unwrap().to_owned(), 26.));
            for (label, values) in &bars {
                let total: f64 = values.iter().map(|(_, v)| v).sum();
                panel.spawn(text(format!("{label} · {}", thousands(total)), 34.));
                panel
                    .spawn(Node { width: percent(100), height: px(56), ..default() })
                    .with_children(|row| {
                        for (key, value) in values {
                            let color = COLORS[keys.iter().position(|k| k == key).unwrap_or(0) % COLORS.len()];
                            row.spawn((
                                BackgroundColor(color),
                                Node { width: percent((value / max * 100.) as f32), height: percent(100), ..default() },
                            ));
                        }
                    });
            }
            // A single "total" key needs no legend.
            if keys.len() > 1 {
                panel
                    .spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: px(24), row_gap: px(8), ..default() })
                    .with_children(|legend| {
                        for (i, key) in keys.iter().enumerate() {
                            legend
                                .spawn(Node { column_gap: px(10), align_items: AlignItems::Center, ..default() })
                                .with_children(|item| {
                                    item.spawn((
                                        BackgroundColor(COLORS[i % COLORS.len()]),
                                        Node { width: px(26), height: px(26), ..default() },
                                    ));
                                    item.spawn(text(key.clone(), 28.));
                                });
                        }
                    });
            }
        });
}
