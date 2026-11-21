use super::{Path, Vec3, Vehicle};

// Integral of a trapezoidal speed profile. Its endpoints keep the existing trip clock.
fn elapsed_distance_fraction(elapsed: f32, duration: f32) -> f32 {
    let ramp = 6.0_f32.min(duration * 0.5);
    let cruise = duration - ramp;
    if elapsed < ramp {
        elapsed * elapsed / (2.0 * ramp * cruise)
    } else if duration - elapsed < ramp {
        1.0 - (duration - elapsed).powi(2) / (2.0 * ramp * cruise)
    } else {
        (elapsed - ramp * 0.5) / cruise
    }
}

impl Path {
    // Road legs slow only at their endpoints. A bend in a polyline is not a stop.
    pub(super) fn trip_pose(&self, age: f32, person: usize) -> Option<(Vec3, Vec3, Vehicle)> {
        let duration = *self.seconds.last()?;
        if age < 0.0 || age > duration || duration == 0.0 {
            return None;
        }
        let leg_index = self
            .legs
            .partition_point(|leg| self.seconds[leg.end] <= age)
            .min(self.legs.len() - 1);
        let leg = &self.legs[leg_index];
        let first = if leg_index == 0 {
            0
        } else {
            self.legs[leg_index - 1].end
        };
        let start = self.seconds[first];
        let span = self.seconds[leg.end] - start;
        if span == 0.0 {
            return self.at(age, &self.seconds);
        }
        if leg.vehicle == Vehicle::Walk {
            return self.at(
                start + walking_time(age - start, span, person),
                &self.seconds,
            );
        }
        if !matches!(leg.vehicle, Vehicle::Car | Vehicle::Bus) {
            return self.at(age, &self.seconds);
        }
        let progress = elapsed_distance_fraction(age - start, span);
        let distance = self.metres[first] + progress * (self.metres[leg.end] - self.metres[first]);
        let distance = self.corner_distance(distance, leg.vehicle, first, leg.end);
        let (pos, dir) = self.road_pose(distance, leg.vehicle, first, leg.end)?;
        Some((pos, dir, leg.vehicle))
    }

    pub(super) fn road_pose(
        &self,
        distance: f32,
        vehicle: Vehicle,
        first: usize,
        last: usize,
    ) -> Option<(Vec3, Vec3)> {
        let (position, segment_direction, _) = self.at(distance, &self.metres)?;
        // Two samples one display wheelbase apart turn the body gradually at a path vertex.
        let half_wheelbase = if vehicle == Vehicle::Bus { 3.0 } else { 1.35 };
        let front = self.route_point(distance + half_wheelbase, first, last);
        let rear = self.route_point(distance - half_wheelbase, first, last);
        let direction = (front - rear).try_normalize().unwrap_or(segment_direction);
        Some((position, direction))
    }

    // Ease through the bend, with the same times at the vertex and both window edges.
    // Adjacent bends use disjoint windows even on a densely sampled street.
    fn corner_distance(&self, distance: f32, vehicle: Vehicle, first: usize, last: usize) -> f32 {
        let upper = self
            .metres
            .partition_point(|&m| m < distance)
            .clamp(first + 1, last);
        let lower = upper - 1;
        let vertex = if distance - self.metres[lower] < self.metres[upper] - distance {
            lower
        } else {
            upper
        };
        if vertex <= first || vertex >= last {
            return distance;
        }
        let radius = (vehicle.speed() * 2.0)
            .min(0.5 * (self.metres[vertex] - self.metres[vertex - 1]))
            .min(0.5 * (self.metres[vertex + 1] - self.metres[vertex]));
        let offset = distance - self.metres[vertex];
        if offset.abs() >= radius {
            return distance;
        }
        let entering = (self.points[vertex] - self.points[vertex - 1]).normalize();
        let leaving = (self.points[vertex + 1] - self.points[vertex]).normalize();
        let strength = (1.0 - entering.dot(leaving)).clamp(0.0, 1.0) * 0.68;
        let window = 1.0 - (offset / radius).powi(2);
        distance - offset * strength * window * window
    }

    fn route_point(&self, distance: f32, first: usize, last: usize) -> Vec3 {
        let value = distance.clamp(self.metres[first], self.metres[last]);
        let upper = self.metres.partition_point(|&metres| metres <= value);
        if upper > last {
            return self.points[last];
        }
        let lower = upper - 1;
        let fraction = (value - self.metres[lower]) / (self.metres[upper] - self.metres[lower]);
        self.points[lower].lerp(self.points[upper], fraction)
    }

    // The timetable locates the front body. Both rigid halves meet at the same hinge.
    // Route history orients the rear half even when the replay jumps to a new time.
    pub(super) fn articulated_rear(
        &self,
        distance: f32,
        front: Vec3,
        heading: Vec3,
        front_to_hinge: f32,
        hinge_to_rear: f32,
    ) -> (Vec3, Vec3) {
        let hinge = front - heading * front_to_hinge;
        let last = self.points.len() - 1;
        let hinge_distance = distance - front_to_hinge;
        let rear_heading = (self.route_point(hinge_distance, 0, last)
            - self.route_point(hinge_distance - 2.0 * hinge_to_rear, 0, last))
        .try_normalize()
        .unwrap_or(heading);
        (hinge - rear_heading * hinge_to_rear, rear_heading)
    }
}

// The easing offset is zero at both endpoints. Walkers vary their pace within a leg
// without changing their boarding time or which trip is visible.
pub(super) fn walking_time(elapsed: f32, duration: f32, person: usize) -> f32 {
    let progress = elapsed / duration;
    let bias = ((person.wrapping_mul(0x9e3779b1) & 1023) as f32 / 1023.0 - 0.5) * 0.16;
    elapsed + bias * elapsed * (1.0 - progress)
}

// Each foot stays planted for half a cycle. The mesh frames share a physical stride.
// Scale the travelled distance into model space before selecting a frame.
pub(super) const GAIT_FRAMES: u8 = 12;
const STRIDE: f32 = 1.48;

pub(super) struct WalkPose {
    pub foot_x: f32,
    pub foot_lift: f32,
    pub shoulder: f32,
    pub hip_lift: f32,
}

pub(super) fn gait_frame(metres: f32, scale: f32, person: usize) -> u8 {
    let offset = (person.wrapping_mul(0x9e3779b1) & 1023) as f32 / 1024.0;
    let phase = (metres / (scale * STRIDE) + offset).rem_euclid(1.0);
    (phase * f32::from(GAIT_FRAMES)) as u8
}

pub(super) fn walk_pose(frame: u8, left: bool) -> WalkPose {
    let phase = (f32::from(frame) / f32::from(GAIT_FRAMES) + if left { 0.0 } else { 0.5 }).fract();
    let swing = (2.0 * phase - 1.0).max(0.0);
    let (foot_x, foot_lift) = if phase < 0.5 {
        (STRIDE * (0.25 - phase), 0.0)
    } else {
        let eased = swing * swing * (3.0 - 2.0 * swing);
        (
            STRIDE * (-0.25 + 0.5 * eased),
            0.12 * (std::f32::consts::PI * swing).sin(),
        )
    };
    WalkPose {
        foot_x,
        foot_lift,
        shoulder: -0.32 * (std::f32::consts::TAU * phase).cos(),
        hip_lift: 0.026 * (std::f32::consts::TAU * phase).sin().powi(2),
    }
}
