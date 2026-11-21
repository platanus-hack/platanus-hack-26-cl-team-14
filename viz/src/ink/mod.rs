//! Ink Studio splats. See LICENSE for the upstream MIT notice.
pub mod animation;
pub mod raster;
pub mod sculpt;

use bevy::{
    image::ImageSampler,
    prelude::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// Render one frame on the art-pixel grid. Bevy keeps each texel sharp when enlarged.
pub fn render_image(
    splats: &[sculpt::Splat],
    camera: raster::Orbit,
    palette: &[[u8; 3]],
    dither: f64,
    width: u32,
    height: u32,
) -> Image {
    let rgba = raster::render(splats, camera, palette, dither, width, height);
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}
