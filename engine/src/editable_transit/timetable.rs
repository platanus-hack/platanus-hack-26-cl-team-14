//! Stop times for the Monday referenced by the editable GTFS catalog.
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const DAY: &str = "20260413";
pub(super) const DEPARTURE_MINUTE: f64 = 8.0 * 60.0;
pub(super) const ARRIVAL_LIMIT: f64 = 11.0 * 60.0;

#[derive(Deserialize)]
pub(super) struct Calendar {
    service_id: String,
    monday: String,
    start_date: String,
    end_date: String,
}

#[derive(Deserialize)]
pub(super) struct Exception {
    service_id: String,
    date: String,
    exception_type: String,
}

pub(super) fn active_services(calendar: &[Calendar], exceptions: &[Exception]) -> BTreeSet<String> {
    let mut active: BTreeSet<_> = calendar
        .iter()
        .filter(|row| {
            row.monday == "1" && row.start_date.as_str() <= DAY && DAY <= row.end_date.as_str()
        })
        .map(|row| row.service_id.clone())
        .collect();
    for row in exceptions.iter().filter(|row| row.date == DAY) {
        if row.exception_type == "1" {
            active.insert(row.service_id.clone());
        } else {
            active.remove(&row.service_id);
        }
    }
    active
}

#[derive(Default, Deserialize)]
pub(super) struct EncodedSchedule {
    #[serde(default)]
    fixed_departures: Vec<(String, String, String)>,
    #[serde(default)]
    frequency_windows: Vec<(String, String, String, String, u32, u8)>,
    #[serde(default)]
    timing_profiles: Vec<Vec<[i32; 2]>>,
    #[serde(default)]
    trip_profile: BTreeMap<String, usize>,
}

#[derive(Clone, Default)]
pub(super) struct Starts {
    fixed: Vec<f64>,
    frequencies: Vec<Frequency>,
}

#[derive(Clone)]
struct Frequency {
    start: f64,
    end: f64,
    interval: f64,
    exact: bool,
}

pub(super) struct Schedule {
    pub(super) calls: Vec<[f64; 2]>,
    pub(super) starts: Starts,
}

fn minutes(time: &str) -> f64 {
    let mut fields = time.split(':').map(|part| part.parse::<f64>().unwrap());
    fields.next().unwrap() * 60.0 + fields.next().unwrap() + fields.next().unwrap() / 60.0
}

impl EncodedSchedule {
    pub(super) fn compile(self, active: &BTreeSet<String>) -> Vec<Schedule> {
        let mut starts = vec![Starts::default(); self.timing_profiles.len()];
        for (service, trip, time) in self.fixed_departures {
            if active.contains(&service) {
                starts[self.trip_profile[&trip]].fixed.push(minutes(&time));
            }
        }
        for (service, trip, start, end, headway, exact) in self.frequency_windows {
            if active.contains(&service) {
                starts[self.trip_profile[&trip]]
                    .frequencies
                    .push(Frequency {
                        start: minutes(&start),
                        end: minutes(&end),
                        interval: f64::from(headway) / 60.0,
                        exact: exact == 1,
                    });
            }
        }
        self.timing_profiles
            .into_iter()
            .zip(starts)
            .filter(|(_, starts)| !starts.fixed.is_empty() || !starts.frequencies.is_empty())
            .map(|(calls, mut starts)| {
                starts.fixed.sort_by(f64::total_cmp);
                let calls = calls
                    .into_iter()
                    .map(|call| call.map(|seconds| f64::from(seconds) / 60.0))
                    .collect();
                Schedule { calls, starts }
            })
            .collect()
    }
}

impl Starts {
    pub(super) fn assumed(interval: f64) -> Self {
        Self {
            fixed: Vec::new(),
            frequencies: vec![Frequency {
                start: DEPARTURE_MINUTE,
                end: DEPARTURE_MINUTE + 60.0,
                interval,
                exact: false,
            }],
        }
    }

    pub(super) fn combine(schedules: &[Schedule]) -> Self {
        let mut starts = Self::default();
        for schedule in schedules {
            starts.fixed.extend(&schedule.starts.fixed);
            starts
                .frequencies
                .extend(schedule.starts.frequencies.iter().cloned());
        }
        starts.fixed.sort_by(f64::total_cmp);
        starts
    }

    pub(super) fn overlaps(&self, first_offset: f64, last_offset: f64) -> bool {
        let earliest_start = DEPARTURE_MINUTE - last_offset;
        let latest_start = ARRIVAL_LIMIT - first_offset;
        let fixed = self.fixed.partition_point(|&time| time < earliest_start);
        self.fixed
            .get(fixed)
            .is_some_and(|&time| time <= latest_start)
            || self
                .frequencies
                .iter()
                .any(|window| window.end > earliest_start && window.start <= latest_start)
    }

    pub(super) fn next(&self, time: f64, offset: f64) -> f64 {
        let time_at_origin = time - offset;
        let i = self.fixed.partition_point(|&first| first < time_at_origin);
        let mut next = self.fixed.get(i).copied().unwrap_or(f64::INFINITY);
        for window in &self.frequencies {
            let observed = time_at_origin.max(window.start);
            let departure = if window.exact {
                window.start
                    + ((observed - window.start) / window.interval).ceil() * window.interval
            } else {
                observed + window.interval * 0.5
            };
            if departure < window.end {
                next = next.min(departure);
            }
        }
        (next + offset).max(time)
    }
}
