// Restore original directed street vertices between the anchors of the published car route.
use super::{Table, gzip_json, points};
use bevy::prelude::Vec3;
use std::{collections::HashMap, sync::Arc};

fn key(p: Vec3) -> (u32, u32) {
    (p.x.to_bits(), p.z.to_bits())
}

pub(super) fn restore(cars: &mut Table) {
    let pack = gzip_json("web/data/car_street_curves.json.gz");
    let mut curves = HashMap::new();
    for row in pack.as_array().unwrap() {
        let line = points(row);
        let mut length = 0.;
        let lengths: Vec<_> = line
            .windows(2)
            .map(|p| {
                length += p[0].distance(p[1]);
                length
            })
            .collect();
        let interior = line[1..line.len() - 1]
            .iter()
            .zip(lengths)
            .map(|(&p, metres)| (p, metres / length))
            .collect::<Vec<_>>();
        curves.insert((key(line[0]), key(*line.last().unwrap())), interior);
    }
    for path in cars.values_mut() {
        let path = Arc::get_mut(path).unwrap();
        let mut coordinates = vec![path.points[0]];
        let mut metres = vec![0.];
        let mut seconds = vec![0.];
        for (i, pair) in path.points.windows(2).enumerate() {
            if let Some(via) = curves.get(&(key(pair[0]), key(pair[1]))) {
                // Each original anchor retains its clock and water classification distance.
                for &(p, fraction) in via {
                    coordinates.push(p);
                    metres.push(path.metres[i] + (path.metres[i + 1] - path.metres[i]) * fraction);
                    seconds
                        .push(path.seconds[i] + (path.seconds[i + 1] - path.seconds[i]) * fraction);
                }
            }
            coordinates.push(pair[1]);
            metres.push(path.metres[i + 1]);
            seconds.push(path.seconds[i + 1]);
        }
        path.legs[0].end = coordinates.len() - 1;
        path.points = coordinates;
        path.metres = metres;
        path.seconds = seconds;
    }
}
