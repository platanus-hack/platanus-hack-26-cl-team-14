//! Large label of a side-by-side tile, from the `label` that tools/replay-relay.mjs adds.
use crate::{focus::{self, CueCaption, CueStep}, net::Scenario};
use accesskit::{Node as AccessNode, Role};
use bevy::{
    a11y::AccessibilityNode,
    prelude::*,
    text::FontSize,
    ui::RelativeCursorPosition,
};

// The darkest blue-violet ink of the city print, opaque so text stays legible over white roofs.
pub(crate) const PANEL_INK: Color = Color::srgb_u8(0x24, 0x29, 0x42);

// Glass panels: a translucent ink tint, a bright rim and a soft shadow. The city shows through.
pub(crate) const GLASS: Color = Color::srgba(0.07, 0.09, 0.16, 0.6);
pub(crate) const GLASS_RIM: Color = Color::srgba(1., 1., 1., 0.5);
pub(crate) fn glass_shadow() -> BoxShadow {
    BoxShadow::new(Color::srgba(0., 0., 0., 0.35), px(0), px(10), px(0), px(28))
}

pub struct LabelPlugin;
impl Plugin for LabelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CueCaption>()
            .init_resource::<CueStep>()
            .add_systems(Startup, spawn_caption)
            .add_systems(Update, (
                draw.run_if(resource_changed::<Scenario>),
                click_arrow.before(focus::cue),
                style_arrows.after(focus::cue),
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

// A caption arrow and its step: -1 back, 1 forward.
#[derive(Component)]
struct CueArrow(i32);

// The X that hides the captions and the arrows.
#[derive(Component)]
struct CueClose;

// Floating glass subtitle bar: the projector audience reads one large line per camera cue.
fn spawn_caption(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(include_bytes!("../assets/barlow/Barlow-Regular.ttf").to_vec()));
    commands
        .spawn((
            CaptionBand,
            Visibility::Hidden,
            BackgroundColor(GLASS),
            BorderColor::all(GLASS_RIM),
            glass_shadow(),
            Node {
                position_type: PositionType::Absolute,
                left: px(40),
                right: px(40),
                bottom: px(32),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(44)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|band| {
            // Step arrows above the band: a click works when the chat has the keyboard.
            band.spawn(Node {
                position_type: PositionType::Absolute,
                bottom: percent(100),
                right: px(0),
                column_gap: px(12),
                margin: UiRect::bottom(px(16)),
                ..default()
            })
            .with_children(|row| {
                for (step, glyph, label) in [(-1, "<", "Previous cue"), (1, ">", "Next cue")] {
                    let mut accessibility = AccessNode::new(Role::Button);
                    accessibility.set_label(label);
                    accessibility.set_disabled();
                    accessibility.set_hidden();
                    row.spawn((
                        CueArrow(step),
                        AccessibilityNode(accessibility),
                        RelativeCursorPosition::default(),
                        BackgroundColor(GLASS),
                        BorderColor::all(GLASS_RIM),
                        glass_shadow(),
                        Node {
                            width: px(96),
                            height: px(96),
                            border: UiRect::all(px(2)),
                            border_radius: BorderRadius::MAX,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|arrow| {
                        arrow.spawn((
                            Text::new(glyph),
                            TextFont { font: font.clone().into(), font_size: FontSize::Px(64.), ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
                row.spawn((
                    CueClose,
                    RelativeCursorPosition::default(),
                    BackgroundColor(GLASS),
                    BorderColor::all(GLASS_RIM),
                    glass_shadow(),
                    Node {
                        width: px(96),
                        height: px(96),
                        border: UiRect::all(px(2)),
                        border_radius: BorderRadius::MAX,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|close| {
                    close.spawn((
                        Text::new("×"),
                        TextFont { font: font.clone().into(), font_size: FontSize::Px(64.), ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            });
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

fn click_arrow(
    mouse: Res<ButtonInput<MouseButton>>,
    band: Single<&Visibility, With<CaptionBand>>,
    arrows: Query<(&CueArrow, &RelativeCursorPosition)>,
    close: Single<&RelativeCursorPosition, With<CueClose>>,
    mut step: ResMut<CueStep>,
) {
    if !mouse.just_pressed(MouseButton::Left) || **band == Visibility::Hidden {
        return;
    }
    if close.cursor_over() {
        step.close = true;
        return;
    }
    if let Some((arrow, _)) = arrows.iter().find(|(arrow, cursor)| {
        step.can_step(arrow.0) && cursor.cursor_over()
    }) {
        step.click = arrow.0;
    }
}

fn style_arrows(
    step: Res<CueStep>,
    caption: Res<CueCaption>,
    mut arrows: Query<(&CueArrow, &mut BackgroundColor, &mut AccessibilityNode)>,
) {
    let hidden = caption.0.is_empty();
    for (arrow, mut background, mut accessibility) in &mut arrows {
        let enabled = step.can_step(arrow.0);
        background.set_if_neq(BackgroundColor(if enabled {
            GLASS
        } else {
            Color::srgba(0.45, 0.47, 0.52, 0.3)
        }));
        if hidden && !accessibility.is_hidden() {
            accessibility.set_hidden();
        } else if !hidden && accessibility.is_hidden() {
            accessibility.clear_hidden();
        }
        if enabled && accessibility.is_disabled() {
            accessibility.clear_disabled();
        } else if !enabled && !accessibility.is_disabled() {
            accessibility.set_disabled();
        }
    }
}
