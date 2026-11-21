//! Large label of a side-by-side tile, from the `label` that tools/replay-relay.mjs adds.
use crate::net::Scenario;
use bevy::{prelude::*, text::FontSize};

pub struct LabelPlugin;
impl Plugin for LabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw.run_if(resource_changed::<Scenario>));
    }
}

#[derive(Component)]
struct Label;

fn draw(mut commands: Commands, scenario: Res<Scenario>, mut fonts: ResMut<Assets<Font>>, old: Query<Entity, With<Label>>) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let Some(label) = scenario.0["label"].as_str() else { return };
    let font = fonts.add(Font::from_bytes(include_bytes!("../assets/barlow/Barlow-Regular.ttf").to_vec()));
    commands
        .spawn((
            Label,
            Node {
                position_type: PositionType::Absolute,
                top: px(24),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont { font: font.into(), font_size: FontSize::Px(56.), ..default() },
                TextColor(Color::WHITE),
                BackgroundColor(Color::srgba(0.05, 0.07, 0.09, 0.85)),
                Node { padding: UiRect::axes(px(28), px(12)), ..default() },
            ));
        });
}
