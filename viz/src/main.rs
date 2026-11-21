mod city;
mod net;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Santiago".into(),
                resolution: (1920, 1080).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<net::Scenario>()
        .run();
}
