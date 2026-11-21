use bevy::math::{DMat3, DVec3};
use santiago_miniature::ink::{
    animation::pose_part,
    raster::Orbit,
    render_image,
    sculpt::{Shape, Solid, Splat, bake},
};
use serde::Deserialize;
use std::{f64::consts::PI, path::PathBuf};

#[derive(Deserialize)]
struct BoxPart {
    center: [f64; 3],
    half: [f64; 3],
    ramp: Vec<usize>,
}

#[derive(Deserialize)]
struct Camera {
    target: [f64; 3],
    dist: f64,
    pitch: f64,
    yaw: f64,
}

#[derive(Deserialize)]
struct Figure {
    seed: u32,
    palette: Vec<[u8; 3]>,
    camera: Camera,
    shoulder: [f64; 3],
    spacing: f64,
    fixed: Vec<BoxPart>,
    arm: BoxPart,
}

impl Figure {
    fn bake_boxes(&self, boxes: &[BoxPart]) -> Vec<Splat> {
        let ramps: Vec<Vec<_>> = boxes
            .iter()
            .map(|part| part.ramp.iter().map(|&i| self.palette[i]).collect())
            .collect();
        let solids: Vec<_> = boxes
            .iter()
            .zip(&ramps)
            .map(|(part, ramp)| Solid {
                shape: Shape::Box {
                    half_size: DVec3::from_array(part.half),
                },
                center: DVec3::from_array(part.center),
                subtract: false,
                spacing: self.spacing,
                ramp,
            })
            .collect();
        bake(&solids, self.seed)
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let directory = PathBuf::from(args.next().expect("Pass the output directory"));
    std::fs::create_dir_all(&directory).unwrap();
    let mut times: Vec<f64> = args.map(|arg| arg.parse().unwrap()).collect();
    if times.is_empty() {
        times = (0..8).map(|frame| f64::from(frame) * 0.25).collect();
    }

    let figure: Figure = serde_json::from_str(include_str!("figure.json")).unwrap();
    let mut posed = figure.bake_boxes(&figure.fixed);
    let first_arm = posed.len();
    let arm = figure.bake_boxes(std::slice::from_ref(&figure.arm));
    posed.extend_from_slice(&arm);
    let shoulder = DVec3::from_array(figure.shoulder);
    let camera = Orbit {
        target: figure.camera.target,
        distance: figure.camera.dist,
        pitch: figure.camera.pitch,
        yaw: figure.camera.yaw,
    };
    for (index, time) in times.into_iter().enumerate() {
        let angle = 0.43 * PI * (1. - (PI * time).cos());
        pose_part(
            &arm,
            &mut posed[first_arm..],
            DMat3::from_rotation_z(angle),
            shoulder,
            shoulder,
        );
        render_image(&posed, camera, &figure.palette, 0.25, 160, 120)
            .try_into_dynamic()
            .unwrap()
            .save(directory.join(format!("frame-{index:02}.png")))
            .unwrap();
    }
}
