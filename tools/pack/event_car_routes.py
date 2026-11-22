"""Build inbound driving and last-mile walking geometry for the two Monumental caches.

Usage: python -B tools/pack/event_car_routes.py REGION.osm.pbf A-cache.json B-cache.json
Use the November 20 regional extract described in data/mvp/README.md. Requires osmium and Shapely 2.
"""

import csv
import gzip
import heapq
import json
import math
import subprocess
import sys
from collections import defaultdict
from functools import lru_cache
from pathlib import Path
from tempfile import TemporaryDirectory

from shapely import LineString, Point, Polygon, STRtree, union_all
from shapely.geometry import shape

from car_paths import SPEEDS
from car_water_spans import parts
from editable_roads import car_allowed, posted_cap, travel_direction
from roads import PACK, distance, read_roads

# Gate on the mapped stadium parking access. B translates the same physical fixture.
GATE = 12471507127
WALK_EXCLUDED = {
    "motorway",
    "motorway_link",
    "trunk",
    "trunk_link",
    "construction",
    "proposed",
    "raceway",
}
DATA = Path("web/data")


def run(*args):
    subprocess.run(["osmium", *args], check=True)


def result(path):
    cache = json.loads(Path(path).read_bytes())
    return cache.get("result", cache)


def directed_roads(points, ways):
    reverse, majors, regional_anchors = defaultdict(list), set(), set()
    tags_by_way = {}
    for wid, tags, ns in ways:
        tags_by_way[wid] = tags
        kind = tags["highway"]
        oneway, _ = travel_direction(tags)
        for direction in ("forward", "backward"):
            if not car_allowed(tags, direction) or oneway == "unknown":
                continue
            if (oneway == "yes" and direction == "backward") or (
                oneway == "-1" and direction == "forward"
            ):
                continue
            cap, _ = posted_cap(tags, direction)
            speed = min(SPEEDS[kind], cap or SPEEDS[kind])
            ordered = ns if direction == "forward" else reversed(ns)
            sequence = list(ordered)
            for a, b in zip(sequence, sequence[1:]):
                if a != b:
                    reverse[b].append(
                        (a, wid, round(distance(points[a], points[b]), 3) * 3.6 / speed)
                    )
            if kind in {"primary", "secondary"}:
                regional_anchors.update(ns)
                if tags.get("tunnel", "no") == tags.get("bridge", "no") == "no":
                    majors.update(ns)
    return reverse, majors, sorted(regional_anchors), tags_by_way


def origin_nodes(points, candidates):
    clipped = json.loads((PACK / "road_graph.json").read_bytes())["anchors"]
    reference = json.loads((DATA / "car_routes.json").read_bytes())
    packed = {k.split("-")[0]: path[0] for k, path in reference.items()}
    anchors = {}
    with (PACK / "communes.csv").open() as file:
        for row in csv.DictReader(file):
            cut = row["cut"]
            if clipped[cut] is not None:
                anchors[cut] = int(clipped[cut])
            else:
                pos = tuple(map(float, (row["longitude"], row["latitude"])))
                anchors[cut] = min(candidates, key=lambda n: distance(points[n], pos))
                if (
                    list(map(lambda v: round(v, 5), points[anchors[cut]]))
                    != packed[cut]
                ):
                    raise ValueError(
                        f"Regional origin differs from the published car origin for {cut}"
                    )
    return anchors


def intersects_bowl(a, b, bowl):
    xmin, ymin, xmax, ymax = bowl.bounds
    if (
        max(a[0], b[0]) < xmin
        or min(a[0], b[0]) > xmax
        or max(a[1], b[1]) < ymin
        or min(a[1], b[1]) > ymax
    ):
        return False
    return LineString([a, b]).intersects(bowl)


def paths_to(reverse, dest, closed, points, bowl):
    times, next_step, queue = {dest: 0.0}, {}, [(0.0, dest)]
    while queue:
        seconds, node = heapq.heappop(queue)
        if times[node] != seconds:
            continue
        for start, wid, step in reverse[node]:
            value = seconds + step
            if value >= times.get(start, math.inf) or wid in closed:
                continue
            if intersects_bowl(points[start], points[node], bowl):
                continue
            times[start] = value
            next_step[start] = (node, wid)
            heapq.heappush(queue, (value, start))
    return times, next_step


