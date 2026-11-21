// Intended as viz/src/terrain.rs. The grid metadata lives beside terrain.i16.
use crate::city::Surface;
use bevy::prelude::*;
use std::{fs, sync::OnceLock};

const W: usize = 481;
const H: usize = 421;
const WEST: f32 = -18_583.0;
const NORTH: f32 = -16_635.75;
const DX: f32 = 92_915.0 / 1200.0;
const DZ: f32 = 110_905.0 / 1200.0;
const DATUM: f32 = 500.0;
const SOIL: [f32; 4] = [0.839, 0.831, 0.913, 1.];
type Rgba = [f32; 4];

fn heights() -> &'static [i16] {
    static METRES: OnceLock<Vec<i16>> = OnceLock::new();
    METRES.get_or_init(|| {
        fs::read("web/data/terrain.i16")
            .unwrap()
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect()
    })
}

pub fn cursor_ground(ray: Ray3d) -> Option<Vec3> {
    let hit = ground_on_line(ray.origin, ray.direction, ray.origin.y);
    let east = WEST + (W - 1) as f32 * DX;
    let south = NORTH + (H - 1) as f32 * DZ;
    ((WEST..=east).contains(&hit.x) && (NORTH..=south).contains(&hit.z)).then_some(hit)
}

/// Find the ground on a downward line. Both the cursor and the camera target use
/// this intersection so scrolling on a slope keeps the target on the terrain.
pub fn ground_on_line(point: Vec3, direction: Dir3, ceiling: f32) -> Vec3 {
    static RANGE: OnceLock<(f32, f32)> = OnceLock::new();
    let &(floor, top) = RANGE.get_or_init(|| {
        let h = heights();
        (
            *h.iter().min().unwrap() as f32 - DATUM,
            *h.iter().max().unwrap() as f32 - DATUM,
        )
    });
    let at_height = |y| point + *direction * ((y - point.y) / direction.y);
    let mut above = at_height(top.min(ceiling));
    // Walk from the camera side to pick the foreground hillside at a ridge.
    while above.y > floor {
        let mut below = at_height((above.y - 16.).max(floor));
        if below.y <= terrain_from_xz(below.x, below.z) || below.y <= floor + 0.01 {
            for _ in 0..12 {
                let middle = above.midpoint(below);
                if middle.y > terrain_from_xz(middle.x, middle.z) {
                    above = middle;
                } else {
                    below = middle;
                }
            }
            return above.midpoint(below);
        }
        above = below;
    }
    above
}

// Interpolate the same two triangles used by the mesh. Roads and movers stay on it.
pub fn terrain_from_xz(x: f32, z: f32) -> f32 {
    let col = ((x - WEST) / DX).clamp(0., (W - 1) as f32);
    let row = ((z - NORTH) / DZ).clamp(0., (H - 1) as f32);
    let i = (col as usize).min(W - 2);
    let j = (row as usize).min(H - 2);
    let (u, v) = (col - i as f32, row - j as f32);
    let p = j * W + i;
    let h = heights();
    let (a, b, c, d) = (
        h[p] as f32,
        h[p + 1] as f32,
        h[p + W] as f32,
        h[p + W + 1] as f32,
    );
    let y = if u + v <= 1.0 {
        a + u * (b - a) + v * (c - a)
    } else {
        d + (1.0 - u) * (c - d) + (1.0 - v) * (b - d)
    };
    y - DATUM
}

