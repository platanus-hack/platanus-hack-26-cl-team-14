use serde_json::Value;

fn line(coordinates: &Value) -> Result<Vec<[f64; 2]>, String> {
    let positions = coordinates
        .as_array()
        .ok_or("Road coordinates must be an array.")?;
    let mut points = Vec::with_capacity(positions.len());
    for position in positions {
        let ordinates = position
            .as_array()
            .ok_or("A road position must be a coordinate array.")?;
        if ordinates.len() < 2
            || !ordinates
                .iter()
                .all(|v| v.as_f64().is_some_and(f64::is_finite))
        {
            return Err("Road positions need finite longitude and latitude.".into());
        }
        let lon = ordinates[0].as_f64().unwrap();
        let lat = ordinates[1].as_f64().unwrap();
        if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
            return Err("Road coordinates must be WGS84 longitude and latitude.".into());
        }
        points.push([lon, lat]);
    }
    if points.len() < 2 || points.iter().all(|p| *p == points[0]) {
        return Err("Each road line needs at least two distinct positions.".into());
    }
    Ok(points)
}

pub(super) fn lines(geometry: &Value) -> Result<Vec<Vec<[f64; 2]>>, String> {
    let coordinates = &geometry["coordinates"];
    match geometry["type"].as_str() {
        Some("LineString") => Ok(vec![line(coordinates)?]),
        Some("MultiLineString") => {
            let parts = coordinates
                .as_array()
                .filter(|p| !p.is_empty())
                .ok_or("A MultiLineString needs at least one line.")?;
            parts.iter().map(line).collect()
        }
        _ => Err("Road geometry must be a LineString or MultiLineString.".into()),
    }
}

pub(super) fn changed_feature(
    id: &str,
    before: Option<&Value>,
    change: &Value,
) -> Result<Option<Value>, String> {
    let op = change["op"]
        .as_str()
        .ok_or("A road change needs an operation.")?;
    let geometry = change.get("geometry");
    let properties = change.get("properties");
    match op {
        "add" if before.is_some() => return Err(format!("Road {id} already exists.")),
        "remove" | "set" | "move" if before.is_none() => return Err(format!("Unknown road {id}.")),
        "add" | "set" | "move" | "remove" => (),
        _ => return Err(format!("Unknown road operation {op}.")),
    }
    if geometry.is_some() && !matches!(op, "add" | "move") {
        return Err("Use move to change an existing road's geometry.".into());
    }
    if properties.is_some() && !matches!(op, "add" | "set") {
        return Err("Use set to change an existing road's properties.".into());
    }
    if op == "remove" {
        return Ok(None);
    }

    let mut feature = before.cloned().unwrap_or_else(|| {
        serde_json::json!({
            "type": "Feature", "id": id, "properties": {"kind": "road", "class": "road", "name": ""}
        })
    });
    feature["properties"]["id"] = id.into();
    if feature.get("id").is_some() {
        feature["id"] = id.into();
    }
    if matches!(op, "add" | "move") {
        let geometry = geometry.ok_or("Adding or moving a road needs geometry.")?;
        if geometry["type"] == "MultiLineString"
            && !before.is_some_and(|f| f["geometry"]["type"] == "MultiLineString")
        {
            return Err("Use LineString unless moving an existing multipart road.".into());
        }
        lines(geometry)?;
        feature["geometry"] = geometry.clone();
    }
    if properties.is_some() || op == "set" {
        let patch = properties
            .and_then(Value::as_object)
            .ok_or("Road properties must be an object.")?;
        if op == "set" && patch.is_empty() {
            return Err("Set needs at least one road property.".into());
        }
        for (key, value) in patch {
            match key.as_str() {
                "class" if value.as_str().is_some_and(|s| !s.trim().is_empty()) => {
                    feature["properties"]["kind"] = value.clone();
                }
                "speed_kmh" if value.as_f64().is_some_and(|n| n.is_finite() && n > 0.0) => (),
                "oneway" => {
                    let direction = match (value.as_bool(), value.as_str()) {
                        (Some(true), _) | (_, Some("yes")) => "yes",
                        (Some(false), _) | (_, Some("no")) => "no",
                        (_, Some("-1")) => "-1",
                        _ => return Err("oneway must be true, false, yes, no, or -1.".into()),
                    };
                    feature["properties"]["oneway"] = direction.into();
                    continue;
                }
                "name" if op == "add" && value.is_string() => (),
                _ => return Err(format!("Invalid road property {key} for {op}.")),
            }
            feature["properties"][key] = value.clone();
        }
    }
    Ok(Some(feature))
}
