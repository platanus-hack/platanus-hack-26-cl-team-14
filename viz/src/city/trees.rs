use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// Four silhouettes share a print sheet. Dots get denser on the shaded side.
pub(super) fn canopies() -> Image {
    const SIDE: usize = 40;
    let mut rgba = Vec::with_capacity(4 * SIDE * SIDE * 4);
    for y in 0..SIDE {
        for column in 0..4 * SIDE {
            let x = column % SIDE;
            let shape = (column / SIDE) as f32;
            let u = (x as f32 + 0.5) / SIDE as f32 * 2. - 1.;
            let v = ((y as f32 + 0.5) / SIDE as f32 - 0.39) / 0.37;
            let angle = v.atan2(u);
            let radius = (u * u + v * v).sqrt();
            let edge = 0.88 + 0.06 * (3. * angle + shape).sin() + 0.05 * (5. * angle - shape).cos();
            let hash = ((column / 2) as u32)
                .wrapping_mul(73856093)
                .wrapping_add(((y / 2) as u32).wrapping_mul(19349663));
            let noise = (hash.wrapping_mul(hash ^ 0x85ebca6b) >> 24) as f32 / 255.;
            let density = 0.25 + 0.38 * (0.5 * u + 0.5) + 0.15 * (v + 0.4).max(0.);
            let texel = if radius < edge && noise < density {
                [48, 111, 69, 255]
            } else if radius < edge - 0.08 {
                [128, 177, 137, 255]
            } else if u.abs() < 0.045 && y > SIDE / 2 {
                [68, 92, 80, 255]
            } else {
                [0, 0, 0, 0]
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
