//! Large label of a side-by-side tile, from the `label` that tools/replay-relay.mjs adds.
use crate::{focus::{self, CueCaption}, net::Scenario};
use bevy::{prelude::*, text::FontSize};

// The darkest blue-violet ink of the city print, opaque so text stays legible over white roofs.
pub(crate) const PANEL_INK: Color = Color::srgb_u8(0x24, 0x29, 0x42);

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
                BackgroundColor(PANEL_INK),
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

#[derive(Component)]
struct CaptionText;

// Full-width subtitle bar: the projector audience reads one large line per camera cue.
fn spawn_caption(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(include_bytes!("../assets/barlow/Barlow-Regular.ttf").to_vec()));
    commands
        .spawn((
            CaptionBand,
            Visibility::Hidden,
            BackgroundColor(PANEL_INK),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(0),
                width: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|band| {
            // Accent strip in the red of the stadium stands.
            band.spawn((
                BackgroundColor(Color::srgb_u8(0xD6, 0x58, 0x4B)),
                Node { width: percent(100), height: px(10), ..default() },
            ));
            band.spawn((
                CaptionText,
                Text::default(),
                TextFont { font: font.into(), font_size: FontSize::Px(52.), ..default() },
                TextColor(Color::WHITE),
                Node { margin: UiRect::axes(px(40), px(26)), ..default() },
            ));
        });
}

fn show_caption(
    caption: Res<CueCaption>,
    mut band: Single<&mut Visibility, With<CaptionBand>>,
    mut text: Single<&mut Text, With<CaptionText>>,
) {
    text.0.clone_from(&caption.0);
    **band = if caption.0.is_empty() { Visibility::Hidden } else { Visibility::Inherited };
}
