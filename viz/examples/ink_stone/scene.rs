use bevy::math::DVec3;
use santiago_miniature::ink::sculpt::{Shape, Solid, Splat, bake};

pub const PALETTE: [[u8; 3]; 7] = [
    [0x08, 0x0b, 0x10],
    [0x26, 0x31, 0x3d],
    [0x56, 0x67, 0x78],
    [0x9b, 0xae, 0xb8],
    [0xe5, 0xe9, 0xdc],
    [0x38, 0x75, 0x48],
    [0x8f, 0xc7, 0x5e],
];

// The geometry, ramps, spacing, and seed match Ink Studio's examples/sculpt.js.
pub fn carved_stone() -> Vec<Splat> {
    let stone = &PALETTE[1..=4];
    bake(
        &[
            Solid {
                shape: Shape::Ground { radius: 4. },
                center: DVec3::ZERO,
                subtract: false,
                spacing: 0.18,
                ramp: &[PALETTE[1], PALETTE[1], PALETTE[2]],
            },
            Solid {
                shape: Shape::Box {
                    half_size: DVec3::new(1., 1.2, 1.),
                },
                center: DVec3::new(0., 1.2, 0.),
                subtract: false,
                spacing: 0.12,
                ramp: stone,
            },
            Solid {
                shape: Shape::Sphere { radius: 0.65 },
                center: DVec3::new(0., 1.4, 0.9),
                subtract: true,
                spacing: 0.09,
                ramp: stone,
            },
            Solid {
                shape: Shape::Sphere { radius: 0.5 },
                center: DVec3::new(0., 2.7, 0.),
                subtract: false,
                spacing: 0.1,
                ramp: &[PALETTE[1], PALETTE[5], PALETTE[6]],
            },
        ],
        7,
    )
}
