use super::*;

// OSM's layer tags give relative order. Use a display depth of 12 m in tunnels
// and a display height of 7 m on viaducts. Grade transitions within mixed legs are schematic.
pub(super) struct Levels(Vec<(f32, f32)>);

impl Levels {
    pub(super) fn load(row: &Value, data: &Value, path: &Path) -> Self {
        let mut endpoints = Vec::new();
        for feature in data["features"].as_array().unwrap() {
            let p = &feature["properties"];
            if p["feature"] != "track"
                || p["line"] != row["route"]
                || p["direction"] != row["direction"]
            {
                continue;
            }
            let coords = feature["geometry"]["coordinates"].as_array().unwrap();
            for (coord, kind) in [
                (&coords[0], &p["origin_kind"]),
                (coords.last().unwrap(), &p["destination_kind"]),
            ] {
                let h = match kind.as_str().unwrap() {
                    "tunnel" => -12.,
                    "viaduct" => 7.,
                    "surface" => 0.,
                    _ => continue,
                };
                endpoints.push((path.nearest(point(coord)).2, h));
            }
        }
        endpoints.sort_by(|a, b| a.0.total_cmp(&b.0));
        endpoints.dedup_by(|a, b| a.0 == b.0);
        Self(endpoints)
    }

    pub(super) fn at(&self, distance: f32) -> f32 {
        let i = self.0.partition_point(|&(d, _)| d < distance);
        if i == 0 {
            return self.0[0].1;
        }
        if i == self.0.len() {
            return self.0[i - 1].1;
        }
        let (a, from) = self.0[i - 1];
        let (b, to) = self.0[i];
        // Keep platforms level. The pack locates the endpoints, but not the portal.
        let t = ((distance - a) / (b - a) * 2. - 0.5).clamp(0., 1.);
        from + (to - from) * t
    }
}

#[derive(Component)]
pub(super) struct Underground;

pub(super) fn toggle_tracks(
    mode: Res<crate::city::SubwayView>,
    mut tracks: Query<&mut Visibility, With<Underground>>,
) {
    if mode.is_changed() {
        for mut visibility in &mut tracks {
            *visibility = if mode.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

// Use the same wheelbase chord, lane offset and height as the cars. Keep the two
// meshes separate so the surface view can hide underground segments without rebuilding.
pub(super) fn track_meshes(line: &RailLine) -> [Surface; 2] {
    let mut surfaces = [Surface::default(), Surface::default()];
    let bed = Color::srgb_u8(82, 80, 76).to_linear().to_f32_array();
    let steel = Color::srgb_u8(175, 180, 182).to_linear().to_f32_array();
    let end = *line.path.metres.last().unwrap();
    let mut d = 0.;
    while d < end {
        let next = (d + 16.0_f32).min(end);
        let (a, da) = line.coach(d);
        let (b, db) = line.coach(next);
        let sa = right(da).normalize_or_zero();
        let sb = right(db).normalize_or_zero();
        let surface = &mut surfaces[usize::from(line.levels.at((d + next) * 0.5) < -0.6)];
        for (lo, hi, height, color) in [
            (-1.55, 1.55, -0.15, bed),
            (-0.84, -0.66, -0.03, steel),
            (0.66, 0.84, -0.03, steel),
        ] {
            let i = surface.positions.len() as u32;
            for p in [a + sa * lo, a + sa * hi, b + sb * hi, b + sb * lo] {
                surface.positions.push((p + Vec3::Y * height).to_array());
                surface.normals.push([0., 1., 0.]);
                surface.colors.push(color);
            }
            surface.indices.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
        }
        d = next;
    }
    surfaces
}
