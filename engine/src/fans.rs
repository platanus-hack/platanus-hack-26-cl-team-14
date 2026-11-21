//! Match attendance from a fixed fan base. The fans live where they live, whatever the venue:
//! a comuna sends attendees in proportion to its Census 2024 population. Moving the venue changes
//! how they travel (the commute mode logit of choice.rs for the pair home comuna → venue comuna), not who they are.
use crate::choice::{Choice, shares};
use serde_json::{Value, json};
use std::path::Path;

pub const ASSUMPTIONS: [&str; 3] = [
    "Fan base: attendees come from every comuna in proportion to its Census 2024 population. Colo-Colo has about 45–49% of fans in Greater Santiago (GfK Adimark 2017–2018); the share by socioeconomic group (Cadem 2024: 24% C1 to 49% D/E) is not applied yet.",
    "The same fans go to the match at any venue; the attendance does not fall with travel time.",
    "Mode to the match = the commute mode shares of the pair home comuna → venue comuna (Census 2024 calibration). Weekend service and parking at the venue are not modeled.",
];

pub struct Fans { population: Vec<f64> }

impl Fans {
    pub fn load(dir: &Path) -> Self {
        let population = csv::Reader::from_path(dir.join("communes.csv")).unwrap().records()
            .map(|r| r.unwrap()[2].parse().unwrap()).collect();
        Fans { population }
    }

    /// Attendees by home comuna and by mode [public, car, other] for a match of `attendance` in `venue_cut`.
    pub fn project(&self, c: &Choice, venue_cut: &str, attendance: f64) -> Value {
        let (n, d) = (c.n, c.cuts.iter().position(|x| x == venue_cut).unwrap());
        let total: f64 = self.population.iter().sum();
        let (mut mode, mut minutes, mut by_home) = ([0.0; 3], 0.0, vec![]);
        for o in 0..n {
            let a = attendance * self.population[o] / total;
            let s = shares(&c.mode_coef, c.minutes[o * n + d], c.km[o * n + d], &c.mode_k[o * n + d]);
            for j in 0..3 { mode[j] += a * s[j]; }
            minutes += a * s[0] * c.minutes[o * n + d];
            by_home.push(json!({ "cut": c.cuts[o], "name": c.names[o], "attendees": a, "public": a * s[0], "car": a * s[1], "other": a * s[2] }));
        }
        json!({
            "venue_cut": venue_cut, "venue_comuna": c.names[d], "attendance": attendance,
            "by_mode": { "public": mode[0], "car": mode[1], "other": mode[2] },
            "public_mean_minutes": minutes / mode[0],
            "by_home": by_home, "assumptions": ASSUMPTIONS,
        })
    }
}
