use super::sculpt::Splat;

/// Perspective orbit in Ink Studio's world units and radians.
#[derive(Clone, Copy)]
pub struct Orbit {
    pub target: [f64; 3],
    pub distance: f64,
    pub yaw: f64,
    pub pitch: f64,
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn covariance_axis(covariance: [f32; 6], axis: [f64; 3]) -> [f64; 3] {
    let [xx, xy, xz, yy, yz, zz] = covariance.map(f64::from);
    [
        dot([xx, xy, xz], axis),
        dot([xy, yy, yz], axis),
        dot([xz, yz, zz], axis),
    ]
}

/// Render the native art grid in RGBA8, top row first.
/// The first palette color fills the background.
pub fn render(
    splats: &[Splat],
    camera: Orbit,
    palette: &[[u8; 3]],
    dither: f64,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let (sy, cy) = camera.yaw.sin_cos();
    let (sp, cp) = camera.pitch.sin_cos();
    let sign = cp.signum();
    let right = [sign * cy, 0.0, -sign * sy];
    let up = [-sp * sign * sy, sign * cp, -sp * sign * cy];
    let back = [cp * sy, sp, cp * cy];
    let eye = std::array::from_fn(|i| camera.target[i] + camera.distance * back[i]);
    let translation = [right, up, back].map(|axis| -dot(axis, eye));
    let near = (0.15 * camera.distance).max(0.4);
    let focal = f64::from(height) / (2.0 * 19.0_f64.to_radians().tan());
    let half_width = f64::from(width) / 2.0;
    let half_height = f64::from(height) / 2.0;

    let mut visible = Vec::new();
    for splat in splats {
        let point = splat.position.map(f64::from);
        let z = dot(back, point) + translation[2];
        if z <= -near && point[1] >= -30.0 && splat.density > 0.0 {
            visible.push((splat, z));
        }
    }
    // The reference stores sort keys in a Float32Array and preserves the order of ties.
    visible.sort_by(|(_, a), (_, b)| (*a as f32).total_cmp(&(*b as f32)));
    let background = palette[0].map(|byte| (f64::from(byte) / 255.0) as f32);
    let mut pixels = vec![background; width as usize * height as usize];
    for (splat, z) in visible {
        let point = splat.position.map(f64::from);
        let u = (dot(right, point) + translation[0]) / -z;
        let v = (dot(up, point) + translation[1]) / -z;
        let center_x = half_width + focal * u;
        let center_y = half_height + focal * v;
        // The perspective Jacobian uses a 30% guard band outside the viewport.
        let limit_x = 1.3 * half_width / focal;
        let limit_y = 1.3 * half_height / focal;
        let scale = focal / -z;
        let dx = std::array::from_fn(|i| scale * (right[i] + u.clamp(-limit_x, limit_x) * back[i]));
        let dy = std::array::from_fn(|i| scale * (up[i] + v.clamp(-limit_y, limit_y) * back[i]));
        let cx = covariance_axis(splat.covariance, dx);
        let cy = covariance_axis(splat.covariance, dy);
        let xx = dot(dx, cx) + 0.15;
        let xy = dot(dy, cx);
        let yy = dot(dy, cy) + 0.15;
        let det = xx * yy - xy * xy;
        if det <= 0.0 {
            continue;
        }
        let radius_x = (3.0 * xx.sqrt()).min(2048.0);
        let radius_y = (3.0 * yy.sqrt()).min(2048.0);
        let x0 = (center_x - radius_x).floor().max(0.0) as i32;
        let y0 = (center_y - radius_y).floor().max(0.0) as i32;
        let x1 = ((center_x + radius_x).ceil() as i32).min(width as i32 - 1);
        let y1 = ((center_y + radius_y).ceil() as i32).min(height as i32 - 1);
        let color = splat.color.map(|byte| f64::from(byte) / 255.0);
        for y in y0..=y1 {
            let py = f64::from(y) + 0.5 - center_y;
            for x in x0..=x1 {
                let px = f64::from(x) + 0.5 - center_x;
                let power = (-0.5 * (yy * px * px + xx * py * py) + xy * px * py) / det;
                if !(-4.5..=0.0).contains(&power) {
                    continue;
                }
                let weight = f64::from(splat.density) * power.exp();
                let alpha = if splat.density >= 1.0 {
                    let t = ((weight - 0.38) / (0.62 - 0.38)).clamp(0.0, 1.0);
                    0.99 * t * t * (3.0 - 2.0 * t)
                } else {
                    weight.min(0.99)
                };
                if alpha >= 0.004 {
                    let pixel = &mut pixels[y as usize * width as usize + x as usize];
                    for k in 0..3 {
                        pixel[k] = (color[k] * alpha + f64::from(pixel[k]) * (1.0 - alpha)) as f32;
                    }
                }
            }
        }
    }

    let colors: Vec<_> = palette
        .iter()
        .map(|color| color.map(|c| f64::from(c) / 255.0))
        .collect();
    let bayer = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
    let mut rgba = Vec::with_capacity(pixels.len() * 4);
    for y in (0..height as usize).rev() {
        for x in 0..width as usize {
            let rank = (f64::from(bayer[(x & 3) + 4 * (y & 3)]) + 0.5) / 16.0;
            let bias = (rank - 0.5) * 0.35 * 0.3 * dither;
            let rgb = pixels[y * width as usize + x].map(|c| (f64::from(c) + bias).clamp(0.0, 1.0));
            let mut nearest = 0;
            let mut min_error = f64::INFINITY;
            for (i, color) in colors.iter().enumerate() {
                let delta = std::array::from_fn(|k| (rgb[k] - color[k]).powi(2));
                let error = dot(delta, [0.30, 0.55, 0.15]);
                if error < min_error {
                    nearest = i;
                    min_error = error;
                }
            }
            rgba.extend_from_slice(&palette[nearest]);
            rgba.push(255);
        }
    }
    rgba
}
