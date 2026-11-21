//! Weekday station entries per scheduled nominal place, using the April 13–17 timetable.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize)]
struct Rating {
    line: String,
    places_per_train: Option<f64>,
}

#[derive(Deserialize)]
struct Call {
    station_id: String,
    line: String,
    half_hour: String,
    trains: f64,
}

pub struct StationCapacity {
    places: HashMap<String, [Option<f64>; 48]>,
}

#[derive(Serialize)]
pub struct StationPressure {
    pub status: &'static str,
    pub modeled_station_entries: f64,
    pub scheduled_nominal_places: Option<f64>,
    pub entries_per_scheduled_place: Option<f64>,
}

impl StationCapacity {
    pub fn load(data: &Path) -> Self {
        let ratings: HashMap<_, _> = csv::Reader::from_path(data.join("metro_capacity.csv"))
            .unwrap()
            .deserialize::<Rating>()
            .map(Result::unwrap)
            .map(|r| (r.line, r.places_per_train))
            .collect();
        let mut places = HashMap::new();
        let mut service = csv::Reader::from_path(data.join("metro_station_service.csv")).unwrap();
        for call in service.deserialize::<Call>().map(Result::unwrap) {
            let (hour, minute) = call.half_hour.split_once(':').unwrap();
            let bin = hour.parse::<usize>().unwrap() * 2 + minute.parse::<usize>().unwrap() / 30;
            let day = places.entry(call.station_id).or_insert([Some(0.0); 48]);
            // One unknown line makes the combined denominator unknown.
            day[bin] = day[bin]
                .zip(ratings[&call.line])
                .map(|(total, rating)| total + call.trains * rating);
        }
        Self { places }
    }

    /// Use the scenario's station-access state and its modeled weekday entries for this bin.
    pub fn half_hour(&self, id: &str, bin: usize, entries: f64, open: bool) -> StationPressure {
        let places = if open {
            self.places.get(id).and_then(|day| day[bin])
        } else {
            Some(0.0)
        };
        let (status, ratio) = match places {
            _ if !open => ("station closed", None),
            None => ("capacity not verified", None),
            Some(0.0) => ("no scheduled service", None),
            Some(n) => ("available", Some(entries / n)),
        };
        StationPressure {
            status,
            modeled_station_entries: entries,
            scheduled_nominal_places: places,
            entries_per_scheduled_place: ratio,
        }
    }
}
