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
const SOIL: [f32; 4] = [0.54, 0.50, 0.41, 1.0];
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
