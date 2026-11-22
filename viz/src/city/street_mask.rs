use super::{Rgba, Surface, terrain::drape_land, terrain_from_xz};
use bevy::prelude::Vec3;
use geo::{BooleanOps, BoundingRect, Coord, Intersects, LineString, Polygon, Rect, Relate};

#[derive(Default, PartialEq)]
pub(super) struct StreetMask(Vec<(Rect<f32>, Polygon<f32>)>);

fn polygon(rings: &[Vec<Vec3>]) -> Polygon<f32> {
    let mut lines = rings
        .iter()
        .map(|ring| ring.iter().map(|p| (p.x, p.z)).collect::<LineString<_>>());
    Polygon::new(
        lines.next().unwrap_or_else(LineString::empty),
        lines.collect(),
    )
}

impl StreetMask {
    pub(super) fn from_footprints(footprints: impl IntoIterator<Item = Vec<Vec<Vec3>>>) -> Self {
        Self(
            footprints
                .into_iter()
                .filter_map(|rings| {
                    let polygon = polygon(&rings);
                    Some((polygon.bounding_rect()?, polygon))
                })
                .collect(),
        )
    }

    pub(super) fn intersects(&self, bounds: &Rect<f32>) -> bool {
        self.0.iter().any(|(rect, _)| rect.intersects(bounds))
    }

    pub(super) fn contains(&self, point: Vec3) -> bool {
        let p = Coord {
            x: point.x,
            y: point.z,
        };
        self.0
            .iter()
            .any(|(rect, mask)| rect.intersects(&p) && mask.intersects(&p))
    }

    pub(super) fn drape(&self, surface: &mut Surface, rings: &[Vec<Vec3>], lift: f32, color: Rgba) {
        if self.0.is_empty() {
            return drape_land(surface, rings, lift, color);
        }
        let Some(outer) = rings.first().filter(|ring| !ring.is_empty()) else {
            return;
        };
        let (lo, hi) = outer
            .iter()
            .fold((outer[0], outer[0]), |(lo, hi), &p| (lo.min(p), hi.max(p)));
        let bounds = Rect::new((lo.x, lo.z), (hi.x, hi.z));
        if !self.intersects(&bounds) {
            return drape_land(surface, rings, lift, color);
        }

        let road = polygon(rings);
        let mut cuts = self.0.iter().filter_map(|(rect, mask)| {
            if !rect.intersects(&bounds) {
                return None;
            }
            let relation = road.relate(mask);
            (relation.is_intersects() && !relation.is_touches()).then_some(mask)
        });
        let Some(first) = cuts.next() else {
            return drape_land(surface, rings, lift, color);
        };

        let remaining = cuts.fold(road.difference(first), |parts, mask| parts.difference(mask));
        for part in remaining {
            let (exterior, holes) = part.into_inner();
            let rings: Vec<_> = std::iter::once(exterior)
                .chain(holes)
                .map(|mut line| {
                    line.0.pop(); // drape_land takes open rings.
                    line.0
                        .into_iter()
                        .map(|p| Vec3::new(p.x, terrain_from_xz(p.x, p.y), p.y))
                        .collect()
                })
                .collect();
            drape_land(surface, &rings, lift, color);
        }
    }
}
