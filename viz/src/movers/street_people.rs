use super::*;



type OriginKey = (u64, u64);
type WalkRoute = (Arc<Path>, String);

pub(super) struct WalkRoutes(HashMap<String, HashMap<OriginKey, WalkRoute>>);

impl WalkRoutes {
    pub(super) fn load() -> Self {
        let pack = gzip_json("web/data/station_access_routes.json.gz");
        let paths: Vec<_> = pack["paths"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                let mut path = points(row);
                path.dedup();
                Path::single(path, Vehicle::Walk)
            })
            .collect();
        let mut stations = HashMap::<String, HashMap<OriginKey, WalkRoute>>::new();
        for row in pack["routes"].as_array().unwrap() {
            if row["status"] != "Routed" {
                continue;
            }
            let origin = &row["origin"];
            let key = (
                origin[0].as_f64().unwrap().to_bits(),
                origin[1].as_f64().unwrap().to_bits(),
            );
            stations
                .entry(row["station"].as_str().unwrap().to_owned())
                .or_default()
                .insert(
                    key,
                    (
                        paths[row["route"].as_u64().unwrap() as usize].clone(),
                        row["entrance"].as_str().unwrap().to_owned(),
                    ),
                );
        }
        Self(stations)
    }
}

#[path = "access_flows.rs"]
mod access_flows;
pub(super) use access_flows::{AccessFlows, AccessFootprint, entry_disc};
