"""Build the Santiago major-road display from the OSM extract described in the pack README."""
import json
import math
import xml.etree.ElementTree as ET
from pathlib import Path

PACK = Path("data/mvp")
RAW = Path("data/raw/osm-roads/santiago.osm")
VIEW = (-70.85, -33.65, -70.45, -33.30)
X_METRES = 111_320 * math.cos(math.radians(-33.45))
Y_METRES = 110_540


def read_roads(raw=RAW):
    points, ways = {}, []
    for _, item in ET.iterparse(raw):
        if item.tag == "node":
            points[int(item.get("id"))] = (float(item.get("lon")), float(item.get("lat")))
            item.clear()
        elif item.tag == "way":
            tags = {t.get("k"): t.get("v") for t in item.findall("tag")}
            nodes = [int(n.get("ref")) for n in item.findall("nd")]
            ways.append((int(item.get("id")), tags, nodes))
            item.clear()
    return points, ways


def distance(a, b):
    return math.hypot((a[0] - b[0]) * X_METRES, (a[1] - b[1]) * Y_METRES)


def simplify(path, tolerance):
    """Douglas-Peucker simplification in local metres."""
    ax, ay = path[0]
    bx, by = path[-1]
    dx, dy = (bx - ax) * X_METRES, (by - ay) * Y_METRES
    length2 = dx * dx + dy * dy
    furthest, error = 0, tolerance
    for i in range(1, len(path) - 1):
        x = (path[i][0] - ax) * X_METRES
        y = (path[i][1] - ay) * Y_METRES
        t = max(0, min(1, (x * dx + y * dy) / length2)) if length2 else 0
        d = math.hypot(x - t * dx, y - t * dy)
        if d > error:
            furthest, error = i, d
    if furthest:
        return simplify(path[:furthest + 1], tolerance)[:-1] + simplify(path[furthest:], tolerance)
    return [path[0], path[-1]]


def clipped_segments(path):
    """Clip each segment to the rectangular city view. Split at each excursion."""
    xmin, ymin, xmax, ymax = VIEW
    part = []
    for a, b in zip(path, path[1:]):
        x, y = a
        dx, dy = b[0] - x, b[1] - y
        lo, hi = 0, 1
        for p, q in ((-dx, x - xmin), (dx, xmax - x), (-dy, y - ymin), (dy, ymax - y)):
            if p < 0:
                lo = max(lo, q / p)
            elif p > 0:
                hi = min(hi, q / p)
            elif q < 0:
                hi = -1
        if hi <= lo:
            if part:
                yield part
                part = []
            continue
        start = a if lo == 0 else (x + lo * dx, y + lo * dy)
        end = b if hi == 1 else (x + hi * dx, y + hi * dy)
        if not part:
            part = [start]
        part.append(end)
        if hi < 1:
            yield part
            part = []
    if part:
        yield part


def rounded_path(path, tolerance):
    points = []
    for lon, lat in simplify(path, tolerance):
        p = [round(lon, 5), round(lat, 5)]
        if not points or p != points[-1]:
            points.append(p)
    return points


def write_json(file, value):
    file.write_text(json.dumps(value, separators=(",", ":"), ensure_ascii=False) + "\n", encoding="utf8")


if __name__ == "__main__":
    points, ways = read_roads()
    features, ids, metres = [], set(), 0
    for way_id, tags, nodes in ways:
        for part in clipped_segments([points[n] for n in nodes]):
            path = rounded_path(part, 8)
            if len(path) < 2:
                continue
            ids.add(way_id)
            metres += sum(distance(a, b) for a, b in zip(part, part[1:]))
            features.append({"type": "Feature", "properties": {"id": way_id,
                             "kind": tags["highway"].removesuffix("_link"), "name": tags.get("name", "")},
                             "geometry": {"type": "LineString", "coordinates": path}})
    output = PACK / "roads.geojson"
    write_json(output, {"type": "FeatureCollection", "features": features})
    print(f"{len(ids)} ways, {len(features)} paths, {metres / 1000:.1f} km, {output.stat().st_size} bytes")
