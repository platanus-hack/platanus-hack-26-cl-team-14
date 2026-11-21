use super::{Rgba, Surface};
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// One wall module in the miniature. Mip levels blend the panes into a facade at a distance.
pub(super) fn window_texture() -> Image {
    let mut level = Vec::with_capacity(64 * 64 * 4);
    for y in 0..64 {
        for x in 0..64 {
            let color = match (x, y) {
                (8..=55, 9..=45) if x == 8 || x == 55 || y == 9 || y == 45 => [109, 111, 115],
                (10..=53, 12..=43) if x == 32 || x == 33 => [165, 174, 177],
                (10..=53, 12..=43) if y < 16 => [54, 71, 84],
                (10..=53, 12..=43) if (x + y) % 64 < 22 => [125, 153, 167],
                (10..=53, 12..=43) => [84, 113, 130],
                (6..=57, 46..=48) => [224, 224, 214],
                (6..=57, 49..=50) => [141, 141, 136],
                (_, 59..=60) => [233, 231, 221],
                _ => [255, 255, 255],
            };
            level.extend([color[0], color[1], color[2], 255]);
        }
    }
    let size = Extent3d {
        width: 64,
        height: 64,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        level.clone(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    let mut width = 64;
    while width > 1 {
        let half = width / 2;
        let mut next = Vec::with_capacity(half * half * 4);
        for y in 0..half {
            for x in 0..half {
                for ch in 0..4 {
                    let index = (2 * y * width + 2 * x) * 4 + ch;
                    let sum = [index, index + 4, index + width * 4, index + width * 4 + 4]
                        .map(|i| level[i] as u16)
                        .into_iter()
                        .sum::<u16>();
                    next.push((sum / 4) as u8);
                }
            }
        }
        image.data.as_mut().unwrap().extend(&next);
        image.texture_descriptor.mip_level_count += 1;
        level = next;
        width = half;
    }
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        min_filter: ImageFilterMode::Linear,
        mag_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

fn roof_face(surface: &mut Surface, points: &[Vec3], color: Rgba) {
    let first = surface.positions.len() as u32;
    let mut normal = (points[1] - points[0])
        .cross(points[2] - points[0])
        .normalize_or_zero();
    if normal.y < 0. {
        normal = -normal;
    }
    for p in points {
        surface.vertex(*p, normal, color);
    }
    for i in 1..points.len() as u32 - 1 {
        surface.indices.extend([first, first + i, first + i + 1]);
    }
}

pub(super) fn building(
    surface: &mut Surface,
    rings: &[Vec<Vec3>],
    height: f32,
    wall: Rgba,
    roof: Rgba,
) {
    let ring = &rings[0];
    if rings.len() == 1
        && ring.len() == 4
        && super::convex_quad(ring)
        && height < 10.
        && ring[1].distance(ring[0]).min(ring[2].distance(ring[1])) < 23.
    {
        // Four-sided low buildings get a model roof. The ridge stays at the supplied height.
        let first =
            usize::from(ring[1].distance_squared(ring[0]) > ring[2].distance_squared(ring[1]));
        let [a, b, c, d] = [0, 1, 2, 3].map(|i| ring[(first + i) % 4]);
        let rise = (a.distance(b) * 0.21).min(height * 0.33);
        let eave = Vec3::Y * (height - rise);
        let ridge_a = (a + b) * 0.5 + Vec3::Y * height;
        let ridge_c = (c + d) * 0.5 + Vec3::Y * height;
        surface.polygon(rings, 0., height - rise, wall, roof);
        roof_face(surface, &[a + eave, b + eave, ridge_a], wall);
        roof_face(surface, &[c + eave, d + eave, ridge_c], wall);
        roof_face(surface, &[a + eave, d + eave, ridge_c, ridge_a], roof);
        roof_face(surface, &[b + eave, c + eave, ridge_c, ridge_a], roof);
    } else {
        surface.polygon(rings, 0., height, wall, roof);
        // A raised parapet catches the light at the edge of larger flat roofs.
        if height >= 10. {
            let lip = (height * 0.012).clamp(0.18, 0.65);
            for ring in rings {
                for (a, b) in ring.iter().zip(ring.iter().cycle().skip(1)) {
                    let u = (*b - *a).normalize_or_zero();
                    let side = Vec3::new(-u.z, 0., u.x) * 0.23;
                    let top = Vec3::Y * (height + lip);
                    roof_face(
                        surface,
                        &[
                            *a + side + top,
                            *b + side + top,
                            *b - side + top,
                            *a - side + top,
                        ],
                        [0.62, 0.61, 0.55, 1.],
                    );
                }
            }
        }
    }
}
