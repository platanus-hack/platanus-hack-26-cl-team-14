use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Scenario(pub serde_json::Value);
