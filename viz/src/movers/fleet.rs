//! Display-only samples from issuing-comuna permit shares and the regional listing proxy.
use super::{Model, Models, Paint, sprite_stack};
use bevy::{math::Affine2, prelude::*};
use std::collections::HashMap;

type Shares = HashMap<String, Vec<(u16, f64)>>;

#[derive(Resource)]
pub(super) struct Fleet {
    classes: Shares,
    colours: Shares,
    swatches: u16,
    passenger_bodies: u16,
}

impl Fleet {
    pub(super) fn load(
        models: &mut Models,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
        let ([styles, swatches, passenger_bodies, columns, slices], [classes, colours]): (
            [u16; 5],
            [Shares; 2],
        ) = serde_json::from_slice(include_bytes!("../../assets/car_fleet.json")).unwrap();
        let rows = styles.div_ceil(columns);
        let (mesh, mut image) = sprite_stack(
            include_bytes!("../../assets/car_fleet.png"),
            slices.into(),
            [columns.into(), rows.into()],
        );
        let mesh = meshes.add(mesh);
        if models.ink {
            // Keep the source profile, cutouts and dark tyres. Paper windows
            // separate the cabin from the bonnet at street zoom.
            for pixel in image.data.as_mut().unwrap().chunks_exact_mut(4) {
                if pixel[3] != 0 {
                    let ink = match pixel[0..3] {
                        [43, 62, 83] | [91, 126, 151] | [255, 225, 152] => super::PAPER,
                        [24, 33, 45] | [41, 48, 57] => [24, 33, 45],
                        _ => super::CAR_INK,
                    };
                    pixel[..3].copy_from_slice(&ink);
                }
            }
        }
        let texture = images.add(image);
        let step = Vec2::new(1. / columns as f32, 1. / rows as f32);
        for style in 0..styles {
            let cell = Vec2::new((style % columns) as f32, (style / columns) as f32);
            let model = Model::Car(style);
            models.mesh.insert(model, mesh.clone());
            models.paint.insert(
                (model, Paint::Plain),
                materials.add(StandardMaterial {
                    base_color_texture: Some(texture.clone()),
                    uv_transform: Affine2::from_scale_angle_translation(step, 0., cell * step),
                    unlit: true,
                    alpha_mode: AlphaMode::Mask(0.5),
                    ..default()
                }),
            );
        }
        Self {
            classes,
            colours,
            swatches,
            passenger_bodies,
        }
    }

    pub(super) fn cars(&self, cut: &str, id: u64, count: usize) -> Vec<u16> {
        let classes = self.classes.get(cut).unwrap_or(&self.classes["13"]);
        let proxy = &self.colours["13"];
        let passenger_colours = self.colours.get(cut).unwrap_or(proxy);
        (0..count)
            .map(|copy| {
                let key = id.wrapping_add((copy as u64).wrapping_mul(0x9e3779b97f4a7c15));
                let body = sample(classes, key);
                // Pudahuel's municipal colour sample excludes vans, pickups and minibuses.
                let colours = if body < self.passenger_bodies {
                    passenger_colours
                } else {
                    proxy
                };
                let swatch = sample(colours, key ^ 0xd1b54a32d192ed03);
                body * self.swatches + swatch
            })
            .collect()
    }
}

// SplitMix64 finalizer. Separate body and colour streams depend only on engine id and copy.
fn sample(distribution: &[(u16, f64)], mut key: u64) -> u16 {
    key = (key ^ (key >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    key = (key ^ (key >> 27)).wrapping_mul(0x94d049bb133111eb);
    let draw = ((key ^ (key >> 31)) >> 11) as f64 / ((1u64 << 53) as f64);
    distribution[distribution.partition_point(|&(_, upper)| upper <= draw)].0
}
