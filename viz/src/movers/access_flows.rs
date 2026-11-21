use super::*;
struct Approach {
    path: Arc<Path>,
    scenario_share: [f64; 2],
    base_share: [f64; 2],
    seed: f64,
    centre: Vec3,
    radius: f32,
}

pub(in crate::movers) struct AccessFlows {
    approaches: Vec<Approach>,
    scenario: Vec<f64>,
    base: Vec<f64>,
    has_baseline: bool,
}

fn walk_shares(origins: &Value) -> HashMap<OriginKey, [f64; 2]> {
    let mut shares = HashMap::<_, [f64; 2]>::new();
    for (half, period) in ["home", "job"].into_iter().enumerate() {
        for row in origins[period].as_array().into_iter().flatten() {
            if row[2] == "walk" {
                let key = (
                    row[0].as_f64().unwrap().to_bits(),
                    row[1].as_f64().unwrap().to_bits(),
                );
                shares.entry(key).or_default()[half] += row[3].as_f64().unwrap();
            }
        }
    }
    shares
}

impl AccessFlows {
    // Receive the baseline origin shares from the same engine baseline as base_half_hour.
    // An absent comparison still draws today's access, without claiming a route-level gain.
    pub(in crate::movers) fn new(row: &Value, base_origins: Option<&Value>, routes: &Routes) -> Self {
        let id = row["id"].as_str().unwrap();
        let bins = |field: &str| {
            row[field]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect()
        };
        let mut flow = Self {
            approaches: Vec::new(),
            scenario: bins("scenario_half_hour"),
            base: bins("base_half_hour"),
            has_baseline: base_origins.is_some(),
        };
        let Some(paths) = routes.station_walk_routes.0.get(id) else {
            return flow;
        };
        let gates = &routes.entrances[id];
        let base = base_origins.map(walk_shares).unwrap_or_default();
        for (key, scenario_share) in walk_shares(&row["access_origins"]) {
            let Some((path, door)) = paths.get(&key) else {
                continue;
            };
            if !gates
                .iter()
                .any(|gate| gate.gtfs.as_deref() == Some(door.as_str()))
            {
                continue;
            }
            let centre = (path.points[0] + *path.points.last().unwrap()) * 0.5;
            let radius = path
                .points
                .iter()
                .map(|p| p.distance(centre))
                .fold(0., f32::max);
            // Stable phase for the same station/block across runs. Distinct destinations
            // from one block get independent illustrative arrivals.
            let mut hash = key.0 ^ key.1.rotate_left(29);
            for byte in id.bytes() {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
            hash ^= hash >> 33;
            hash = hash.wrapping_mul(0xff51afd7ed558ccd);
            hash ^= hash >> 33;
            flow.approaches.push(Approach {
                path: path.clone(),
                scenario_share,
                base_share: base.get(&key).copied().unwrap_or_default(),
                seed: (hash >> 11) as f64 / (1u64 << 53) as f64,
                centre,
                radius,
            });
        }
        flow.approaches.sort_by(|a, b| a.seed.total_cmp(&b.seed));
        flow
    }

    pub(in crate::movers) fn draw(
        &self,
        now: f32,
        eye: Vec3,
        view_distance: f32,
        visible: &impl Fn(Vec3, f32) -> bool,
        place: &mut impl FnMut(Model, Paint, Transform),
    ) {
        // At district scale each figure is a sample of entries in its half-hour.
        // The same scale, hash and path apply to both runs. At street scale each
        // whole entry has a figure, with deterministic rounding of fractional entries.
        let weight = if view_distance < 650. {
            1.
        } else if view_distance < 3500. {
            8.
        } else {
            48.
        };
        let size = (view_distance * 0.005).clamp(1.8, 11.);
        let slot = (now / BIN).floor() as i32;
        for approach in &self.approaches {
            if !visible(approach.centre, approach.radius + 20.) {
                continue;
            }
            let duration = *approach.path.seconds.last().unwrap();
            for arrival_slot in slot..=((now + duration) / BIN).floor() as i32 {
                let bin = arrival_slot.rem_euclid(48) as usize;
                let half = bin / 24;
                let count = self.scenario[bin] * approach.scenario_share[half];
                let reference = self.base[bin] * approach.base_share[half];
                let phase = (approach.seed + bin as f64 * 0.61803398875).fract();
                let rounding = (approach.seed * 251. + bin as f64 * 0.41421356237).fract();
                let copies = (count / weight + rounding).floor() as usize;
                let plain = (reference / weight + rounding).floor() as usize;
                for copy in 0..copies {
                    // Distribute arrivals over the bin. A larger flow appends figures and
                    // leaves the timing and path of each common display sample intact.
                    let fraction = (phase + copy as f64 * 0.7548776662466927).fract();
                    let arrival = (arrival_slot as f32 + fraction as f32) * BIN;
                    let age = now - (arrival - duration);
                    if let Some((p, dir, _, metres)) = approach.path.trip_pose(age, copy)
                        && visible(p, 20.)
                    {
                        let paint = if self.has_baseline && copy >= plain {
                            Paint::Gain
                        } else {
                            Paint::Plain
                        };
                        let scale = pedestrian_scale(eye.distance(p), size);
                        let frame = motion::gait_frame(metres, scale, copy);
                        place(
                            Model::Walking(frame),
                            paint,
                            pose(p, dir, Vec3::splat(scale)),
                        );
                    }
                }
            }
        }
    }
}

pub(in crate::movers) fn entry_disc() -> Mesh {
    painted(Cylinder::new(1., 0.12).mesh().build(), [0.; 3], [255; 3])
}

impl AccessFlows {
    // One proportional-area symbol beside the physical station in the subway view.
    // Both layers describe entries per half-hour. Neither measures platform occupancy.
    pub(in crate::movers) fn draw_station_entries(
        &self,
        now: f32,
        anchor: Vec3,
        place: &mut impl FnMut(Model, Paint, Transform),
    ) {
        if self.approaches.is_empty() {
            return;
        }
        let bin = (now / BIN) as usize % 48;
        let count = self.scenario[bin];
        let common = count.min(self.base[bin]);
        let mut disc = |entries: f64, paint: Paint, height: f32| {
            let radius = entries.sqrt() as f32 * 0.24;
            place(
                Model::StationEntries,
                paint,
                pose(
                    anchor + Vec3::Y * height,
                    Vec3::X,
                    Vec3::new(radius, 1., radius),
                ),
            );
        };
        if count > common {
            disc(count, Paint::Gain, 0.10)
        }
        if common > 0. {
            disc(common, Paint::Plain, 0.30)
        }
    }
}
