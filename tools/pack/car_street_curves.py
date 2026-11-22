"""Restore source vertices where a published comuna car route cuts through a building.

Use the regional santiago.osm used for car_paths.py (data/mvp/README.md).
Pass its path and the port of the original local OSRM car graph.
Run from the repository root with NumPy and Shapely 2.
"""
import csv
import gzip
import json
import struct
import sys
from urllib.request import urlopen
from collections import defaultdict
from pathlib import Path

import numpy as np
import shapely
from shapely import STRtree
from shapely.geometry import shape

from car_paths import car_graph, route
from roads import PACK, distance, read_roads, rounded_path, simplify

DATA = Path("web/data")


def building_crossings(segments):
    chords = shapely.linestrings(np.asarray(segments))
    tree = STRtree(chords)
    raw = gzip.decompress((DATA / "city_building_polygons.bin.gz").read_bytes())
    nx, ny, x, y = struct.unpack_from("<HHii", raw, 4)
    tiles = np.frombuffer(raw, "<u4", nx * ny + 1, 28)
    witnesses, owners = [], []
    for first, last in zip(tiles, tiles[1:]):
        if first == last:
            continue
        pos = int(first)
        buildings, rings, vertices = struct.unpack_from("<III", raw, pos)
        pr = np.frombuffer(raw, "<u4", buildings + 1, pos + 12)
        rv = np.frombuffer(raw, "<u4", rings + 1, pos + 12 + pr.nbytes)
        xy = np.frombuffer(raw, "<u2", vertices * 2, pos + 12 + pr.nbytes + rv.nbytes)
        polygons = shapely.from_ragged_array(shapely.GeometryType.POLYGON,
            (xy.reshape(-1, 2) + [x, y]) / 1e5, (rv.astype(int), pr.astype(int)))
        pids, sids = tree.query(polygons, predicate="intersects")
        for pid, sid in zip(pids, sids):
            footprint = polygons[pid]
            if not footprint.is_valid:
                footprint = shapely.make_valid(footprint)
            cut = chords[sid].intersection(footprint)
            for part in shapely.get_parts(cut):
                if part.geom_type != "LineString" or part.length < 0.00001:
                    continue
                for fraction in (0.25, 0.5, 0.75):
                    point = part.interpolate(fraction, normalized=True)
                    if footprint.contains(point):
                        witnesses.append(point)
                        owners.append(sid)
    with gzip.open(DATA / "road_surface.geojson.gz") as f:
        surface = STRtree([shape(row["geometry"]) for row in json.load(f)["features"]
                           if row["properties"]["use"] == "road"])
    on_road = set(surface.query(np.asarray(witnesses, dtype=object), predicate="intersects")[0])
    return {segments[sid] for i, sid in enumerate(owners) if i not in on_road}


def restore(routes, damaged, source):
    points, ways = read_roads(source)
    graph, candidates = car_graph(points, ways)
    candidates = sorted(candidates)
    anchors = {}
    with (PACK / "communes.csv").open() as f:
        for row in csv.DictReader(f):
            pos = (float(row["longitude"]), float(row["latitude"]))
            anchors[row["cut"]] = min(candidates, key=lambda n: distance(pos, points[n]))
    by_origin = defaultdict(list)
    for pair, coords in routes.items():
        if damaged.intersection(zip(coords, coords[1:])):
            origin, dest = pair.split("-")
            by_origin[origin].append((dest, coords))
    result = {}
    for origin, destinations in by_origin.items():
        start = anchors[origin]
        previous, _, _ = route(graph, start, [anchors[d] for d, _ in destinations])
        for dest, packed in destinations:
            nodes = [anchors[dest]]
            while nodes[-1] != start:
                nodes.append(previous[nodes[-1]])
            full = [points[n] for n in reversed(nodes)]
            retained = [tuple(round(v, 5) for v in p) for p in full]
            index = 0
            for a, b in zip(packed, packed[1:]):
                end = retained.index(b, index + 1)
                if (a, b) in damaged and (a, b) not in result:
                    original = full[index:end + 1]
                    # Only restore the original graph chain removed by the 20 m simplifier.
                    if len(original) > 2 and len(simplify(original, 20)) == 2:
                        result[a, b] = original[1:-1]
                index = end
    return result


def local_curves(port):
    with gzip.open(DATA / "local_mover_routes.json.gz") as f:
        local = json.load(f)
    groups = {g["route"]: g for g in local["groups"] if g["mode"] == "car"}
    paths = {i: list(map(tuple, local["paths"][i])) for i in groups}
    damaged = building_crossings(sorted({ab for path in paths.values() for ab in zip(path, path[1:])}))
    curves = {}
    for idx, path in paths.items():
        if not damaged.intersection(zip(path, path[1:])):
            continue
        row = groups[idx]
        coords = ";".join(f"{lon},{lat}" for lon, lat in (row["origin"], row["destination"]))
        with urlopen(f"http://127.0.0.1:{port}/route/v1/driving/{coords}?geometries=geojson&overview=full&radiuses=250;250") as response:
            original = json.load(response)["routes"][0]["geometry"]["coordinates"]
        if rounded_path(original, 3) != local["paths"][idx]:
            raise ValueError(f"OSRM route {idx} differs from the published route")
        rounded = [tuple(round(n, 5) for n in p) for p in original]
        start = 0
        for a, b in zip(path, path[1:]):
            end = rounded.index(b, start + 1)
            if (a, b) in damaged:
                curves[a, b] = original[start + 1:end]
            start = end
    return curves


if __name__ == "__main__":
    routes = {k: list(map(tuple, v)) for k, v in json.loads((DATA / "car_routes.json").read_text()).items()}
    segments = sorted({ab for line in routes.values() for ab in zip(line, line[1:])})
    damaged = building_crossings(segments)
    curves = restore(routes, damaged, Path(sys.argv[1]))
    print(f"{len(curves)} restored of {len(damaged)} comuna crossings")
    curves.update(local_curves(sys.argv[2]))
    rows = [[a, *curves[a, b], b] for a, b in sorted(curves)]
    packed = json.dumps(rows, separators=(",", ":")).encode()
    (DATA / "car_street_curves.json.gz").write_bytes(gzip.compress(packed, mtime=0))
    print(f"{len(curves)} restored car curves")
