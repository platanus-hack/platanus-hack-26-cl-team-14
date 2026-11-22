//! Large label of a side-by-side tile, from the `label` that tools/replay-relay.mjs adds.
use crate::{focus::{self, CueCaption}, net::Scenario};
use bevy::{prelude::*, text::FontSize};

pub struct LabelPlugin;
impl Plugin for LabelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CueCaption>()
            .add_systems(Startup, spawn_caption)
            .add_systems(Update, (
                draw.run_if(resource_changed::<Scenario>),
                show_caption.after(focus::cue).run_if(resource_changed::<CueCaption>),
            ));
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
                // Leave the top row for the clock on narrow tiles.
                top: px(104),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                BackgroundColor(Color::srgba(0.05, 0.07, 0.09, 0.85)),
                Node {
                    max_width: percent(92),
                    padding: UiRect::axes(px(28), px(12)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(4),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(label),
                    TextFont { font: font.clone().into(), font_size: FontSize::Px(56.), ..default() },
                    TextColor(Color::WHITE),
                ));
                if let Some(detail) = scenario.0["label_detail"].as_str() {
                    panel.spawn((
                        Text::new(detail),
                        TextFont { font: font.into(), font_size: FontSize::Px(24.), ..default() },
                        TextColor(Color::WHITE),
                    ));
                }
            });
        });
}


#[derive(Component)]
struct CaptionBand;

fn spawn_caption(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(include_bytes!("../assets/barlow/Barlow-Regular.ttf").to_vec()));
    commands.spawn((
        CaptionBand,
        Text::default(),
        TextFont { font: font.into(), font_size: FontSize::Px(34.), ..default() },
        TextColor(Color::WHITE),
        Visibility::Hidden,
        BackgroundColor(Color::srgba(0.05, 0.07, 0.09, 0.90)),
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            bottom: px(48),
            max_width: percent(66),
            padding: UiRect::axes(px(16), px(10)),
            overflow: Overflow::clip(),
            ..default()
        },
    ));
}

fn show_caption(caption: Res<CueCaption>, mut band: Single<(&mut Text, &mut Visibility), With<CaptionBand>>) {
    let (text, visibility) = &mut *band;
    text.0.clone_from(&caption.0);
    **visibility = if caption.0.is_empty() { Visibility::Hidden } else { Visibility::Inherited };
}
