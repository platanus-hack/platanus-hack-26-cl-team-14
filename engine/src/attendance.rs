//! Temporary event attendance. The changed city's inbound workers provide return comunas and mode shares.
use crate::population::Population;
use crate::stations::access::StationAccess;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

#[derive(Deserialize, Serialize)]
pub struct AttendanceRequest {
    pub longitude: f64,
    pub latitude: f64,
    pub venue_cut: String,
    pub attendance: u32,
    pub assumed_end_half_hour: u8,
}

#[derive(Serialize)]
pub struct AttendanceProjection {
    #[serde(flatten)]
    request: AttendanceRequest,
    source_workers: usize,
    unmodeled_attendance: u32,
    attendees_by_census_mode: BTreeMap<i8, f64>,
    attendees_by_return_comuna: BTreeMap<String, f64>,
    metro_given_public: f64,
    potential_metro_users: Option<f64>,
    metro_users_with_network_access: Option<f64>,
    window: &'static str,
    assumptions: [&'static str; 3],
}

/// Call once per event after rechoose and fine-mode assignment, with the scenario's active Metro parent set.
pub fn project(pop: &Population, access: &StationAccess, active: &HashSet<String>, request: AttendanceRequest) -> AttendanceProjection {
    let jobs_in_venue_cut: Vec<_> = pop.jobs.iter().map(|point| point.cut == request.venue_cut).collect();
    let mut by_mode = BTreeMap::<i8, usize>::new();
    let mut by_home = BTreeMap::<&str, usize>::new();
    for p in pop.persons.iter().filter(|p| !p.student && jobs_in_venue_cut[p.dest as usize]) {
        *by_mode.entry(p.mode).or_default() += 1;
        *by_home.entry(&pop.homes[p.home as usize].cut).or_default() += 1;
    }
    let source_workers: usize = by_mode.values().sum();
    let expected = move |count: usize| f64::from(request.attendance) * count as f64 / source_workers as f64;
    #[derive(Deserialize)]
    struct SaturdayPublic { metro_any_share: f64 }
    let table = include_bytes!("../../data/mvp/saturday_public_metro.json");
    let metro_given_public = serde_json::from_slice::<SaturdayPublic>(table).unwrap().metro_any_share;
    // With zero attendance the demand is known even if this comuna has no proxy workers.
    let potential_metro_users = if request.attendance == 0 { Some(0.0) } else {
        (source_workers > 0).then(|| expected(*by_mode.get(&2).unwrap_or(&0)) * metro_given_public)
    };
    let metro_users_with_network_access = potential_metro_users.map(|mass| {
        mass * access.event_access_share(request.longitude, request.latitude, active)
    });
    AttendanceProjection {
        source_workers,
        unmodeled_attendance: if source_workers == 0 { request.attendance } else { 0 },
        request,
        attendees_by_census_mode: by_mode.into_iter().map(|(m, n)| (m, expected(n))).collect(),
        attendees_by_return_comuna: by_home.into_iter().map(|(cut, n)| (cut.to_owned(), expected(n))).collect(),
        metro_given_public,
        potential_metro_users,
        metro_users_with_network_access,
        window: "whole return period after the assumed event end",
        assumptions: [
            "Attendees take the home comunas and travel modes of workers employed in the venue comuna in the changed city.",
            "Santiago EOD 2012 normal Saturday trips with only public stages supply the conditional share with any Metro stage.",
            "The event transfers weekday 08:00 walking and one-bus access to the return period. End time and attendance are scenario inputs.",
        ],
    }
}
