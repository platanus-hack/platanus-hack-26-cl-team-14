//! Counts people on one ordered route. Each interval carries its queue and in-progress work forward.

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    num::NonZeroU16,
};

#[derive(Deserialize)]
pub struct Activity {
    start_minute: u16,
    step_minutes: NonZeroU16,
    steps: NonZeroU16,
    segments: Vec<Segment>,
    arrivals: Vec<Arrival>,
}

#[derive(Deserialize)]
struct Segment {
    id: String,
    travel_steps: NonZeroU16,
    capacity_per_step: u32,
    visit: Option<Visit>,
}

#[derive(Deserialize)]
struct Visit {
    place_id: String,
    dwell_steps: NonZeroU16,
    starts_per_step: u32,
}

#[derive(Deserialize)]
struct Arrival {
    step: usize,
    segment: usize,
    participants: u32,
}

#[derive(Serialize)]
pub struct ActivityRun<'a> {
    start_minute: u16,
    end_minute: u16,
    step_minutes: u16,
    steps: Vec<ActivityStep<'a>>,
}

#[derive(Serialize)]
struct ActivityStep<'a> {
    minute: u16,
    open: bool,
    entered: u64,
    exited: u64,
    remaining: u64,
    segments: Vec<SegmentStep<'a>>,
}

#[derive(Serialize)]
struct SegmentStep<'a> {
    id: &'a str,
    visit_place_id: Option<&'a str>,
    admitted: u64,
    queued: u64,
    moving: u64,
    visiting: u64,
    visits_started: u64,
}

struct TimedPeople {
    due: VecDeque<u64>,
    present: u64,
}

impl TimedPeople {
    fn new(duration: NonZeroU16) -> Self {
        Self {
            due: VecDeque::from(vec![0; usize::from(duration.get())]),
            present: 0,
        }
    }

    fn complete(&mut self) -> u64 {
        let completed = self.due.pop_front().unwrap();
        self.due.push_back(0);
        self.present -= completed;
        completed
    }

    fn start(&mut self, count: u64) {
        *self.due.back_mut().unwrap() += count;
        self.present += count;
    }
}

struct SegmentState {
    queued: u64,
    moving: TimedPeople,
    visiting: TimedPeople,
    visits_started: u64,
}

impl Activity {
    pub fn run(&self) -> Result<ActivityRun<'_>, &'static str> {
        let step_minutes = self.step_minutes.get();
        let count = usize::from(self.steps.get());
        let end =
            u32::from(self.start_minute) + u32::from(self.steps.get()) * u32::from(step_minutes);
        if end > 1440 {
            return Err("An activity must fit in one local day.");
        }
        let ids: HashSet<_> = self
            .segments
            .iter()
            .map(|segment| segment.id.as_str())
            .collect();
        if ids.is_empty() || ids.len() != self.segments.len() || ids.contains("") {
            return Err("Provide an ordered route with distinct segment IDs.");
        }
        let mut arrivals = vec![Vec::new(); count + 1];
        for arrival in &self.arrivals {
            if arrival.step >= count || arrival.segment >= self.segments.len() {
                return Err("Each arrival needs an open step and a route segment.");
            }
            arrivals[arrival.step].push(arrival);
        }

        let mut states: Vec<_> = self
            .segments
            .iter()
            .map(|segment| SegmentState {
                queued: 0,
                moving: TimedPeople::new(segment.travel_steps),
                visiting: TimedPeople::new(
                    segment
                        .visit
                        .as_ref()
                        .map_or(NonZeroU16::MIN, |visit| visit.dwell_steps),
                ),
                visits_started: 0,
            })
            .collect();
        let mut entered = 0;
        let mut exited = 0;
        let mut steps = Vec::with_capacity(count + 1);

        for (step, scheduled) in arrivals.into_iter().enumerate() {
            let open = step < count;
            for (index, segment) in self.segments.iter().enumerate() {
                let state = &mut states[index];
                let completed = state.moving.complete();
                let returning = state.visiting.complete();
                state.visits_started = if open {
                    segment
                        .visit
                        .as_ref()
                        .map_or(0, |visit| completed.min(u64::from(visit.starts_per_step)))
                } else {
                    0
                };
                state.visiting.start(state.visits_started);
                let continuing = completed - state.visits_started + returning;
                if let Some(next) = states.get_mut(index + 1) {
                    next.queued += continuing;
                } else {
                    exited += continuing;
                }
            }
            for arrival in scheduled {
                let people = u64::from(arrival.participants);
                states[arrival.segment].queued += people;
                entered += people;
            }
            let segments: Vec<_> = self
                .segments
                .iter()
                .zip(&mut states)
                .map(|(segment, state)| {
                    let admitted = if open {
                        state.queued.min(u64::from(segment.capacity_per_step))
                    } else {
                        0
                    };
                    state.queued -= admitted;
                    state.moving.start(admitted);
                    SegmentStep {
                        id: &segment.id,
                        visit_place_id: segment.visit.as_ref().map(|visit| visit.place_id.as_str()),
                        admitted,
                        queued: state.queued,
                        moving: state.moving.present,
                        visiting: state.visiting.present,
                        visits_started: state.visits_started,
                    }
                })
                .collect();
            steps.push(ActivityStep {
                minute: self.start_minute + step as u16 * step_minutes,
                open,
                entered,
                exited,
                remaining: segments
                    .iter()
                    .map(|s| s.queued + s.moving + s.visiting)
                    .sum(),
                segments,
            });
        }
        Ok(ActivityRun {
            start_minute: self.start_minute,
            end_minute: end as u16,
            step_minutes,
            steps,
        })
    }
}
