use super::{GLASS, Mesh, Path, Quat, Rgb, Vec3, Vehicle, block, oval, shape};

// Local cubic ramps have zero endpoint speed and join cruise speed smoothly.
// A sample outside the ramps keeps its original route time.
fn elapsed_distance_fraction(elapsed: f32, duration: f32) -> f32 {
    let ramp = 6.0_f32.min(duration * 0.5);
    let ease = |time: f32| {
        let t = time / ramp;
        ramp * t * t * (2.0 - t)
    };
    let travelled = if elapsed < ramp {
        ease(elapsed)
    } else if duration - elapsed < ramp {
        duration - ease(duration - elapsed)
    } else {
        elapsed
    };
    travelled / duration
}

impl Path {
    // Road legs slow only at their endpoints. A bend in a polyline is not a stop.
    pub(super) fn trip_pose(&self, age: f32, person: usize) -> Option<(Vec3, Vec3, Vehicle, f32)> {
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
            return self
                .at(age, &self.seconds)
                .map(|(p, d, kind)| (p, d, kind, 0.0));
        }
        if leg.vehicle == Vehicle::Walk {
            let elapsed = walking_time(age - start, span, person);
            let metres = self.metres[first] + elapsed * Vehicle::Walk.speed();
            return self
                .at(start + elapsed, &self.seconds)
                .map(|(p, d, kind)| (p, d, kind, metres));
        }
        if !matches!(leg.vehicle, Vehicle::Car | Vehicle::Bus) {
            return self
                .at(age, &self.seconds)
                .map(|(p, d, kind)| (p, d, kind, 0.0));
        }
        let progress = elapsed_distance_fraction(age - start, span);
        let distance = self.metres[first] + progress * (self.metres[leg.end] - self.metres[first]);
        let distance = self.corner_distance(distance, leg.vehicle, first, leg.end);
        let (pos, dir) = self.road_pose(distance, leg.vehicle, first, leg.end)?;
        Some((pos, dir, leg.vehicle, distance))
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

// The cached meshes use the same body dimensions as the standing passenger.
// Bend the knee between the hip and ankle while the shoe follows its contact arc.
pub(super) fn pedestrian(frame: Option<u8>, ink: bool) -> Mesh {
    let coat = if ink { [255; 3] } else { [36, 134, 171] };
    let skin = if ink { [255; 3] } else { [248, 189, 137] };
    let trousers = if ink { [255; 3] } else { GLASS };
    let side_pose = |left| {
        frame.map(|f| walk_pose(f, left)).unwrap_or(WalkPose {
            foot_x: 0.0,
            foot_lift: 0.0,
            shoulder: 0.0,
            hip_lift: 0.0,
        })
    };
    let lift = side_pose(true).hip_lift;
    let mut parts = vec![
        oval([0.10, 0.125, 0.095], [0., 1.625 + lift, 0.], skin),
        block([0.105, 0.14, 0.11], [0., 1.485 + lift, 0.], skin),
        oval([0.15, 0.36, 0.23], [0., 1.15 + lift, 0.], coat),
    ];
    let limb = |a: Vec3, b: Vec3, radius: f32, color: Rgb| {
        let offset = b - a;
        oval([radius, offset.length() * 0.5, radius], [0.; 3], color)
            .rotated_by(Quat::from_rotation_arc(Vec3::Y, offset.normalize()))
            .translated_by((a + b) * 0.5)
    };
    for (left, side) in [(true, 1.), (false, -1.)] {
        let step = side_pose(left);
        let hip = Vec3::new(0., 0.82 + lift, 0.095 * side);
        let ankle = Vec3::new(step.foot_x, 0.095 + step.foot_lift, hip.z);
        let hip_to_ankle = ankle - hip;
        let forward = Vec3::new(-hip_to_ankle.y, hip_to_ankle.x, 0.).normalize();
        let bend = (0.43_f32.powi(2) - hip_to_ankle.length_squared() * 0.25)
            .max(0.)
            .sqrt();
        let knee = (hip + ankle) * 0.5 + forward * bend;
        let shoulder = Vec3::new(0., 1.42 + lift, 0.22 * side);
        let wrist = shoulder + Vec3::new(step.shoulder.sin(), -step.shoulder.cos(), 0.) * 0.59;
        parts.push(limb(shoulder, wrist, 0.065, coat));
        parts.push(limb(hip, knee, 0.09, trousers));
        parts.push(limb(knee, ankle, 0.078, trousers));
        parts.push(block(
            [0.23, 0.10, 0.13],
            [ankle.x + 0.045, ankle.y - 0.045, ankle.z],
            trousers,
        ));
    }
    shape(parts)
}
