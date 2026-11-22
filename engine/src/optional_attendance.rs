//! Conditional attendance for a named synthetic person and supplied travel options.
use crate::population::Population;
use super::optional_visit::{choose_visit, VisitOption};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Group {
    WorkerOrStudent,
    OtherResident,
}

#[derive(Deserialize, Serialize)]
struct PersonRef {
    group: Group,
    index: usize,
    home_block: String,
}

#[derive(Deserialize, Serialize)]
struct TimeValues {
    in_vehicle_clp_per_hour: u32,
    wait_clp_per_hour: u32,
    walk_clp_per_hour: u32,
    cycle_clp_per_hour: u32,
    source: String,
}

#[derive(Deserialize, Serialize)]
struct RoundTrip {
    #[serde(default)]
    in_vehicle_seconds: u32,
    #[serde(default)]
    wait_seconds: u32,
    #[serde(default)]
    walk_seconds: u32,
    #[serde(default)]
    cycle_seconds: u32,
    fare_clp: u32,
    source: String,
}

#[derive(Deserialize, Serialize)]
struct Activity {
    id: String,
    place: String,
    interest_clp: i32,
    duration_minutes: u32,
    admission_clp: u32,
    open: bool,
    places_available: u32,
    access_basis: String,
    route: Option<RoundTrip>,
}

#[derive(Deserialize)]
pub struct Request {
    person: PersonRef,
    available_minutes: u32,
    budget_clp: u32,
    preference_basis: String,
    time_values: TimeValues,
    options: Vec<Activity>,
}

impl Request {
    pub fn evaluate(&self, pop: &Population) -> Result<Value, &'static str> {
        let (home_index, role, age_band) = match self.person.group {
            Group::WorkerOrStudent => {
                let person = pop.persons.get(self.person.index).ok_or("Unknown worker or student index.")?;
                (person.home, if person.student { "student" } else { "worker" }, person.age_band)
            }
            Group::OtherResident => {
                let person = pop.residents.get(self.person.index).ok_or("Unknown other-resident index.")?;
                let roles = ["under_15", "higher_education", "employed_without_internal_commute",
                             "unemployed", "outside_labour_force", "unknown"];
                (person.home, roles[usize::from(person.role)], person.age_band)
            }
        };
        let home = &pop.homes[home_index as usize];
        if self.person.home_block != home.id {
            return Err("The supplied home block disagrees with this synthetic person index.");
        }
        if self.options.is_empty() || self.preference_basis.trim().is_empty() || self.time_values.source.trim().is_empty() {
            return Err("Supply activity options, a preference basis and a time-value source.");
        }

        let mut ids = HashSet::new();
        let mut choices = Vec::new();
        let mut options = Vec::new();
        for option in &self.options {
            if option.id.is_empty() || option.id == "stay_home" || !ids.insert(&option.id)
                || option.place.trim().is_empty() || option.access_basis.trim().is_empty()
            {
                return Err("Supply unique option IDs, places and access evidence.");
            }
            let Some(route) = &option.route else {
                options.push(json!({"option":option, "feasible":false, "reason":"route_unavailable"}));
                continue;
            };
            if route.source.trim().is_empty() {
                return Err("Supply a source for round-trip time and fare.");
            }
            let seconds = [route.in_vehicle_seconds, route.wait_seconds, route.walk_seconds, route.cycle_seconds]
                .into_iter().map(u64::from).sum::<u64>();
            let total_minutes = u32::try_from(seconds.div_ceil(60) + u64::from(option.duration_minutes))
                .map_err(|_| "Activity time is too large.")?;
            let cash = route.fare_clp.checked_add(option.admission_clp).ok_or("Activity cash cost is too large.")?;
            let time_cost = (f64::from(route.in_vehicle_seconds) * f64::from(self.time_values.in_vehicle_clp_per_hour)
                           + f64::from(route.wait_seconds) * f64::from(self.time_values.wait_clp_per_hour)
                           + f64::from(route.walk_seconds) * f64::from(self.time_values.walk_clp_per_hour)
                           + f64::from(route.cycle_seconds) * f64::from(self.time_values.cycle_clp_per_hour)) / 3600.0;
            let generalized_cost = u64::from(cash) + time_cost.round() as u64;
            let preference = i32::try_from(i64::from(option.interest_clp) - generalized_cost as i64)
                .map_err(|_| "Activity utility is outside the supported range.")?;
            let available = option.open && option.places_available > 0;
            let feasible = available && total_minutes <= self.available_minutes && cash <= self.budget_clp;
            choices.push(VisitOption { choice: option.id.as_str(), total_minutes,
                cost_clp: cash, available, preference });
            options.push(json!({
                "option": option, "total_minutes": total_minutes, "out_of_pocket_clp": cash,
                "time_cost_clp": time_cost.round() as u64, "generalized_cost_clp": generalized_cost,
                "utility_clp": preference, "feasible": feasible,
            }));
        }
        let choice = choose_visit(&choices, self.available_minutes, self.budget_clp, 0)
            .copied().unwrap_or("stay_home");
        Ok(json!({
            "person": self.person, "home_cut": home.cut, "synthetic_role": role, "age_band": age_band,
            "choice": choice, "available_minutes": self.available_minutes, "budget_clp": self.budget_clp,
            "preference_basis": self.preference_basis, "time_values": self.time_values, "options": options,
            "basis": "conditional_choice_for_one_synthetic_person",
            "capacity_basis": "supplied_places_at_the_venue",
        }))
    }
}
