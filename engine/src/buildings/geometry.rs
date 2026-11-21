use serde_json::Value;

type Point = [f64; 2];

fn rings(geometry: &Value) -> Result<Vec<Vec<Point>>, String> {
    if geometry["type"] != "Polygon" {
        return Err("Building geometry must be a WGS84 Polygon.".into());
    }
    let rings: Vec<Vec<Point>> = serde_json::from_value(geometry["coordinates"].clone())
        .map_err(|_| "Building coordinates must be [longitude, latitude] pairs.")?;
    if rings.is_empty() {
        return Err("A building needs an outer footprint.".into());
    }
    for ring in &rings {
        if ring.len() < 4 || ring.first() != ring.last() {
            return Err(
                "Each building ring needs at least three vertices and a closed boundary.".into(),
            );
        }
        if ring
            .iter()
            .any(|&[lon, lat]| !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat))
        {
            return Err("Building coordinates are outside the WGS84 range.".into());
        }
    }
    Ok(rings)
}

fn side([ax, ay]: Point, [bx, by]: Point, [px, py]: Point) -> f64 {
    (bx - ax) * (py - ay) - (by - ay) * (px - ax)
}

fn on_segment(a: Point, b: Point, p: Point) -> bool {
    side(a, b, p) == 0.0 && (0..2).all(|i| (a[i].min(b[i])..=a[i].max(b[i])).contains(&p[i]))
}

fn intersects(a: &[Point], b: &[Point]) -> bool {
    let (p, q, r, s) = (a[0], a[1], b[0], b[1]);
    (side(p, q, r) * side(p, q, s) < 0.0 && side(r, s, p) * side(r, s, q) < 0.0)
        || on_segment(p, q, r)
        || on_segment(p, q, s)
        || on_segment(r, s, p)
        || on_segment(r, s, q)
}

fn inside(ring: &[Point], p: Point) -> bool {
    let mut inside = false;
    for edge in ring.windows(2) {
        let (a, b) = (edge[0], edge[1]);
        if on_segment(a, b, p) {
            return true;
        }
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < a[0] + (p[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1])
        {
            inside = !inside;
        }
    }
    inside
}

pub(super) fn contains(geometry: &Value, point: Point) -> bool {
    rings(geometry).is_ok_and(|rings| {
        inside(&rings[0], point) && !rings[1..].iter().any(|hole| inside(hole, point))
    })
}

// Local tangent-plane scale is sufficient for a Santiago building footprint.
fn measure(geometry: &Value) -> Result<(f64, Point), String> {
    let rings = rings(geometry)?;
    let origin = rings[0][0];
    let mut net_twice_area = 0.;
    let mut moments = [0.; 2];
    for (index, ring) in rings.iter().enumerate() {
        let mut twice_area = 0.;
        let mut ring_moments = [0.; 2];
        for (i, edge) in ring.windows(2).enumerate() {
            let (a, b) = (edge[0], edge[1]);
            if a == b
                || ring
                    .windows(2)
                    .enumerate()
                    .skip(i + 2)
                    .any(|(j, other)| !(i == 0 && j == ring.len() - 2) && intersects(edge, other))
            {
                return Err("The building has a repeated edge or a self-intersecting ring.".into());
            }
            let cross = side(origin, a, b);
            twice_area += cross;
            for k in 0..2 {
                ring_moments[k] += ((a[k] - origin[k]) + (b[k] - origin[k])) * cross;
            }
        }
        if twice_area == 0. {
            return Err("Each building ring needs positive area.".into());
        }
        for (prior_index, prior) in rings[..index].iter().enumerate() {
            if prior
                .windows(2)
                .any(|a| ring.windows(2).any(|b| intersects(a, b)))
                || (prior_index == 0 && !inside(prior, ring[0]))
                || (prior_index > 0 && (inside(prior, ring[0]) || inside(ring, prior[0])))
            {
                return Err(
                    "Courtyards must be disjoint and inside the building footprint.".into(),
                );
            }
        }
        let sign = if index == 0 {
            twice_area.signum()
        } else {
            -twice_area.signum()
        };
        net_twice_area += sign * twice_area;
        for k in 0..2 {
            moments[k] += sign * ring_moments[k];
        }
    }
    let centroid = [
        origin[0] + moments[0] / (3. * net_twice_area),
        origin[1] + moments[1] / (3. * net_twice_area),
    ];
    let metres_per_degree = 6_371_008.8_f64 * std::f64::consts::PI / 180.;
    let area = net_twice_area * 0.5 * metres_per_degree.powi(2) * centroid[1].to_radians().cos();
    Ok((area, centroid))
}

pub(super) fn area_m2(geometry: &Value) -> Result<f64, String> {
    Ok(measure(geometry)?.0)
}

pub(super) fn centroid(geometry: &Value) -> Result<Point, String> {
    Ok(measure(geometry)?.1)
}
