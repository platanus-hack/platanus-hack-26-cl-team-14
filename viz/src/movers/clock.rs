//! Top-left panel with the replay day and time, sized for a projector.
use super::{DAY, FixedClock, PlaybackRate, Replay};
use bevy::{prelude::*, text::FontSize};

pub(crate) struct ClockPlugin;
impl Plugin for ClockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn).add_systems(Update, show);
    }
}

#[derive(Component)]
struct ClockText;

fn spawn(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(
        include_bytes!("../../assets/barlow/Barlow-Regular.ttf").to_vec(),
    ));
    commands.spawn((
        ClockText,
        Text::default(),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(44.0),
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.05, 0.07, 0.09, 0.78)),
        Node {
            position_type: PositionType::Absolute,
            top: px(24),
            left: px(24),
            padding: UiRect::axes(px(22), px(10)),
            ..default()
        },
        Visibility::Hidden,
    ));
}

// Same clock as `draw`. The engine models an average weekday, so the panel names no date.
fn show(
    time: Res<Time>,
    replay: Res<Replay>,
    fixed: Res<FixedClock>,
    rate: Res<PlaybackRate>,
    mut night: ResMut<crate::Night>,
    mut text: Single<(&mut Text, &mut Visibility), With<ClockText>>,
) {
    let Some(now) = fixed
        .0
        .or(replay.clock_offset.map(|offset| offset + time.elapsed_secs() * rate.0))
    else {
        return;
    };
    let minutes = (now.rem_euclid(DAY) / 60.) as u32;
    night.set_if_neq(crate::Night(!(6 * 60..20 * 60).contains(&minutes)));
    text.0.0 = format!("Día hábil · {:02}:{:02}", minutes / 60, minutes % 60);
    *text.1 = Visibility::Inherited;
}
