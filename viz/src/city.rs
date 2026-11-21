use bevy::prelude::*;

/// Local Santiago coordinates in meters. East is +X and south is +Z.
pub fn world(lon: f64, lat: f64) -> Vec3 {
    Vec3::new(((lon + 70.65) * 92_915.0) as f32, 0.0, (-(lat + 33.45) * 110_905.0) as f32)
}
