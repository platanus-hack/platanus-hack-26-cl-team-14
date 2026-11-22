//! Fixed attendance and the audience evidence supplied for an event.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize)]
pub struct AudienceOrigins {
    source: String,
    origin_shares: BTreeMap<String, f64>,
}

impl AudienceOrigins {
    pub fn valid(&self, known_cuts: &[String]) -> bool {
        !self.source.trim().is_empty()
            && !self.origin_shares.is_empty()
            && self.origin_shares.iter().all(|(cut, share)| {
                known_cuts.contains(cut) && (0.0..=1.0).contains(share)
            })
            && self.origin_shares.values().sum::<f64>() <= 1.0 + f64::EPSILON * self.origin_shares.len() as f64
    }
}

#[derive(Deserialize, Serialize)]
pub struct AttendanceRequest {
    pub longitude: f64,
    pub latitude: f64,
    pub venue_cut: String,
    pub attendance: u32,
    pub assumed_end_half_hour: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience_profile: Option<AudienceOrigins>,
}

#[derive(Serialize)]
pub struct AttendanceProjection {
    #[serde(flatten)]
    request: AttendanceRequest,
    audience_status: &'static str,
    attendees_by_origin_comuna: BTreeMap<String, f64>,
    unmodeled_attendance: f64,
    attendees_by_census_mode: BTreeMap<i8, f64>,
    attendees_by_return_comuna: BTreeMap<String, f64>,
    potential_metro_users: Option<f64>,
    metro_users_with_network_access: Option<f64>,
    assumptions: [&'static str; 2],
}

pub fn project(request: AttendanceRequest, has_fan_profile: bool) -> AttendanceProjection {
    let attendance = f64::from(request.attendance);
    let origins = request.audience_profile.as_ref();
    let origin_share = origins.map_or(0.0, |profile| profile.origin_shares.values().sum());
    // Shares totaling slightly above one can result from floating-point addition.
    let assigned_mass = attendance / origin_share.max(1.0);
    let attendees_by_origin_comuna = origins.into_iter()
        .flat_map(|profile| &profile.origin_shares)
        .map(|(cut, share)| (cut.clone(), assigned_mass * share))
        .collect::<BTreeMap<_, _>>();
    let (audience_status, unmodeled_attendance) = if has_fan_profile {
        ("colo_colo", 0.0)
    } else if origins.is_some() {
        ("supplied_origins", attendance * (1.0 - origin_share.min(1.0)))
    } else {
        ("unestimated", attendance)
    };
    let zero_demand = (request.attendance == 0).then_some(0.0);
    AttendanceProjection {
        request,
        audience_status,
        attendees_by_origin_comuna,
        unmodeled_attendance,
        attendees_by_census_mode: BTreeMap::new(),
        attendees_by_return_comuna: BTreeMap::new(),
        potential_metro_users: zero_demand,
        metro_users_with_network_access: zero_demand,
        assumptions: [
            "Attendance and the end half hour are scenario inputs. Unmodeled attendance has no assigned origin.",
            "A supplied origin profile gives origins only. Mode, return destination, and Metro demand are unestimated. The separate Colo-Colo fan_base has its own modeled origins and modes.",
        ],
    }
}
