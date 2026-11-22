//! Representative walks along mapped streets near proposed ferry stops.
//! The last segment is the proposed gangway, not an existing infrastructure asset.
use bevy::prelude::{Vec2, Vec3, Vec3Swizzles};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    sync::OnceLock,
};

use crate::city::terrain_from_xz;

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

#[derive(Deserialize)]
struct Route {
    coordinates: Vec<[f64; 2]>,
}

fn flat([lon, lat]: [f64; 2]) -> Vec2 {
    Vec2::new(
        ((lon + 70.65) * 92915.) as f32,
        (-(lat + 33.45) * 110905.) as f32,
    )
}

fn ground(p: Vec2) -> Vec3 {
    Vec3::new(p.x, terrain_from_xz(p.x, p.y), p.y)
}

pub(super) struct AccessMap {
    surveyed: Vec<Vec<Vec2>>,
    streets: OnceLock<WalkNetwork>,
}

impl AccessMap {
    pub(super) fn load() -> Self {
        #[derive(Deserialize)]
        struct Pack {
            routes: BTreeMap<String, Route>,
        }
        let pack: Pack =
            serde_json::from_str(include_str!("../../../web/data/mapocho_pier_walks.json"))
                .unwrap();
        let surveyed = pack
            .routes
            .into_values()
            .map(|r| r.coordinates.into_iter().map(flat).collect())
            .collect();
        Self {
            surveyed,
            streets: OnceLock::new(),
        }
    }

    pub(super) fn path(&self, call: Vec2, pier: Vec3, heading: Vec3) -> AccessPath {
        let p = pier.xz();
        // A precisely routed approach belongs to its location, independent of the stop ID.
        let mut walk = if let Some(path) = self.surveyed.iter().find(|r| {
            let stop = r.last().unwrap();
            stop.distance(call) <= 2. && stop.distance(p) <= 2.
        }) {
            path[..path.len() - 1].to_vec()
        } else {
            self.streets
                .get_or_init(WalkNetwork::load)
                .approach(call, p)
        };
        if walk.is_empty() {
            // Outside the mapped walk catchment only the boarding plank is shown.
            walk.push(p + Vec2::new(heading.z, -heading.x) * 14.);
        }
        let mut points: Vec<_> = walk.into_iter().map(ground).collect();
        let shore = *points.last().unwrap();
        let span = shore.xz().distance(p);
        points.push(pier.lerp(shore, (5. / span).min(0.5)));
        let mut distances = vec![0.];
        for pair in points.windows(2) {
            distances.push(distances.last().unwrap() + pair[0].distance(pair[1]));
        }
        AccessPath { points, distances }
    }
}

struct WalkNetwork {
    ways: Vec<Vec<Vec2>>,
    blocks: Vec<Vec2>,
}

impl WalkNetwork {
    fn load() -> Self {
        #[derive(Deserialize)]
        struct Feature {
            geometry: Route,
        }
        #[derive(Deserialize)]
        struct Streets {
            features: Vec<Feature>,
        }
        let reader = BufReader::new(GzDecoder::new(
            File::open("web/data/residential_streets.geojson.gz").unwrap(),
        ));
        let streets: Streets = serde_json::from_reader(reader).unwrap();
        let ways = streets
            .features
            .into_iter()
            .map(|f| f.geometry.coordinates.into_iter().map(flat).collect())
            .collect();
        let mut blocks = Vec::new();
        for row in BufReader::new(File::open("data/mvp/blocks.csv").unwrap())
            .lines()
            .skip(1)
        {
            let row = row.unwrap();
            let mut cols = row.split(',').skip(2);
            let lon = cols.next().unwrap().parse().unwrap();
            let lat = cols.next().unwrap().parse().unwrap();
            if cols.next().unwrap().parse::<u32>().unwrap() > 0 {
                blocks.push(flat([lon, lat]));
            }
        }
        Self { ways, blocks }
    }

    fn approach(&self, call: Vec2, pier: Vec2) -> Vec<Vec2> {
        let mut best = 1000_f32.powi(2); // Same walking catchment as line_riders.rs.
        let mut pick = None;
        for way in &self.ways {
            for (i, pair) in way.windows(2).enumerate() {
                let delta = pair[1] - pair[0];
                if delta.length_squared() == 0. {
                    continue;
                }
                let along = ((call - pair[0]).dot(delta) / delta.length_squared()).clamp(0., 1.);
                let shore = pair[0] + delta * along;
                let gap = shore.distance_squared(call);
                if gap < best {
                    best = gap;
                    pick = Some((way, i, shore));
                }
            }
        }
        let Some((way, i, shore)) = pick else {
            return Vec::new();
        };
        // Select an approach on the same connected OSM way as the shore point.
        // The engine publishes pier totals, so the nearby populated block is illustrative.
        let nearby: Vec<_> = self
            .blocks
            .iter()
            .filter(|p| p.distance_squared(shore) <= 500_f32.powi(2))
            .collect();
        let mut chosen = vec![shore];
        let mut score = f32::INFINITY;
        let mut consider = |next: &[Vec2], reverse: bool| {
            let mut path = vec![shore];
            let mut length = 0.;
            for index in 0..next.len() {
                let end = next[if reverse {
                    next.len() - 1 - index
                } else {
                    index
                }];
                let start = *path.last().unwrap();
                let leg = end.distance(start);
                if leg < 0.1 {
                    continue;
                }
                let travel = leg.min(200. - length);
                let steps = (travel / 30.).ceil() as u32;
                for step in 1..=steps {
                    let along = travel * step as f32 / steps as f32;
                    let point = start.lerp(end, along / leg);
                    path.push(point);
                    let target = (length + along - 140.).abs();
                    let snap = nearby
                        .iter()
                        .map(|b| b.distance(point))
                        .reduce(f32::min)
                        .unwrap_or(0.);
                    if target * 0.6 + snap < score {
                        score = target * 0.6 + snap;
                        chosen = path.clone();
                    }
                }
                length += travel;
                if length >= 199.99 {
                    break;
                }
            }
        };
        consider(&way[..=i], true);
        consider(&way[i + 1..], false);
        chosen.reverse();
        // If the nominal stop lies right on the street, use the end of the street
        // approach as the boarding ramp. Avoid a zero-length pier or walkway.
        while chosen.last().is_some_and(|n| n.distance(pier) < 8.) {
            chosen.pop();
        }
        chosen
    }
}
