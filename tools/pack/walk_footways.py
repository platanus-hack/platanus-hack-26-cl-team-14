"""Identify local OSRM walk segments that already follow a mapped pedestrian way.

Input: osmium export --geometry-types=linestring -f geojsonseq of the urban OSM
highways used by the street pack. Run from the repository root with that file
as the argument. Road segments keep the viewer's existing lateral placement.
"""
import gzip
import json
import math
import sys
import struct
from collections import defaultdict
from itertools import pairwise

import numpy as np
import shapely
from shapely.affinity import affine_transform
from shapely.ops import unary_union
from pathlib import Path

from shapely.geometry import LineString, Point, box
from shapely.strtree import STRtree

DATA = Path("web/data")
FOOT = {"footway", "pedestrian", "path", "steps", "corridor", "bridleway"}


def metres(point):
    lon, lat = point
    return ((lon + 70.65) * 92_915, -(lat + 33.45) * 110_905)


def on_footway(a, b, tree, ways, foot):
    dx, dz = b[0] - a[0], b[1] - a[1]
    length = math.hypot(dx, dz)
    if not length:
        return False
    # OSRM geometry was simplified by 3 m, then rounded to 1e-5 degrees.
    radius = 3.8
    candidates = tree.query(box(min(a[0], b[0]) - radius, min(a[1], b[1]) - radius,
                                max(a[0], b[0]) + radius, max(a[1], b[1]) + radius))
    if not any(foot[i] for i in candidates):
        return False
    votes = 0
    for fraction in (0.15, 0.5, 0.85):
        sample = Point(a[0] + dx * fraction, a[1] + dz * fraction)
        nearest = [math.inf, math.inf]
        for i in candidates:
            way = ways[i]
            distance = way.distance(sample)
            if distance > radius or distance >= nearest[foot[i]]:
                continue
            s = way.project(sample)
            p = way.interpolate(max(0, s - 1))
            q = way.interpolate(min(way.length, s + 1))
            span = p.distance(q)
            if span and abs(dx * (q.x - p.x) + dz * (q.y - p.y)) >= 0.8 * length * span:
                nearest[foot[i]] = distance
        # Keep the previous placement when a parallel motor way fits equally well.
        votes += nearest[True] + 0.3 < nearest[False]
    return votes >= 2


def foot_network():
    ways, foot = [], []
    with open(sys.argv[1]) as features:
        for line in features:
            feature = json.loads(line.lstrip("\x1e"))
            tags = feature["properties"]
            if "highway" not in tags or tags.get("area") == "yes":
                continue
            ways.append(LineString(map(metres, feature["geometry"]["coordinates"])))
            foot.append(tags["highway"] in FOOT)
    return STRtree(ways), ways, foot


def ground_heights(points, grid):
    # Same two terrain triangles as viz/src/city/terrain.rs. The native 3D
    # tangent determines both the horizontal offset and its transition length.
    col = np.clip((points[:, 0] + 18583) * 1200 / 92915, 0, 480)
    row = np.clip((points[:, 1] + 16635.75) * 1200 / 110905, 0, 420)
    i, j = np.minimum(col.astype(int), 479), np.minimum(row.astype(int), 419)
    u, v = col - i, row - j
    a, b, c, d = grid[j, i], grid[j, i+1], grid[j+1, i], grid[j+1, i+1]
    return np.where(u+v <= 1, a+u*(b-a)+v*(c-a), d+(1-u)*(c-d)+(1-v)*(b-d))


def building_tiles():
    # SGB1 exterior and interior rings share each tile's vertex array.
    data = gzip.decompress((DATA / "city_building_polygons.bin.gz").read_bytes())
    _, cols, rows, west, south = struct.unpack_from("<4sHHii", data)
    offsets = np.frombuffer(data, "<u4", cols*rows + 1, 28)
    for start, stop in pairwise(offsets):
        if start == stop:
            continue
        count, rings, vertices = struct.unpack_from("<III", data, start)
        at = int(start) + 12
        polygons = np.frombuffer(data, "<u4", count+1, at).astype("i4")
        at += polygons.nbytes
        ends = np.frombuffer(data, "<u4", rings+1, at).astype("i4")
        at += ends.nbytes
        xy = np.frombuffer(data, "<u2", vertices*2, at).reshape(-1, 2).astype(float)
        coords = ((xy + [west, south]) / 1e5 + [70.65, 33.45]) * [92915, -110905]
        yield shapely.from_ragged_array(shapely.GeometryType.POLYGON, coords, (ends, polygons))


