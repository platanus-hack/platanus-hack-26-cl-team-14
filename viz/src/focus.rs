//! Frame the place that the engine selected. A new scene takes precedence over an edit;
//! a scene already framed yields to a later physical edit.
use crate::{FOV, Rig, city, net::Scenario};
use bevy::{prelude::*, time::Real, window::PrimaryWindow};
use serde_json::Value;

#[derive(Clone, Copy)]
pub(super) struct Flight {
    from: Vec3,
    to: Vec3,
    from_distance: f32,
    to_distance: f32,
    elapsed: f32,
}

impl Rig {
    pub(super) fn fly_to(&mut self, target: Vec3, distance: f32) {
        self.flight = Some(Flight {
            from: self.target,
            to: target,
            from_distance: self.distance,
            to_distance: distance,
            elapsed: 0.,
        });
    }
}

pub(super) fn advance(time: Res<Time<Real>>, mut rig: ResMut<Rig>) {
    let Some(mut flight) = rig.flight else { return };
    flight.elapsed += time.delta_secs();
    let t = flight.elapsed.min(1.);
    let blend = t * t * (3. - 2. * t);
    rig.target = flight.from.lerp(flight.to, blend);
    rig.target.y = city::terrain_from_xz(rig.target.x, rig.target.z);
    rig.distance = flight.from_distance + (flight.to_distance - flight.from_distance) * blend;
    rig.flight = (t < 1.).then_some(flight);
}

pub(super) fn select(
    scenario: Res<Scenario>,
    mut rig: ResMut<Rig>,
    mut previous_scene: Local<Option<u64>>,
    mut previous_requests: Local<Vec<Value>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if rig.fixed {
        return;
    }
    let result = &scenario.0;
    rig.event_focus = None;
    // Snapshots contain every applied request, including requests from the previous revision.
    let mut retained = std::mem::take(&mut *previous_requests);
    let mut added = Vec::new();
    for change in result["applied"].as_array().into_iter().flatten() {
        let mut request = change.clone();
        request.as_object_mut().unwrap().remove("understood");
        if let Some(index) = retained.iter().position(|old| *old == request) {
            retained.swap_remove(index);
        } else {
            added.push(change);
        }
        previous_requests.push(request);
    }
    let scene = &result["scene"];
    let kept_scene = scene["revision"].as_u64().is_some();
    if let Some(revision) = scene["revision"].as_u64() {
        if *previous_scene != Some(revision) {
            let place = &scene["place"];
            if let Some(target) = shot_position(place) {
                let distance = match place["type"].as_str() {
                    Some("line") => 18_000.,
                    Some("comuna") => 4_000.,
                    _ => 360.,
                };
                rig.fly_to(target, distance);
                *previous_scene = Some(revision);
                info!(
                    "Camera scene {revision} at {:?}, distance {distance} m",
                    target
                );
            }
            return;
        }
    } else {
        *previous_scene = None;
    }
    let aspect = window.width() / window.height().max(1.);
    for change in added.into_iter().rev() {
        // A kept scene holds its framing, except for a physical edit somewhere else.
        if kept_scene && change["understood"]["city_edit_index"].is_null() {
            continue;
        }
        if change["kind"] == "event"
            && let Some(venue) = shot_position(&change["target"])
        {
            // The event renderer has the mapped exit needed to frame the crowd.
            rig.event_focus = Some(venue);
            return;
        }
        if let Some((target, distance)) = change_shot(result, change, aspect) {
            rig.fly_to(target, distance);
            info!(
                "Camera scenario {} at {:?}, distance {distance} m",
                result["revision"], target
            );
            return;
        }
    }
}

fn frame(mut points: impl Iterator<Item = Vec3>, minimum: f32, aspect: f32) -> Option<(Vec3, f32)> {
    let first = points.next()?;
    let (low, high) = points.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p)));
    let center = low.midpoint(high);
    let radius = center.distance(high);
    let half_angle = ((FOV / 2.).tan() * aspect.min(1.)).atan();
    Some((center, minimum.max(1.15 * radius / half_angle.sin())))
}

#[derive(Default, Resource)]
pub(super) struct CueCaption(pub String);

// A cue changes the caption and, when the camera is unlocked, starts a flight.
pub(super) fn cue(
    time: Res<Time<Real>>,
    scenario: Res<Scenario>,
    mut rig: ResMut<Rig>,
    mut caption: ResMut<CueCaption>,
    mut scene_revision: Local<Option<u64>>,
    mut start: Local<f32>,
    mut next: Local<usize>,
) {
    let scene = &scenario.0["scene"];
    if scene["revision"].as_u64() != *scene_revision {
        *scene_revision = scene["revision"].as_u64();
        *start = time.elapsed_secs();
        *next = 0;
        caption.0.clear();
    }
    let Some(cue) = scene["cues"].get(*next) else { return };
    if time.elapsed_secs() - *start < cue["after_seconds"].as_f64().unwrap() as f32 {
        return;
    }
    caption.0 = cue["caption"].as_str().unwrap_or_default().to_owned();
    if !rig.fixed {
        let (target, distance) = (shot_position(cue).unwrap(), cue["distance"].as_f64().unwrap() as f32);
        rig.fly_to(target, distance);
        info!("Camera cue {} at {target:?}, distance {distance} m", *next);
    }
    *next += 1;
}

