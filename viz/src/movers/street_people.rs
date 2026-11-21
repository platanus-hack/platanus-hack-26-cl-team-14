use super::*;

const ENTRIES_PER_STREET_FIGURE: f64 = 48.;

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

struct Origin {
    until: f64,
    path: Option<Arc<Path>>,
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
            let mut until = 0.;
            row["access_origins"][period]
                .as_array()
                .into_iter()
                .flatten()
                .map(|origin| {
                    until += origin[3].as_f64().unwrap();
                    let key = (
                        origin[0].as_f64().unwrap().to_bits(),
                        origin[1].as_f64().unwrap().to_bits(),
                    );
                    let path = (origin[2] == "walk")
                        .then(|| paths?.get(&key))
                        .flatten()
                        .and_then(|(path, entrance)| {
                            routes
                                .entrances
                                .get(id)?
                                .iter()
                                .any(|gate| gate.gtfs.as_deref() == Some(entrance.as_str()))
                                .then(|| path.clone())
                        });
                    Origin { until, path }
                })
                .collect::<Vec<_>>()
        });
        let longest = origins
            .iter()
            .flatten()
            .filter_map(|origin| origin.path.as_ref())
            .map(|path| *path.seconds.last().unwrap())
            .fold(0., f32::max);
        let mut entries = vec![0.];
        for count in row["scenario_half_hour"].as_array().unwrap() {
            entries.push(entries.last().unwrap() + count.as_f64().unwrap());
        }
        Self {
            centre: world(
                row["longitude"].as_f64().unwrap(),
                row["latitude"].as_f64().unwrap(),
            ),
            entries,
            origins,
            longest,
        }
    }

    fn route_for_entry(&self, bin: usize, entry_index: usize) -> Option<&Arc<Path>> {
        // Spread each origin's weighted entries across the half hour.
        let share = ((entry_index as f64 + 0.5) * 0.7548776662466927).fract();
        let origins = &self.origins[usize::from(bin >= 24)];
        origins
            .get(origins.partition_point(|origin| origin.until <= share))?
            .path
            .as_ref()
    }

    pub(super) fn draw(
        &self,
        now: f32,
        eye: Vec3,
        visible: &impl Fn(Vec3, f32) -> bool,
        place: &mut impl FnMut(Model, Paint, Transform),
    ) {
        if self.longest == 0. {
            return;
        }
        let first_slot = (now / BIN) as i32;
        let last_slot = ((now + self.longest) / BIN) as i32;
        let distance = eye.distance(self.centre);
        let size = (distance * 0.003).clamp(1.8, 3.6);
        let stride = if distance <= 2_000. {
            1
        } else {
            ENTRIES_PER_STREET_FIGURE as usize
        };
        for slot in first_slot..=last_slot {
            let bin = slot.rem_euclid(48) as usize;
            let count = self.entries[bin + 1] - self.entries[bin];
            if count <= 0. {
                continue;
            }
            let first = (self.entries[bin] - 0.5).ceil() as usize;
            let end = (self.entries[bin + 1] - 0.5).ceil() as usize;
            let first = first + (stride / 2 + stride - first % stride) % stride;
            for index in (first..end).step_by(stride) {
                let Some(path) = self.route_for_entry(bin, index) else {
                    continue;
                };
                let entry = slot as f32 * BIN
                    + ((index as f64 + 0.5 - self.entries[bin]) / count) as f32 * BIN;
                let duration = *path.seconds.last().unwrap();
                let age = now - (entry - duration);
                if let Some((position, direction, _)) = path.at(age, &path.seconds)
                    && visible(position, 100.)
                {
                    // shortcut: the foot graph has a door snap; add that connector when it is measured.
                    let scale = pedestrian_scale(eye.distance(position), size);
                    place(
                        Model::Person,
                        Paint::Plain,
                        pose(position, direction, Vec3::splat(scale)),
                    );
                }
            }
        }
    }
}