// Clip each Earcut triangle to the two planar faces of every DEM cell it crosses.
// The overlay then has the same height as the ground at every interior point.
pub(super) fn drape_land(mesh: &mut Surface, rings: &[Vec<Vec3>], lift: f32, color: Rgba) {
    use bevy::math::DVec2;

    let start = mesh.indices.len();
    mesh.polygon(rings, lift, 0., color, color);
    let earcut = mesh.indices.split_off(start);
    let mut polygon = Vec::with_capacity(6);
    let mut clipped = Vec::with_capacity(6);
    for ids in earcut.chunks_exact(3) {
        let tri = [ids[0], ids[1], ids[2]].map(|index| {
            let [x, _, z] = mesh.positions[index as usize];
            DVec2::new(
                (f64::from(x) - f64::from(WEST)) / f64::from(DX),
                (f64::from(z) - f64::from(NORTH)) / f64::from(DZ),
            )
        });
        let lo = tri[0].min(tri[1]).min(tri[2]).floor().as_ivec2();
        let hi = tri[0].max(tri[1]).max(tri[2]).floor().as_ivec2();
        if lo == hi {
            let diag = f64::from(lo.x + lo.y + 1);
            if tri.iter().all(|p| p.x + p.y <= diag) || tri.iter().all(|p| p.x + p.y >= diag) {
                mesh.indices.extend_from_slice(ids);
                continue;
            }
        }
        for row in lo.y..=hi.y {
            for col in lo.x..=hi.x {
                let x = f64::from(col);
                let y = f64::from(row);
                let diag = x + y + 1.;
                for edges in [
                    [(DVec2::X, x), (DVec2::Y, y), (-DVec2::ONE, -diag)],
                    [
                        (-DVec2::X, -x - 1.),
                        (-DVec2::Y, -y - 1.),
                        (DVec2::ONE, diag),
                    ],
                ] {
                    polygon.clear();
                    polygon.extend(tri);
                    for (normal, boundary) in edges {
                        clipped.clear();
                        for (&a, &b) in polygon.iter().zip(polygon.iter().cycle().skip(1)) {
                            let da = normal.dot(a) - boundary;
                            let db = normal.dot(b) - boundary;
                            if da >= 0. {
                                clipped.push(a);
                            }
                            if da * db < 0. {
                                clipped.push(a.lerp(b, da / (da - db)));
                            }
                        }
                        std::mem::swap(&mut polygon, &mut clipped);
                        if polygon.len() < 3 {
                            break;
                        }
                    }
                    if polygon.len() < 3 {
                        continue;
                    }
                    let mut face = [0u32; 6]; // Intersection of two triangles has at most six corners.
                    for (slot, &p) in face.iter_mut().zip(&polygon) {
                        *slot = if let Some(i) = tri.iter().position(|&original| original == p) {
                            ids[i]
                        } else {
                            let x = (p.x * f64::from(DX) + f64::from(WEST)) as f32;
                            let z = (p.y * f64::from(DZ) + f64::from(NORTH)) as f32;
                            let index = mesh.positions.len() as u32;
                            mesh.vertex(
                                Vec3::new(x, terrain_from_xz(x, z) + lift, z),
                                Vec3::Y,
                                color,
                            );
                            index
                        };
                    }
                    for n in 1..polygon.len() - 1 {
                        mesh.indices.extend([face[0], face[n], face[n + 1]]);
                    }
                }
            }
        }
    }
}

pub struct LandCover(Vec<Rgba>);

impl Default for LandCover {
    fn default() -> Self {
        Self(vec![SOIL; W * H])
    }
}

impl LandCover {
    // Fill the native grid by scanline so large parks follow the actual terrain.
    // Crossings from all rings preserve holes.
    pub fn fill(&mut self, rings: &[Vec<Vec3>], color: Rgba) {
        let north = rings[0].iter().map(|p| p.z).fold(f32::MAX, f32::min);
        let south = rings[0].iter().map(|p| p.z).fold(f32::MIN, f32::max);
        let row0 = ((north - NORTH) / DZ).ceil().max(0.) as usize;
        let row1 = (((south - NORTH) / DZ).floor() as usize).min(H - 1);
        let mut crossings = Vec::new();
        for row in row0..=row1 {
            let z = NORTH + row as f32 * DZ;
            crossings.clear();
            for ring in rings {
                for (a, b) in ring.iter().zip(ring.iter().cycle().skip(1)) {
                    if (a.z > z) != (b.z > z) {
                        crossings.push(a.x + (b.x - a.x) * (z - a.z) / (b.z - a.z));
                    }
                }
            }
            crossings.sort_by(f32::total_cmp);
            for pair in crossings.chunks_exact(2) {
                let first = (((pair[0] - WEST) / DX).ceil() as usize).min(W);
                let end = ((((pair[1] - WEST) / DX).floor() as usize) + 1).min(W);
                self.0[row * W + first..row * W + end].fill(color);
            }
        }
    }

    pub fn meshes(&self) -> Vec<(String, Mesh)> {
        let h = heights();
        let mut chunks = Vec::new();
        for j0 in (0..H - 1).step_by(40) {
            for i0 in (0..W - 1).step_by(40) {
                let i1 = (i0 + 40).min(W - 1);
                let j1 = (j0 + 40).min(H - 1);
                let stride = (i1 - i0 + 1) as u32;
                let mut surface = Surface::default();
                for j in j0..=j1 {
                    for i in i0..=i1 {
                        let p = j * W + i;
                        let west = i.saturating_sub(1);
                        let east = (i + 1).min(W - 1);
                        let north = j.saturating_sub(1);
                        let south = (j + 1).min(H - 1);
                        let slope_x = (h[j * W + east] - h[j * W + west]) as f32
                            / ((east - west) as f32 * DX);
                        let slope_z = (h[south * W + i] - h[north * W + i]) as f32
                            / ((south - north) as f32 * DZ);
                        surface.positions.push([
                            WEST + i as f32 * DX,
                            h[p] as f32 - DATUM,
                            NORTH + j as f32 * DZ,
                        ]);
                        surface
                            .normals
                            .push(Vec3::new(-slope_x, 1., -slope_z).normalize().to_array());
                        surface.colors.push(self.0[p]);
                        if i < i1 && j < j1 {
                            let a = (j - j0) as u32 * stride + (i - i0) as u32;
                            surface.indices.extend([
                                a,
                                a + stride,
                                a + 1,
                                a + 1,
                                a + stride,
                                a + stride + 1,
                            ]);
                        }
                    }
                }
                chunks.push((format!("Terrain {i0} {j0}"), surface.finish()));
            }
        }
        chunks
    }
}