def walk_to_gate(gate, major, reverse, closed, bowl, scratch):
    # A 2 km square includes both access streets. Only connected OSM nodes form junctions.
    x, y = gate
    run(
        "extract",
        "-b",
        f"{x - 0.012},{y - 0.010},{x + 0.012},{y + 0.010}",
        scratch / "region.pbf",
        "-o",
        scratch / "walk.pbf",
        "--overwrite",
    )
    run(
        "tags-filter",
        scratch / "walk.pbf",
        "w/highway",
        "-o",
        scratch / "walk.osm",
        "--overwrite",
    )
    points, ways = read_roads(scratch / "walk.osm")
    adjacency, terminal = defaultdict(list), []
    gm = Point(gate)
    for wid, tags, ns in ways:
        hw = tags["highway"]
        if (
            wid in closed
            or hw in WALK_EXCLUDED
            or tags.get("foot", tags.get("access")) in {"private", "no", "customers"}
        ):
            continue
        if (
            tags.get("bridge", "no") != "no"
            or tags.get("tunnel", "no") != "no"
            or tags.get("area") == "yes"
        ):
            continue
        for a, b in zip(ns, ns[1:]):
            xy, yz = points[a], points[b]
            if a == b or intersects_bowl(xy, yz, bowl):
                continue
            length = distance(xy, yz)
            adjacency[a].append((b, length))
            adjacency[b].append((a, length))
            segment = LineString([xy, yz])
            snap = tuple(segment.interpolate(segment.project(gm)).coords[0])
            gap = distance(gate, snap)
            if gap < 0.01 or (hw == "footway" and gap <= 0.9):
                terminal.append((gap, a, b, snap, wid))
    gap, a, b, snap, way = min(terminal)
    # The translated gate falls within the mapped 1.8 m footway. The last
    # transverse movement stays within its half-width.
    dist = {a: distance(points[a], snap), b: distance(points[b], snap)}
    queue = [(v, n) for n, v in dist.items()]
    heapq.heapify(queue)
    toward = {}
    while queue:
        length, node = heapq.heappop(queue)
        if dist[node] != length:
            continue
        if node in major and any(w not in closed for _, w, _ in reverse[node]):
            path, cursor = [points[node]], node
            while cursor in toward:
                cursor = toward[cursor]
                path.append(points[cursor])
            for xy in (snap, gate):
                if distance(path[-1], xy) > 0.005:
                    path.append(xy)
            return node, path, length + gap, way, gap
        for nxt, step in adjacency[node]:
            if length + step < dist.get(nxt, math.inf):
                dist[nxt] = length + step
                toward[nxt] = node
                heapq.heappush(queue, (length + step, nxt))
    raise ValueError("No open connected walking route from a major road to the gate")


def water_index(region, scratch):
    subset = scratch / "water.pbf"
    run(
        "tags-filter",
        region,
        "wr/natural=water",
        "wr/waterway=riverbank",
        "wr/landuse=reservoir",
        "-o",
        subset,
    )
    features = json.loads(
        subprocess.check_output(
            ["osmium", "export", "--geometry-types", "polygon", "-f", "geojson", subset]
        )
    )["features"]
    water = [
        shape(f["geometry"])
        for f in features
        if any(
            f["properties"].get(k) == v
            for k, v in (
                ("natural", "water"),
                ("waterway", "riverbank"),
                ("landuse", "reservoir"),
            )
        )
    ]
    with gzip.open(DATA / "car_water_spans.json.gz") as file:
        water.extend(Polygon(rings[0], rings[1:]) for rings in json.load(file)["water"])
    return water, STRtree(water)