def parts(geometry):
    if geometry.geom_type == "LineString":
        if geometry.length > 0:
            yield geometry
    else:
        for piece in getattr(geometry, "geoms", []):
            yield from parts(piece)


def on_axis(geometry):
    return unary_union([LineString([(line.bounds[0], 0), (line.bounds[2], 0)])
                        for line in parts(geometry)])


def safe_spans(length, lane, buildings):
    # In segment coordinates x measures progress and y the lateral offset.
    prior = LineString([(0, lane), (length, lane)])
    old_hits = on_axis(prior.intersection(buildings))
    if old_hits.is_empty:
        return
    axis = LineString([(0, 0), (length, 0)])
    for clear in parts(axis.difference(buildings.buffer(0.01))):
        lo, _, hi, _ = clear.bounds
        if clear.intersection(old_hits).length < 0.2:
            continue
        # Ease between the old lane and the route. Shorten an unsafe transition
        # until it adds no overlap at a previously clear distance.
        for entering in (True, False):
            for _ in range(24):
                if hi - lo < 2*lane:
                    break
                ends = [(lo, lane), (lo+lane, 0)] if entering else [(hi-lane, 0), (hi, lane)]
                hits = on_axis(LineString(ends).intersection(buildings))
                if hits.difference(old_hits).length < 0.005:
                    break
                if entering:
                    lo += 0.5
                else:
                    hi -= 0.5
        if hi - lo < 2*lane:
            continue
        corrected = LineString([(lo, lane), (lo+lane, 0), (hi-lane, 0), (hi, lane)])
        new_hits = on_axis(corrected.intersection(buildings))
        gain = LineString([(lo, 0), (hi, 0)]).intersection(old_hits).length - new_hits.length
        if gain > 0.2 and new_hits.difference(old_hits).length <= 0.005:
            yield round(lo / length, 7), round(hi / length, 7)


def main():
    network = foot_network()
    with gzip.open(DATA / "local_mover_routes.json.gz") as source:
        pack = json.load(source)
    grid = np.fromfile(DATA / "terrain.i16", dtype="<i2").reshape(421, 481)
    segments, geometry = [], []
    for route in sorted({g["route"] for g in pack["groups"] if g["mode"] == "walk"}):
        points = np.array(list(map(metres, pack["paths"][route])))
        heights = ground_heights(points, grid)
        for i, (a, b) in enumerate(pairwise(points)):
            if on_footway(a, b, *network):
                length = np.linalg.norm(b-a)
                lane = 3.4 * length / math.hypot(length, heights[i+1]-heights[i])
                segments.append((route, i, a, (b-a)/length, length, lane))
                geometry.append(LineString([a, b]))

    # Index only the pedestrian segments. Load one tile of building polygons at a time.
    tree = STRtree(geometry)
    nearby = defaultdict(list)
    for tile in building_tiles():
        indices, lines = tree.query(tile, predicate="dwithin", distance=4)
        for j, idx in zip(indices, lines):
            polygon = tile[j]
            nearby[idx].append(polygon if polygon.is_valid else shapely.make_valid(polygon))

    spans = defaultdict(list)
    for idx, footprints in sorted(nearby.items()):
        route, i, start, direction, length, lane = segments[idx]
        right = np.array([-direction[1], direction[0]])
        matrix = [*direction, *right, -start.dot(direction), -start.dot(right)]
        buildings = affine_transform(unary_union(footprints), matrix)
        for lo, hi in safe_spans(length, lane, buildings):
            spans[route].append([i, lo, hi])
    (DATA / "walk_footways.json").write_text(json.dumps(spans, separators=(",", ":")) + "\n")


if __name__ == "__main__":
    main()
