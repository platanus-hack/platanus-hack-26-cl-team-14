mod scene;

use santiago_miniature::ink::{raster::Orbit, render_image};

fn main() {
    let output = std::env::args_os().nth(1).expect("Pass an output PNG path");
    let splats = scene::carved_stone();
    let camera = Orbit {
        target: [0., 1.4, 0.],
        distance: 12.,
        yaw: 0.45,
        pitch: 0.2,
    };
    render_image(&splats, camera, &scene::PALETTE, 0.25, 160, 120)
        .try_into_dynamic()
        .unwrap()
        .save(output)
        .unwrap();
    println!("Rendered {} splats", splats.len());
}
