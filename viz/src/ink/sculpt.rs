//! Surface sampling and lighting from Ink Studio's `sculpt.js` and `builder.js`.
use bevy::math::DVec3;
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Splat {
    pub position: [f32; 3],
    /// Symmetric covariance in xx, xy, xz, yy, yz, zz order.
    pub covariance: [f32; 6],
    pub color: [u8; 3],
    pub density: f32,
}

#[derive(Clone, Copy)]
pub enum Shape {
    Sphere {
        radius: f64,
    },
    Box {
        half_size: DVec3,
    },
    /// An infinite plane for distance queries with a finite, irregular footprint for sampling.
    Ground {
        radius: f64,
    },
}

pub struct Solid<'a> {
    pub shape: Shape,
    pub center: DVec3,
    pub subtract: bool,
    pub spacing: f64,
    /// Color stops from shadow to light, including any repeated stops.
    pub ramp: &'a [[u8; 3]],
}

impl Solid<'_> {
    fn distance(&self, position: DVec3) -> f64 {
        let local = position - self.center;
        match self.shape {
            Shape::Sphere { radius } => local.length() - radius,
            Shape::Box { half_size } => {
                let q = local.abs() - half_size;
                q.max(DVec3::ZERO).length() + q.max_element().min(0.)
            }
            Shape::Ground { .. } => local.y,
        }
    }

    fn samples(&self, noise: &Noise, mut visit: impl FnMut(DVec3)) {
        let spacing = self.spacing;
        match self.shape {
            Shape::Sphere { radius } => {
                let count = (1.3 * 4. * PI * radius.powi(2) / spacing.powi(2)).ceil() as usize;
                for i in 0..count {
                    let y = 1. - 2. * (i as f64 + 0.5) / count as f64;
                    let r = (1. - y * y).max(0.).sqrt();
                    let angle = i as f64 * 2.399963;
                    visit(self.center + radius * DVec3::new(r * angle.cos(), y, r * angle.sin()));
                }
            }
            Shape::Box { half_size } => {
                for face in 0..3 {
                    let u = (face + 1) % 3;
                    let v = (face + 2) % 3;
                    let stops =
                        |axis: usize| (2. * half_size[axis] / spacing).round().max(1.) as usize;
                    let (nu, nv) = (stops(u), stops(v));
                    for i in 0..nu {
                        for j in 0..nv {
                            let mut p = DVec3::ZERO;
                            p[u] = ((i as f64 + 0.5) / nu as f64 * 2. - 1.) * half_size[u];
                            p[v] = ((j as f64 + 0.5) / nv as f64 * 2. - 1.) * half_size[v];
                            for sign in [-1., 1.] {
                                p[face] = sign * half_size[face];
                                visit(self.center + p);
                            }
                        }
                    }
                }
            }
            Shape::Ground { radius } => {
                let mut z = -radius;
                let mut odd = false;
                while z <= radius {
                    let mut x = -radius + if odd { spacing * 0.5 } else { 0. };
                    while x <= radius {
                        let theta = z.atan2(x);
                        let edge = DVec3::new(
                            theta.cos() * 1.6 + 5. + self.center.x,
                            theta.sin() * 1.6 + 5.,
                            3.3 + self.center.z,
                        );
                        if x.hypot(z) <= radius * (1. - 0.32 + 0.32 * noise.fbm(edge)) {
                            visit(self.center + DVec3::new(x, 0., z));
                        }
                        x += spacing;
                    }
                    z += spacing * 0.87;
                    odd = !odd;
                }
            }
        }
    }
}

struct Noise {
    permutation: [usize; 512],
    values: [f32; 256],
}

impl Noise {
    fn new(seed: u32) -> Self {
        let mut state = seed ^ 0x9e3779b9;
        let mut random = || {
            state = state.wrapping_add(0x6d2b79f5);
            let t = (state ^ (state >> 15)).wrapping_mul(1 | state);
            let t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
            (t ^ (t >> 14)) as f64 / 4294967296.
        };
        let values = std::array::from_fn(|_| random() as f32);
        let mut permutation = std::array::from_fn(|i| i & 255);
        for i in (1..256).rev() {
            permutation.swap(i, (random() * (i + 1) as f64) as usize);
        }
        permutation.copy_within(0..256, 256);
        Self {
            permutation,
            values,
        }
    }

    fn sample(&self, point: DVec3) -> f64 {
        let cell = point.floor();
        let [x, y, z] = cell.to_array().map(|v| (v as i64 & 255) as usize);
        let [u, v, w] = (point - cell).to_array().map(|v| v * v * (3. - 2. * v));
        let p = &self.permutation;
        let corner = |i, j, k| self.values[p[p[p[i] + j] + k]] as f64;
        let mix = |a, b, t| a + t * (b - a);
        let layer = |k| {
            mix(
                mix(corner(x, y, k), corner(x + 1, y, k), u),
                mix(corner(x, y + 1, k), corner(x + 1, y + 1, k), u),
                v,
            )
        };
        mix(layer(z), layer(z + 1), w)
    }

