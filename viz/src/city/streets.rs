use super::{Surface, terrain::drape_land, world};
use bevy::prelude::Mesh;
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};

#[derive(Deserialize)]
struct Roads {
    features: Vec<Road>,
}

#[derive(Deserialize)]
struct Road {
    geometry: Polygon,
    properties: RoadProperties,
}

#[derive(Deserialize)]
struct Polygon {
    coordinates: Vec<Vec<[f64; 2]>>,
}

#[derive(Deserialize)]
struct RoadProperties {
    tile: usize,
    structure: String,
    layer: i32,
    #[serde(rename = "use")]
    road_use: String,
}

pub(super) fn load(path: &Path) -> impl Iterator<Item = Mesh> {
    let gz = GzDecoder::new(File::open(path).unwrap());
    let roads: Roads = serde_json::from_reader(BufReader::new(gz)).unwrap();
    let mut tiles = BTreeMap::<usize, Surface>::new();
    for road in roads.features {
        let p = road.properties;
        if p.structure == "tunnel" {
            continue;
        }
        let (color, extra_lift) = match p.road_use.as_str() {
            "sidewalk" => ([0.66, 0.67, 0.68, 1.], 0.03),
            "foot" => ([0.47, 0.48, 0.49, 1.], 0.02),
            "cycle" => ([0.30, 0.31, 0.32, 1.], 0.01),
            _ => ([0.18, 0.19, 0.20, 1.], 0.),
        };
        let rings = road
            .geometry
            .coordinates
            .into_iter()
            .map(|mut ring| {
                ring.pop(); // GeoJSON repeats the first vertex at the end.
                ring.into_iter().map(|[lon, lat]| world(lon, lat)).collect()
            })
            .collect::<Vec<_>>();
        let height = super::road_surface_lift(p.layer) + extra_lift;
        drape_land(tiles.entry(p.tile).or_default(), &rings, height, color);
    }
    tiles.into_values().map(Surface::finish)
}
