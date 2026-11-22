//! Use the same detailed banks, islands and outer river ribbon as the city mesh.
use bevy::prelude::{Vec2, Vec3, Vec3Swizzles};
use geo::{LineString, MultiPolygon, Orient, Polygon, Relate, orient::Direction, unary_union};
use std::sync::OnceLock;

static WATER: OnceLock<Map> = OnceLock::new();

#[derive(Default)]
pub(super) struct Map {
    surfaces: Vec<Polygon<f64>>,
    channels: Vec<Vec<Vec2>>,
    union: OnceLock<MultiPolygon<f64>>,
}

fn line(points: &[Vec2]) -> LineString<f64> {
    points.iter().map(|p| (p.x as f64, p.y as f64)).collect()
}

impl Map {
    pub(super) fn add_polygon(&mut self, rings: &[Vec<Vec3>]) {
        let mut rings = rings
            .iter()
            .map(|ring| ring.iter().map(|p| (p.x as f64, p.z as f64)).collect());
        self.surfaces
            .push(Polygon::new(rings.next().unwrap(), rings.collect()).orient(Direction::Default));
    }

    pub(super) fn add_channel(&mut self, points: &[Vec3]) {
        let mut channel: Vec<_> = points.iter().map(|p| p.xz()).collect();
        channel.dedup();
        self.channels.push(channel);
    }

    pub(super) fn install(self) {
        let _ = WATER.set(self);
    }
}

fn projection(points: &[Vec2], p: Vec2) -> (usize, Vec2) {
    points
        .windows(2)
        .enumerate()
        .map(|(i, pair)| {
            let d = pair[1] - pair[0];
            let t = ((p - pair[0]).dot(d) / d.length_squared()).clamp(0., 1.);
            (i, pair[0] + t * d)
        })
        .min_by(|a, b| a.1.distance_squared(p).total_cmp(&b.1.distance_squared(p)))
        .unwrap()
}

// Try the portion of the mapped channel between the endpoints, in travel order.
// Keep both joins so the coverage check includes the approach to each call.
fn detour(channel: &[Vec2], a: Vec2, b: Vec2) -> Vec<Vec2> {
    let (i, first) = projection(channel, a);
    let (j, last) = projection(channel, b);
    let mut path = vec![a, first];
    if i < j {
        path.extend_from_slice(&channel[i + 1..=j]);
    } else if j < i {
        path.extend(channel[j + 1..=i].iter().rev().copied());
    }
    path.extend([last, b]);
    path.dedup();
    path
}

pub(crate) fn route(mut points: Vec<Vec2>) -> Option<Vec<Vec2>> {
    points.dedup();
    if points.len() < 2 {
        return None;
    }
    let map = WATER.get()?;
    let mask = map.union.get_or_init(|| unary_union(&map.surfaces));
    let covers = |points: &[Vec2]| mask.relate(&line(points)).is_covers();
    if covers(&points) {
        return Some(points);
    }
    let mut path = vec![points[0]];
    for pair in points.windows(2) {
        if covers(pair) {
            path.push(pair[1]);
        } else {
            // Prefer the detailed centreline. The less detailed full river can
            // repair an outer reach only if the same rendered water covers it.
            let replacement = map
                .channels
                .iter()
                .rev()
                .map(|channel| detour(channel, pair[0], pair[1]))
                .find(|part| covers(part))?;
            path.extend_from_slice(&replacement[1..]);
        }
    }
    Some(path)
}
