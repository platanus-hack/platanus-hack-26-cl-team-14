use super::*;

const ENTRIES_PER_STREET_FIGURE: f64 = 48.;
const GATE_APPROACH: f32 = 18. / 1.4;

type OriginKey = (u64, u64);

pub(super) struct WalkRoutes(HashMap<String, HashMap<OriginKey, Arc<Path>>>);

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
        let mut stations = HashMap::<String, HashMap<OriginKey, Arc<Path>>>::new();
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
                .insert(key, paths[row["route"].as_u64().unwrap() as usize].clone());
        }
        Self(stations)
    }
}

struct Origin {
    share: f64,
    path: Arc<Path>,
}

pub(super) struct StreetPeople {
    centre: Vec3,
    entries: Vec<f64>,
    origins: [Vec<Origin>; 2],
    longest: f32,
}

impl StreetPeople {
    pub(super) fn new(row: &Value, routes: &Routes) -> Self {
        let id = row["id"].as_str().unwrap();
        let paths = routes.station_walk_routes.0.get(id);
        let origins = ["home", "job"].map(|period| {
            row["access_origins"][period]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|origin| {
                    if origin[2] != "walk" {
                        return None;
                    }
                    let key = (
                        origin[0].as_f64().unwrap().to_bits(),
                        origin[1].as_f64().unwrap().to_bits(),
                    );
                    Some(Origin {
                        share: origin[3].as_f64().unwrap(),
                        path: paths?.get(&key)?.clone(),
                    })
                })
                .collect::<Vec<_>>()
        });
        let longest = origins
            .iter()
            .flatten()
            .map(|origin| *origin.path.seconds.last().unwrap())
            .fold(0., f32::max);
        Self {
            centre: world(row["longitude"].as_f64().unwrap(), row["latitude"].as_f64().unwrap()),
            entries: row["scenario_half_hour"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect(),
            origins,
            longest,
        }
    }

    pub(super) fn draw(
        &self,
        now: f32,
        eye: Vec3,
        visible: &impl Fn(Vec3, f32) -> bool,
        place: &mut impl FnMut(Model, Paint, Transform),
    ) -> bool {
        let first_slot = (now / BIN) as i32;
        let last_slot = ((now + self.longest) / BIN) as i32;
        let size = (eye.distance(self.centre) * 0.003).clamp(1.2, 3.6);
        for slot in first_slot..=last_slot {
            let bin = slot.rem_euclid(48) as usize;
            let count = self.entries[bin];
            let mut assigned = 0.;
            for origin in &self.origins[usize::from(bin >= 24)] {
                let before = ((assigned / ENTRIES_PER_STREET_FIGURE) - 0.5).ceil() as usize;
                assigned += count * origin.share;
                let after = ((assigned / ENTRIES_PER_STREET_FIGURE) - 0.5).ceil() as usize;
                for index in before..after {
                    let phase = ((index as f32 + slot as f32 * 0.37) * 0.7548777 + 0.5).fract();
                    let entry = slot as f32 * BIN + phase * BIN;
                    let duration = *origin.path.seconds.last().unwrap();
                    let age = now - (entry - duration);
                    if age < duration - GATE_APPROACH
                        && let Some((position, direction, _)) = origin.path.at(age, &origin.path.seconds)
                        && visible(position, 100.)
                    {
                        // shortcut: the foot graph has a door snap; add that connector when it is measured.
                        place(Model::Person, Paint::Plain, pose(position, direction, Vec3::splat(size)));
                    }
                }
            }
        }
        // The existing station renderer owns the final 18 m and all bus or unrouted entries.
        false
    }
}
