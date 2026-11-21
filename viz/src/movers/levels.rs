use super::*;

// OSM's layer tags give relative order. Use a display depth of 12 m in tunnels
// and a display height of 7 m on viaducts. Grade transitions within mixed legs are schematic.
pub(super) struct Levels {
    anchors: Vec<(f32, f32)>,
    open_cut: Option<f32>,
}

impl Levels {
    pub(super) fn load(row: &Value, data: &Value, path: &Path) -> Self {
        let mut anchors = Vec::new();
        let mut open_cut = None;
        for feature in data["features"].as_array().unwrap() {
            let p = &feature["properties"];
            if p["line"] != row["route"] {
                continue;
            }
            let coords = &feature["geometry"]["coordinates"];
            if p["feature"] == "platform" {
                if p["station_setting"] == "partly_open_below_street" {
                    open_cut = Some(path.nearest(point(coords)).2);
                }
                continue;
            }
            if p["direction"] != row["direction"] {
                continue;
            }
            let coords = coords.as_array().unwrap();
            let start = path.nearest(point(&coords[0])).2;
            let end = path.nearest(point(coords.last().unwrap())).2;
            let kinds = p["kind_sequence"].as_array().unwrap();
            let lengths = p["osm_kind_metres"].as_object().unwrap();
            let total: f64 = lengths.values().map(|v| v.as_f64().unwrap()).sum();
            let mut left = start;
            for kind in kinds {
                let name = kind.as_str().unwrap();
                // Repeated classes share their published total equally. The portal positions
                // are illustrative. Unknown intervals inherit interpolation from adjacent evidence.
                let repeats = kinds.iter().filter(|other| *other == kind).count();
                let metres = lengths[name].as_f64().unwrap();
                let right = left + (end - start) * (metres / (total * repeats as f64)) as f32;
                let h = match name {
                    "tunnel" => Some(-12.),
                    "surface" => Some(0.),
                    "viaduct" => Some(7.),
                    _ => None,
                };
                if let Some(height) = h {
                    let ramp = ((right - left) * 0.25).min(45.);
                    anchors.push((left + ramp, height));
                    anchors.push((right - ramp, height));
                }
                left = right;
            }
        }
        anchors.sort_by(|a, b| a.0.total_cmp(&b.0));
        anchors.dedup_by(|a, b| a.0 == b.0);
        Self { anchors, open_cut }
    }

    pub(super) fn at(&self, distance: f32) -> f32 {
        let i = self.anchors.partition_point(|&(d, _)| d < distance);
        let h = if i == 0 {
            self.anchors[0].1
        } else if i == self.anchors.len() {
            self.anchors[i - 1].1
        } else {
            let (a, from) = self.anchors[i - 1];
            let (b, to) = self.anchors[i];
            from + (to - from) * (distance - a) / (b - a)
        };
        // Neptuno is in an open cut. Four metres is a representative display depth.
        let blend = self
            .open_cut
            .map_or(0., |s| ((115. - (distance - s).abs()) / 70.).clamp(0., 1.));
        h + (-4. - h) * blend
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