fn shot_position(row: &Value) -> Option<Vec3> {
    Some(crate::city::world(
        row["longitude"].as_f64()?,
        row["latitude"].as_f64()?,
    ))
}

fn shot_vertices(coords: &Value, points: &mut Vec<Vec3>) {
    if let Some(items) = coords.as_array() {
        if let [x, y, ..] = items.as_slice()
            && let (Some(lon), Some(lat)) = (x.as_f64(), y.as_f64())
        {
            points.push(crate::city::world(lon, lat));
        } else {
            for child in items {
                shot_vertices(child, points);
            }
        }
    }
}

fn shot_block(result: &Value, id: &str, kind: &str) -> Option<Vec3> {
    if let Some(point) = result["blocks_changed"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|row| row["id"].as_str() == Some(id))
        .and_then(shot_position)
    {
        return Some(point);
    }

    // The engine returns at most 300 changed rows. The first four CSV fields are
    // block_id, CUT, longitude and latitude in both original engine input files.
    let file = if kind == "jobs" {
        "data/mvp/jobs.csv"
    } else {
        "data/mvp/blocks.csv"
    };
    let csv = std::fs::read_to_string(file).ok()?;
    let mut fields = csv
        .lines()
        .find(|l| l.split_once(',').is_some_and(|(key, _)| key == id))?
        .split(',');
    let lon = fields.nth(2)?.parse().ok()?;
    Some(crate::city::world(lon, fields.next()?.parse().ok()?))
}

fn planned_extent(selector: &str, id: &str, points: &mut Vec<Vec3>) {
    static CATALOG: std::sync::OnceLock<Option<Value>> = std::sync::OnceLock::new();
    let data = CATALOG.get_or_init(|| {
        serde_json::from_reader(std::fs::File::open("data/mvp/metro_planned.geojson").ok()?).ok()
    });
    if let Some(features) = data.as_ref().and_then(|d| d["features"].as_array()) {
        for feature in features {
            if feature["properties"][selector].as_str() == Some(id) {
                shot_vertices(&feature["geometry"]["coordinates"], points);
            }
        }
    }
}

fn change_shot(result: &Value, change: &Value, aspect: f32) -> Option<(Vec3, f32)> {
    let target = &change["target"];
    let note = &change["understood"];
    let kind = change["kind"].as_str()?;
    let id = target["id"].as_str();
    let mut points = Vec::new();
    let mut minimum = 360.0;

    if let Some(index) = note["city_edit_index"].as_u64() {
        let edit = result["city_edits"].get(index as usize)?;
        for side in ["before", "after"] {
            shot_vertices(&edit[side]["geometry"]["coordinates"], &mut points);
        }
    } else {
        match (kind, target["type"].as_str()?) {
            ("station", "line" | "corridor") => {
                let selector = target["type"].as_str()?;
                planned_extent(selector, id?, &mut points);
                let ids = note["physical_station_ids"].as_array()?;
                for row in result["stations"].as_array()? {
                    if ids.contains(&row["id"]) {
                        points.extend(shot_position(row));
                    }
                }
            }
            ("station", "station") => {
                let rows = result["stations"].as_array()?;
                if let Some(ids) = note["physical_station_ids"].as_array() {
                    for row in rows {
                        if ids.contains(&row["id"]) {
                            points.extend(shot_position(row));
                        }
                    }
                    if points.is_empty() {
                        planned_extent("id", id?, &mut points);
                    }
                } else if let Some(key) = note["id"].as_str() {
                    // An added point disappears from stations[] when it is closed.
                    let row = rows.iter().find(|r| r["id"].as_str() == Some(key));
                    points.extend(row.and_then(shot_position).or_else(|| {
                        let point = result["applied"].as_array()?.iter().rev().find(|c| {
                            c["kind"] == "station"
                                && c["target"]["type"] == "point"
                                && c["understood"]["id"].as_str() == Some(key)
                        })?;
                        shot_position(&point["target"])
                    }));
                } else {
                    let row = rows
                        .iter()
                        .find(|r| r["id"].as_str() == id)
                        .or_else(|| rows.iter().find(|r| r["name"] == note["station"]))?;
                    points.extend(shot_position(row));
                }
            }
            ("station" | "jobs" | "residents", "point") => {
                let snapped = note["block"]
                    .as_str()
                    .and_then(|b| shot_block(result, b, kind));
                points.extend(snapped.or_else(|| shot_position(target)));
            }
            ("jobs" | "residents", "block") => points.extend(shot_block(result, id?, kind)),
            (_, "comuna") => {
                let row = result["zones"]
                    .as_array()?
                    .iter()
                    .find(|r| r["id"].as_str() == id)?;
                points.extend(shot_position(row));
                minimum = 3500.0;
            }
            _ => return None,
        }
    }
    frame(points.into_iter(), minimum, aspect)
}