def pack(region, a_cache, b_cache, scratch):
    (scratch / "region.pbf").symlink_to(region.resolve())
    run(
        "tags-filter",
        region,
        "w/highway=" + ",".join(SPEEDS),
        "-o",
        scratch / "major.osm",
    )
    points, ways = read_roads(scratch / "major.osm")
    reverse, major, candidates, tags = directed_roads(points, ways)
    anchors = origin_nodes(points, candidates)
    water, tree = water_index(region, scratch)
    run("getid", region, f"n{GATE}", "-o", scratch / "gate.osm")
    gate = read_roads(scratch / "gate.osm")[0][GATE]
    origin = a_cache["events"][0]
    move = next(
        e
        for e in b_cache["city_edits"]
        if e["kind"] == "building" and e["op"] == "move" and e["id"] == "mesh:152:317"
    )

    @lru_cache(maxsize=None)
    def classify(a, b, wid):
        road = tags[wid]
        if road.get("tunnel", "no") != "no" or road.get("covered") == "yes":
            return [(0.0, 1.0, "tunnel")]
        if road.get("bridge", "no") != "no":
            return [(0.0, 1.0, "bridge")]
        line = LineString([points[a], points[b]])
        hits = tree.query(line, predicate="intersects")
        intervals = []
        if len(hits):
            wet = line.intersection(
                union_all(
                    [
                        water[i] if water[i].is_valid else water[i].buffer(0)
                        for i in hits
                    ]
                )
            )
            for piece in parts(wet, "LineString"):
                lo, hi = sorted(
                    line.project(Point(p), normalized=True)
                    for p in (piece.coords[0], piece.coords[-1])
                )
                if hi > lo:
                    intervals.append((lo, hi, "water"))
        return sorted(intervals)

    venues = {}
    for name, cache, location in (
        ("monumental", a_cache, "before"),
        ("mut", b_cache, "after"),
    ):
        e = cache["events"][0]
        gate_pos = tuple(
            round(g + d, 7)
            for g, d in zip(
                gate,
                (
                    e["longitude"] - origin["longitude"],
                    e["latitude"] - origin["latitude"],
                ),
            )
        )
        closed = {
            int(ed["id"])
            for ed in cache["city_edits"]
            if ed["kind"] == "road" and ed["op"] == "remove"
        }
        bowl = Polygon(shape(move[location]["geometry"]).exterior)
        stop, walk, walk_m, walk_way, gap = walk_to_gate(
            gate_pos, major, reverse, closed, bowl, scratch
        )
        times, steps = paths_to(reverse, stop, closed, points, bowl)
        routes = {}
        for cut in sorted(e["attendees_by_return_comuna"]):
            n = anchors[cut]
            path = [points[n]]
            spans = []
            while n != stop:
                nxt, wid = steps[n]
                idx = len(path) - 1
                for lo, hi, kind in classify(n, nxt, wid):
                    start, end = round(idx + lo, 7), round(idx + hi, 7)
                    if spans and spans[-1][1] == start and spans[-1][2] == kind:
                        spans[-1][1] = end
                    elif end > start:
                        spans.append([start, end, kind])
                path.append(points[nxt])
                n = nxt
            routes[cut] = {
                "path": path,
                "dropoff": points[stop],
                "seconds": round(times[anchors[cut]], 3),
                "crossings": spans,
                "origin_osm_node": anchors[cut],
            }
        venues[name] = {
            "gate": {
                "osm_node": GATE,
                "source": gate,
                "point": gate_pos,
                "translated": name == "mut",
            },
            "dropoff_osm_node": stop,
            "walk_path": walk,
            "walk_metres": round(walk_m, 2),
            "gate_way": walk_way,
            "gate_axis_gap_m": round(gap, 2),
            "closed_osm_ways": sorted(closed),
            "communes": routes,
        }
    return {
        "attribution": "© OpenStreetMap contributors",
        "license": "ODbL-1.0",
        "source": "https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf",
        "seconds": "Directed free-flow driving time from the comuna anchor to the modeled drop-off.",
        "crossings": "Fractional path vertex intervals [start,end,kind]. Hide tunnel and water. Bridge is an OSM deck.",
        "access": "OSM records a gate on a parking access at Monumental. Matchday admission is unverified. Its MUT position is a scenario translation.",
        "venues": venues,
    }


if __name__ == "__main__":
    with TemporaryDirectory() as directory:
        data = pack(
            Path(sys.argv[1]), result(sys.argv[2]), result(sys.argv[3]), Path(directory)
        )
    dest = PACK / "event_car_routes.json.gz"
    dest.write_bytes(
        gzip.compress(
            json.dumps(data, separators=(",", ":"), ensure_ascii=False).encode(),
            mtime=0,
        )
    )
    print(
        [
            (v, len(r["communes"]), r["walk_metres"], r["gate_axis_gap_m"])
            for v, r in data["venues"].items()
        ],
        dest.stat().st_size,
    )