    fn fbm(&self, p: DVec3) -> f64 {
        let q = p * 2.03;
        (0.5 * self.sample(p) + 0.25 * self.sample(q) + 0.125 * self.sample(q * 2.03)) / 0.875
    }
}

struct Field<'a> {
    positive: Vec<&'a Solid<'a>>,
    negative: Vec<&'a Solid<'a>>,
}

impl<'a> Field<'a> {
    fn sample(&self, point: DVec3) -> (f64, Option<&'a Solid<'a>>) {
        let mut distance = 1.; // Ink Studio caps the distance in empty space at one scene unit.
        let mut owner = None;
        for &solid in &self.positive {
            let d = solid.distance(point);
            if d < distance {
                distance = d;
                owner = Some(solid);
            }
        }
        for &solid in &self.negative {
            let bound = match solid.shape {
                Shape::Sphere { radius } => radius,
                Shape::Box { half_size } => half_size.length(),
                Shape::Ground { .. } => f64::INFINITY,
            };
            if point.distance_squared(solid.center) > (bound + 0.5).powi(2) {
                continue;
            }
            let d = -solid.distance(point);
            if d > distance {
                distance = d;
                owner = Some(solid);
            }
        }
        (distance, owner)
    }

    fn normal(&self, p: DVec3, distance: f64) -> (DVec3, f64) {
        let gradient = DVec3::from_array(
            [DVec3::X, DVec3::Y, DVec3::Z].map(|axis| self.sample(p + 0.012 * axis).0 - distance),
        );
        let length = gradient.length();
        let divisor = if length == 0. { 1. } else { length };
        (gradient / divisor, (divisor / 0.012).max(0.6))
    }

    fn shade(&self, p: DVec3, normal: DVec3, noise: &Noise, sun: DVec3) -> f64 {
        let mut occlusion = 0.;
        let mut weight = 1.;
        for step in 1..=3 {
            let h = 0.14 * step as f64;
            occlusion += (h - self.sample(p + normal * h).0).max(0.) * weight / h;
            weight *= 0.6;
        }
        let ambient = (1. - 1.5 * occlusion / 1.96).clamp(0., 1.);
        let mut direct = normal.dot(sun).max(0.);
        if direct > 0. {
            let mut t = 0.07;
            let mut visibility = 1_f64;
            for _ in 0..20 {
                if t >= 16. {
                    break;
                }
                let distance = self.sample(p + normal * 0.05 + sun * t).0;
                if distance < 0.004 {
                    visibility = 0.;
                    break;
                }
                visibility = visibility.min(9. * distance / t);
                t += distance.clamp(0.05, 0.9);
            }
            direct *= visibility;
        }
        ambient * (0.14 + 0.26 * (0.5 + 0.5 * normal.y))
            + 0.72 * direct
            + 0.05 * (noise.sample(p * 3.1 + DVec3::X * 7.) - 0.5)
    }
}

/// Bake positive solids and their cut surfaces in Ink Studio's draw order.
pub fn bake(solids: &[Solid<'_>], seed: u32) -> Vec<Splat> {
    let (positive, negative) = solids.iter().partition(|s| !s.subtract);
    let field = Field { positive, negative };
    let noise = Noise::new(seed);
    let sun = DVec3::new(-0.5, 0.8, 0.45).normalize();
    let mut splats = Vec::new();
    for solid in field.positive.iter().chain(&field.negative) {
        let spacing = solid.spacing;
        solid.samples(&noise, |mut p| {
            let (mut distance, mut owner) = field.sample(p);
            if distance.abs() > spacing * 0.7 {
                return;
            }
            for _ in 0..7 {
                if distance.abs() <= 0.004 {
                    break;
                }
                let (normal, slope) = field.normal(p, distance);
                p -= normal * (distance / slope);
                (distance, owner) = field.sample(p);
            }
            if distance.abs() > spacing * 0.5 {
                return;
            }
            let Some(owner) = owner else { return };
            let (normal, _) = field.normal(p, distance);
            let light = field.shade(p, normal, &noise, sun);
            let color_index = (light * owner.ramp.len() as f64).floor() as usize;
            let plane = (spacing * 0.46).powi(2);
            let delta = (spacing * 0.3).powi(2) - plane;
            let [x, y, z] = normal.to_array();
            let covariance = [
                plane + delta * x * x,
                delta * x * y,
                delta * x * z,
                plane + delta * y * y,
                delta * y * z,
                plane + delta * z * z,
            ]
            .map(|v| v as f32);
            splats.push(Splat {
                position: p.as_vec3().to_array(),
                covariance,
                color: owner.ramp[color_index.min(owner.ramp.len() - 1)],
                density: 3.,
            });
        });
    }
    splats
}
