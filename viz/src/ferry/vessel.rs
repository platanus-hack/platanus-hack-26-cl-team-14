use crate::city::{Surface, terrain_from_xz, world};
use bevy::prelude::*;
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) struct Stop {
    pub id: String,
    pub arrival: f32,
    pub departure: f32,
    distance: f32,
}

pub(super) struct Line {
    pub id: String,
    pub stops: Vec<Stop>,
    pub headway: f32,
    pub duration: f32,
    points: Vec<Vec2>,
    distances: Vec<f32>,
}

fn water_point(p: Vec2) -> Vec3 {
    Vec3::new(p.x, terrain_from_xz(p.x, p.y) + 0.15, p.y)
}

fn flat(coord: &Value) -> Option<Vec2> {
    let p = world(coord[0].as_f64()?, coord[1].as_f64()?);
    Some(Vec2::new(p.x, p.z))
}

impl Line {
    pub fn from_result(result: &Value) -> Vec<Self> {
        // A result contains the edits in order. Its final "after" can also remove a line.
        let mut final_lines = BTreeMap::new();
        for edit in result["city_edits"].as_array().into_iter().flatten() {
            if edit["kind"] == "line" {
                if let Some(id) = edit["id"].as_str() {
                    final_lines.insert(id, &edit["after"]);
                }
            }
        }
        final_lines
            .into_iter()
            .filter(|(_, feature)| feature["properties"]["mode"] == "boat")
            .filter_map(|(id, feature)| Self::parse(id, feature))
            .collect()
    }

    fn parse(id: &str, feature: &Value) -> Option<Self> {
        let props = &feature["properties"];
        let headway = props["headway_minutes"].as_f64()? as f32 * 60.;
        let mut points: Vec<_> = feature["geometry"]["coordinates"]
            .as_array()?
            .iter()
            .map(flat)
            .collect::<Option<_>>()?;
        points.dedup();
        if points.len() < 2 || headway <= 0. {
            return None;
        }
        let mut distances = vec![0.];
        for pair in points.windows(2) {
            distances.push(distances.last()? + pair[0].distance(pair[1]));
        }
        let mut stops = Vec::new();
        let mut departure = 0.;
        let mut first = 0;
        for (i, stop_id) in props["stop_ids"].as_array()?.iter().enumerate() {
            let stop_id = stop_id.as_str()?;
            let pier = flat(&props["new_stops"][stop_id])?;
            // The saved centerline includes a vertex at each proposed pier.
            let closest = (first..points.len()).min_by(|&a, &b| {
                points[a]
                    .distance_squared(pier)
                    .total_cmp(&points[b].distance_squared(pier))
            })?;
            let mut arrival = departure;
            if i > 0 {
                arrival += props["segment_minutes"][i - 1].as_f64()? as f32 * 60.;
                departure = arrival + props["dwell_minutes"][i].as_f64()? as f32 * 60.;
            }
            stops.push(Stop {
                id: stop_id.to_string(),
                arrival,
                departure,
                distance: distances[closest],
            });
            first = closest + 1;
        }
        let duration = stops.last()?.departure;
        if stops.len() < 2 || duration <= 0. {
            return None;
        }
        Some(Self {
            id: id.to_owned(),
            stops,
            headway,
            duration,
            points,
            distances,
        })
    }

    /// Age is in simulated seconds since departure from the first pier.
    pub fn sail(&self, age: f32) -> (Vec3, Vec3) {
        let next = self
            .stops
            .partition_point(|s| s.departure <= age)
            .clamp(1, self.stops.len() - 1);
        let (from, to) = (&self.stops[next - 1], &self.stops[next]);
        let t = ((age - from.departure) / (to.arrival - from.departure)).clamp(0., 1.);
        let distance = if t >= 1. {
            to.distance
        } else {
            from.distance + t * (to.distance - from.distance)
        };
        let end = self
            .distances
            .partition_point(|&d| d <= distance)
            .clamp(1, self.points.len() - 1);
        let a = self.points[end - 1];
        let b = self.points[end];
        let delta = b - a;
        let fraction =
            (distance - self.distances[end - 1]) / (self.distances[end] - self.distances[end - 1]);
        let dir = delta.normalize();
        (
            water_point(a + delta * fraction),
            Vec3::new(dir.x, 0., dir.y),
        )
    }
}

// The mesh faces +Z. Dimensions are illustrative metres. Keep the forward deck open
// so a rider stays visible during the approach, boarding and ride.
pub(super) fn mesh() -> Mesh {
    let mut hull = Surface::default();
    let ink = [0.025, 0.061, 0.095, 1.];
    let paper = [0.87, 0.89, 0.82, 1.];
    let deck = [0.62, 0.74, 0.72, 1.];
    let outline = [
        (-2.55, -9.),
        (-3.1, -7.8),
        (-3.1, 4.8),
        (-2.2, 7.5),
        (0., 10.),
        (2.2, 7.5),
        (3.1, 4.8),
        (3.1, -7.8),
        (2.55, -9.),
    ];
    let ring = |scale: f32| {
        outline
            .iter()
            .map(|&(x, z)| Vec3::new(x * scale, 0., z * scale))
            .collect()
    };
    hull.polygon(&[ring(1.)], 0., 1.2, ink, paper);
    hull.polygon(&[ring(0.91)], 1.2, 0.06, deck, deck);
    let cabin = vec![
        Vec3::new(-2.1, 0., -7.5),
        Vec3::new(-2.1, 0., -2.4),
        Vec3::new(2.1, 0., -2.4),
        Vec3::new(2.1, 0., -7.5),
    ];
    hull.polygon(&[cabin], 1.26, 1.7, ink, [0.38, 0.70, 0.77, 1.]);
    hull.finish()
}
