use super::{ArcEdge, Segment};
use serde_json::Value;

pub(super) fn speed_for_class(class: &str) -> Option<f64> {
    Some(match class {
        "motorway" => 90.,
        "motorway_link" => 45.,
        "trunk" => 85.,
        "trunk_link" => 40.,
        "primary" => 65.,
        "primary_link" => 30.,
        "secondary" => 55.,
        "secondary_link" => 25.,
        _ => return None,
    })
}

fn directions(value: &Value) -> [bool; 2] {
    match value {
        Value::Null | Value::Bool(false) => [true, true],
        Value::Bool(true) => [true, false],
        _ => match value.as_str() {
            Some("yes" | "1" | "true") => [true, false],
            Some("-1") => [false, true],
            Some("no" | "0" | "false") => [true, true],
            _ => [false, false],
        },
    }
}

fn allowed(value: Option<&Value>) -> bool {
    match value {
        None => true, // A user-added road has no source access tag.
        Some(value) => value.as_bool().unwrap_or_else(|| {
            matches!(
                value.as_str(),
                Some("yes" | "true" | "1" | "permissive" | "designated" | "official")
            )
        }),
    }
}

fn speed(props: &Value, direction: &str, cap_key: &str) -> Option<(f64, bool)> {
    if let Some(speed) = props["speed_kmh"].as_f64() {
        return Some((speed, false));
    }
    let class = props["routing_class"]
        .as_str()
        .or_else(|| props["class"].as_str())?;
    let class_speed = speed_for_class(class)?;
    let source = &props["source_speed_by_direction"][direction];
    // Older packs expose the effective cap only when the speed limit bound the speed.
    let cap = match props.get(cap_key) {
        Some(raw) => raw.as_f64(),
        None if source["speed_source"] != "class_assumption" => source["speed_kmh"].as_f64(),
        None => None,
    };
    Some(match cap {
        Some(limit) if limit <= class_speed => (limit, false),
        _ => (class_speed, true),
    })
}

pub(super) fn edges(segments: &[Segment], props: &Value) -> Vec<ArcEdge> {
    let [forward, backward] = directions(&props["oneway"]);
    let mut edges = Vec::new();
    for (permitted, direction, access, cap, forward) in [
        (
            forward,
            "forward",
            "car_access_forward",
            "speed_cap_forward_kmh",
            true,
        ),
        (
            backward,
            "backward",
            "car_access_backward",
            "speed_cap_backward_kmh",
            false,
        ),
    ] {
        if !permitted || !allowed(props.get(access)) {
            continue;
        }
        let Some((speed, assumed)) = speed(props, direction, cap) else {
            continue;
        };
        for segment in segments {
            let (from, to) = if forward {
                (segment.from, segment.to)
            } else {
                (segment.to, segment.from)
            };
            edges.push(ArcEdge {
                from,
                to,
                metres: segment.metres,
                speed,
                forward,
                assumed,
            });
        }
    }
    edges
}
