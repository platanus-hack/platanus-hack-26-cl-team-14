//! Press F to follow the walking person nearest the camera through walk, bus and Metro.
//! At the end of the trip, or on F again, the camera pulls back to the whole city.
//! `SANTIAGO_FOLLOW=1` starts following at the first result, for captures.
use super::{
    BIN, DAY, FixedClock, Life, PlaybackRate, Replay, Routes, departure_offset,
    riders::{RideKey, RiderState},
    trains,
};
use crate::Rig;
use bevy::prelude::*;

const STREET: f32 = 140.;
const CITY: f32 = 18_000.;

pub(crate) struct FollowPlugin;
impl Plugin for FollowPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Follow {
            seen_result: false,
            person: None,
            armed: std::env::var("SANTIAGO_FOLLOW").as_deref() == Ok("1"),
        })
        .add_systems(
            Update,
            follow
                .after(super::read_result)
                .after(crate::focus::advance)
                .before(crate::aim),
        );
    }
}

#[derive(Resource)]
struct Follow {
    seen_result: bool,
    person: Option<(i32, usize, usize)>, // departure bin, group, copy
    armed: bool,
}

fn follow(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    replay: Res<Replay>,
    routes: Res<Routes>,
    fixed: Res<FixedClock>,
    rate: Res<PlaybackRate>,
    mut state: ResMut<Follow>,
    mut rig: ResMut<Rig>,
) {
    let Some(offset) = replay.clock_offset else { return };
    if replay.is_changed() {
        if state.seen_result {
            // Rebuilt departures invalidate the old selection. Let the new scene take the camera.
            state.person = None;
            state.armed = false;
            return;
        }
        state.seen_result = true;
    }
    let now = fixed.0.unwrap_or(offset + time.elapsed_secs() * rate.0).rem_euclid(DAY);
    if keys.just_pressed(KeyCode::KeyF) {
        if state.person.is_some() {
            state.person = None;
            rig.fly_to(crate::city::world(-70.6505, -33.443), CITY);
            return;
        }
        state.armed = true;
    }
    if state.armed {
        // Keep looking until a rider is on foot.
        state.person = nearest_walker(&replay, now, rig.target);
        if let Some(person) = state.person {
            state.armed = false;
            let at = locate(&replay, &routes, now, person, rig.target).unwrap();
            rig.fly_to(at, STREET);
            info!("Following group {} copy {} from {at:?}", person.1, person.2);
        }
        return;
    }
    let Some(person) = state.person else { return };
    // Let the opening flight land before the camera locks on.
    if rig.flight.is_some() {
        return;
    }
    match locate(&replay, &routes, now, person, rig.target) {
        Some(at) => {
            let blend = (time.delta_secs() * 4.).min(1.);
            rig.target = rig.target.lerp(at, blend);
            rig.distance = STREET;
        }
        None => {
            info!("Trip of group {} copy {} ended", person.1, person.2);
            state.person = None;
            rig.fly_to(crate::city::world(-70.6505, -33.443), CITY);
        }
    }
}

fn nearest_walker(replay: &Replay, now: f32, target: Vec3) -> Option<(i32, usize, usize)> {
    let slot = (now / BIN) as i32;
    let mut best = None;
    let mut best_distance = f32::MAX;
    for bin in slot - (replay.longest / BIN).ceil() as i32..=slot {
        let since_bin = now - bin as f32 * BIN;
        for (index, g) in replay.departures[bin.rem_euclid(48) as usize].iter().enumerate() {
            if matches!(g.lifecycle, Life::Leave) {
                continue;
            }
            for (copy, plan) in g.itineraries.iter().enumerate() {
                let age = since_bin - departure_offset(g.phase, copy);
                // The hook needs a routed trip with at least 10 more city minutes.
                if !(1..=10).all(|k| plan.at(age + k as f32 * 60.).is_some()) {
                    continue;
                }
                if let Some(RiderState::Walking(p, _)) = plan.at(age) {
                    let distance = p.distance_squared(target);
                    if distance < best_distance {
                        best_distance = distance;
                        best = Some((bin, index, copy));
                    }
                }
            }
        }
    }
    best
}

// None only when the trip has ended. Between trains the camera holds its place.
fn locate(
    replay: &Replay,
    routes: &Routes,
    now: f32,
    (bin, index, copy): (i32, usize, usize),
    hold: Vec3,
) -> Option<Vec3> {
    let g = replay.departures[bin.rem_euclid(48) as usize].get(index)?;
    let age = now - bin as f32 * BIN - departure_offset(g.phase, copy);
    let plan = g.itineraries.get(copy)?;
    if age > plan.duration() {
        return None;
    }
    match plan.at(age) {
        None => Some(hold),
        Some(RiderState::Walking(p, _) | RiderState::Waiting(p, _)) => Some(p),
        Some(RiderState::Riding(RideKey::Metro { track, run })) => Some(
            trains(routes, now)[track]
                .iter()
                .find(|t| t.run_index == run)
                .map_or(hold, |t| t.pos),
        ),
        Some(RiderState::Riding(RideKey::Surface { template, start })) => Some(
            routes
                .riders
                .surface_pose(template, start + bin.div_euclid(48) * DAY as i32, now)
                .map_or(hold, |(p, _, _)| p),
        ),
    }
}
