use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// Four crown slices seen from above share a print sheet, drawn as lobed blobs filled with a regular
/// halftone dot grid, like the stippled trees of an architectural axonometric. Dots grow on the shaded
/// south-east side and toward the rim.
pub(super) fn crowns() -> Image {
    const SIDE: usize = 64;
    const CELL: usize = 4;
    let mut rgba = Vec::with_capacity(4 * SIDE * SIDE * 4);
    for y in 0..SIDE {
        for column in 0..4 * SIDE {
            let x = column % SIDE;
            let shape = (column / SIDE) as f32;
            let u = (x as f32 + 0.5) / SIDE as f32 * 2. - 1.;
            let v = (y as f32 + 0.5) / SIDE as f32 * 2. - 1.;
            // Three or four overlapping lobes per shape.
            let lobe = |k: f32| {
                let a = k * 2.1 + shape * 1.3;
                let (cx, cy) = (0.34 * a.cos(), 0.34 * a.sin());
                ((u - cx).powi(2) + (v - cy).powi(2)).sqrt() / (0.58 + 0.06 * (k + shape).sin())
            };
            let inside = (0..3 + (shape as usize & 1)).map(|k| lobe(k as f32)).fold(f32::MAX, f32::min);
            // Halftone: one dot per cell, larger where the crown is shaded.
            let (cx, cy) = ((x / CELL * CELL) as f32 + CELL as f32 / 2., (y / CELL * CELL) as f32 + CELL as f32 / 2.);
            let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
            let shade = (0.30 + 0.35 * (0.5 * (u + v) + 0.5) + 0.35 * inside.min(1.)).clamp(0., 1.);
            let texel = if inside >= 1. {
                [0, 0, 0, 0]
            } else if d < 0.6 + 1.2 * shade {
                [38, 96, 66, 255]
            } else {
                [176, 214, 178, 255]
            };
            rgba.extend(texel);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: (4 * SIDE) as u32,
            height: SIDE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}
