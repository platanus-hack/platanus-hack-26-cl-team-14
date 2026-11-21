//! Representative foot access for the ten proposed Mapocho piers.
//! Each path starts on a mapped public way by a populated INE block. The
//! final segment connects the land network to the hypothetical river stop.
use bevy::prelude::Vec3;
use serde::Deserialize;
use std::collections::HashMap;

use crate::city::{terrain_from_xz, world};

pub(super) struct AccessPath {
    points: Vec<Vec3>,
    distances: Vec<f32>,
}

impl AccessPath {
    pub(super) fn length(&self) -> f32 {
        *self.distances.last().unwrap()
    }

    pub(super) fn gangway_start(&self) -> Vec3 {
        self.points[self.points.len() - 2]
    }

    /// Fraction of distance from the block to the pier. Reverse it for egress.
    pub(super) fn at(&self, fraction: f32) -> Vec3 {
        let distance = fraction.clamp(0., 1.) * self.length();
        let i = self
            .distances
            .partition_point(|&d| d < distance)
            .clamp(1, self.points.len() - 1);
        let along =
            (distance - self.distances[i - 1]) / (self.distances[i] - self.distances[i - 1]);
        let mut position = self.points[i - 1].lerp(self.points[i], along);
        if i < self.points.len() - 1 {
            position.y = terrain_from_xz(position.x, position.z);
        }
        position
    }
}

pub(super) fn load() -> HashMap<String, AccessPath> {
    #[derive(Deserialize)]
    struct Pack {
        routes: HashMap<String, Route>,
    }
    #[derive(Deserialize)]
    struct Route {
        coordinates: Vec<[f64; 2]>,
    }

    let pack: Pack =
        serde_json::from_str(include_str!("../../../web/data/mapocho_pier_walks.json")).unwrap();
    pack.routes
        .into_iter()
        .map(|(id, route)| {
            let mut points: Vec<_> = route
                .coordinates
                .into_iter()
                .map(|[lon, lat]| world(lon, lat))
                .collect();
            // The cached stop marks the vessel centreline. Leave a boarding gap
            // between its hull and the end of the proposed gangway.
            let last = points.len() - 1;
            let stop = points[last];
            let shore = points[last - 1];
            let span = (shore - stop).with_y(0.).length();
            points[last] = stop.lerp(shore, (5. / span).min(0.5));
            let mut distances = vec![0.];
            for pair in points.windows(2) {
                distances.push(distances.last().unwrap() + pair[0].distance(pair[1]));
            }
            (id, AccessPath { points, distances })
        })
        .collect()
}
