//! Large label of a side-by-side tile, from the `label` that tools/replay-relay.mjs adds.
use crate::{focus::{self, CueCaption}, net::Scenario};
use bevy::{prelude::*, text::FontSize};

// The navy hatching ink of the city print (pixels/vote.wgsl), opaque so white text stays legible over white roofs.
pub(crate) const PANEL_INK: Color = Color::srgb_u8(0x50, 0x55, 0x7a);
// One type scale for the replay overlays, read from the back of a room.
pub(crate) const TYPE_LEAD: f32 = 40.;
pub(crate) const TYPE_HEAD: f32 = 32.;
pub(crate) const TYPE_NOTE: f32 = 26.;
// Secondary text on the ink panel.
pub(crate) const INK_MUTED: Color = Color::srgb_u8(0xda, 0xde, 0xef);
pub(crate) const PANEL_PAD: UiRect = UiRect { left: Val::Px(20.), right: Val::Px(20.), top: Val::Px(12.), bottom: Val::Px(12.) };
// Overlays stay left of this share of the window: the embedded chat opens over the right 36%.
pub(crate) const CLEAR_OF_CHAT: f32 = 58.;

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
    let font = fonts.add(Font::from_bytes(include_bytes!("../assets/inter/Inter-Regular.ttf").to_vec()));
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

fn spawn_caption(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(include_bytes!("../assets/inter/Inter-Regular.ttf").to_vec()));
    // The outer column sets the wrap width; text alone in an absolute node wraps at about half of it.
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: px(24),
            bottom: px(48),
            width: percent(CLEAR_OF_CHAT),
            ..default()
        })
        .with_child((
            CaptionBand,
            Text::default(),
            TextFont { font: font.into(), font_size: FontSize::Px(TYPE_LEAD), ..default() },
            TextColor(Color::WHITE),
            Visibility::Hidden,
            BackgroundColor(PANEL_INK),
            Node { padding: PANEL_PAD, ..default() },
        ));
}

fn show_caption(caption: Res<CueCaption>, mut band: Single<(&mut Text, &mut Visibility), With<CaptionBand>>) {
    let (text, visibility) = &mut *band;
    text.0.clone_from(&caption.0);
    **visibility = if caption.0.is_empty() { Visibility::Hidden } else { Visibility::Inherited };
}
