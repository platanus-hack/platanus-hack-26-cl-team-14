"""Find dry parts of car routes inside a mapped tunnel. Distances use the native view's metres."""
import math
import subprocess
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from functools import lru_cache

from shapely import LineString, STRtree, union_all
from shapely.geometry import Point

from road_surface import source_way


def with_covered_roads(roads, pbf):
    with TemporaryDirectory() as tmp:
        covered = Path(tmp) / "covered.pbf"
        subprocess.run(["osmium", "tags-filter", pbf, "w/covered=yes", "-o", covered], check=True)
        found = json.loads(subprocess.check_output([
            "osmium", "export", "-u", "type_id", "--geometry-types", "linestring",
            "-f", "geojson", covered]))["features"]
    motor = {f["properties"]["osm_way_id"] for f in roads}
    ids = {int(f["id"][1:]) for f in found}
    for row in roads:
        if row["properties"]["osm_way_id"] in ids:
            row["properties"]["structure"] = "tunnel"
    for row in found:
        tags = row["properties"]
        wid = int(row["id"][1:])
        kind = source_way(tags)
        access = next((tags[k] for k in ("motorcar", "motor_vehicle", "vehicle", "access") if k in tags), None)
        # OSRM can use explicitly accessible pedestrian passages. Keep a roof
        # off the street layer unless that passage is actually covered.
        if wid not in motor and kind and kind[0] == "pedestrian" and access in {"yes", "permissive", "designated"}:
            row["properties"] = {"structure": "tunnel", "width_m": kind[4]}
            roads.append(row)
    return roads


def intervals(geometry, line):
    if geometry.is_empty:
        return
    if geometry.geom_type == "LineString":
        yield sorted(line.project(Point(coord), normalized=True) for coord in (geometry.coords[0], geometry.coords[-1]))
    elif hasattr(geometry, "geoms"):
        for piece in geometry.geoms:
            yield from intervals(piece, line)


class DryTunnels:
    def __init__(self, roads, metres):
        self.metres = metres
        tunnels = [LineString([metres(*p) for p in f["geometry"]["coordinates"]])
                   for f in roads if f["properties"]["structure"] == "tunnel"]
        near = STRtree(tunnels)
        shapes, self.directions, self.tunnel = [], [], []
        for road in roads:
            props = road["properties"]
            points = [metres(*p) for p in road["geometry"]["coordinates"]]
            width = props["width_m"] / 2
            if not len(near.query(LineString(points), predicate="dwithin", distance=width + 12)):
                continue
            is_tunnel = props["structure"] == "tunnel"
            for a, b in zip(points, points[1:]):
                length = math.dist(a, b)
                if length == 0:
                    continue
                shapes.append(LineString((a, b)).buffer(width + 0.75, cap_style="flat"))
                self.directions.append(((b[0] - a[0]) / length, (b[1] - a[1]) / length))
                self.tunnel.append(is_tunnel)
        self.shapes = shapes
        self.tree = STRtree(shapes)

    @lru_cache(maxsize=None)
    def segment(self, a, b):
        a, b = self.metres(*a), self.metres(*b)
        line = LineString((a, b))
        length = line.length
        if length == 0:
            return ()
        ground, tunnel = [], []
        for i in self.tree.query(line, predicate="intersects"):
            x, y = self.directions[i]
            if abs((b[0] - a[0]) * x + (b[1] - a[1]) * y) < 0.9 * length:
                continue
            (tunnel if self.tunnel[i] else ground).append(self.shapes[i])
        if not tunnel:
            return ()
        # When a parallel surface road fits the chord too, its level is unknown.
        underground = line.intersection(union_all(tunnel))
        if ground:
            underground = underground.difference(union_all(ground))
        return tuple(sorted(intervals(underground, line)))

    def add(self, rows, water):
        for key, path in rows:
            key = str(key)
            existing = water.get(key, [])
            dry = []
            for i, (a, b) in enumerate(zip(path, path[1:])):
                for s, e in self.segment(tuple(a), tuple(b)):
                    lo, hi = round(i + s, 7), round(i + e, 7)
                    # Keep every river/bridge classification from the water pack.
                    for start, end, _ in existing:
                        if end <= lo:
                            continue
                        if start >= hi:
                            break
                        if lo < start:
                            dry.append([lo, start, "tunnel"])
                        lo = max(lo, end)
                    if lo < hi:
                        dry.append([lo, hi, "tunnel"])
            if dry:
                merged = []
                for start, end, kind in sorted(existing + dry):
                    if merged and merged[-1][1] == start and merged[-1][2] == kind:
                        merged[-1][1] = end
                    else:
                        merged.append([start, end, kind])
                water[key] = merged
