"""Map packed car route segments to OSM inland water, motor bridges and tunnels.

uv run --with shapely==2.2.0 python -B tools/pack/car_water_spans.py /path/to/urban.osm.pbf
Requires osmium. Run at the repository root. Coordinates in the pack are WGS84.
"""

import gzip
import hashlib
import json
import math
import subprocess
import sys
from functools import lru_cache
from pathlib import Path
from tempfile import TemporaryDirectory

from shapely import STRtree, make_valid, transform, union_all
from shapely.geometry import LineString, box, shape

from roads import VIEW

DATA = Path("web/data")
PBF_SHA = "4a1cfdb3ffdfdb58cbc6247ab36cac38ad2dc35509b5c1e102a7e53461234cb3"
# Linear scale used by the native view. Route spans below use fractional vertex indices.
X, Y = 92915.0, 110905.0


def metres(x, y, z=None):
    return (x + 70.65) * X, (y + 33.45) * Y


def read(name):
    file = DATA / name
    with gzip.open(file) if name.endswith(".gz") else file.open() as stream:
        return json.load(stream)


def parts(geom, kind):
    if geom.geom_type == kind and not geom.is_empty:
        yield geom
    elif hasattr(geom, "geoms"):
        for child in geom.geoms:
            yield from parts(child, kind)


def sha(file):
    return hashlib.sha256(file.read_bytes()).hexdigest()


def water_polygons(pbf):
    with TemporaryDirectory() as tmp:
        extract = Path(tmp) / "water.pbf"
        subprocess.run(
            [
                "osmium",
                "tags-filter",
                pbf,
                "wr/natural=water",
                "wr/waterway=riverbank",
                "wr/landuse=reservoir",
                "-o",
                extract,
            ],
            check=True,
        )
        raw = subprocess.check_output(
            [
                "osmium",
                "export",
                "-u",
                "type_id",
                "--geometry-types",
                "polygon",
                "-f",
                "geojson",
                extract,
            ]
        )
    # The viewer draws these two river relations from its own export. Use those exact banks.
    detail = {
        f["properties"]["osm_id"]: f["geometry"]
        for f in read("mapocho_detail.geojson")["features"]
        if f["properties"]["kind"] == "water"
    }
    features = dict(detail)
    for f in json.loads(raw)["features"]:
        tags = f["properties"]
        # osmium includes dependent ways. An island, detention basin or wood can be such a way.
        if not (
            tags.get("natural") == "water"
            or tags.get("waterway") == "riverbank"
            or tags.get("landuse") == "reservoir"
        ):
            continue
        area_id = int(f["id"][1:])
        osm_id = ("r" if area_id % 2 else "w") + str(area_id // 2)
        features.setdefault(osm_id, f["geometry"])
    bounds = box(*VIEW)
    water, ids = [], []
    for osm_id, geom in sorted(features.items()):
        for polygon in parts(make_valid(shape(geom)).intersection(bounds), "Polygon"):
            ids.append(osm_id)
            water.append(
                [list(r.coords) for r in [polygon.exterior, *polygon.interiors]]
            )
    return water, ids


class CrossingIndex:
    def __init__(self, water):
        self.water = [
            transform(
                shape({"type": "Polygon", "coordinates": p}), metres, interleaved=False
            )
            for p in water
        ]
        self.water_tree = STRtree(self.water)
        self.decks = []
        structures, self.tags = [], []
        for road in read("road_centerlines.geojson.gz")["features"]:
            props = road["properties"]
            coords = road["geometry"]["coordinates"]
            line = LineString([metres(*p) for p in coords])
            width = props["width_m"]
            if not len(
                self.water_tree.query(
                    line, predicate="dwithin", distance=width / 2 + 5.0
                )
            ):
                continue
            kind = props["structure"]
            if kind == "bridge":
                self.decks.append([props["osm_way_id"], width, coords])
            elif props["layer"] < 0:
                # Negative layer is relative order. Use it only on the part of
                # this non-bridge road underneath mapped water.
                kind = "tunnel"
            # A parallel mapped surface carriageway makes a tunnel match ambiguous.
            for a, b in zip(line.coords, list(line.coords)[1:]):
                length = math.dist(a, b)
                if not length:
                    continue
                road_dir = ((b[0] - a[0]) / length, (b[1] - a[1]) / length)
                segment = LineString([a, b])
                # 3 m packed OSRM simplification plus five-decimal rounding.
                structures.append(segment.buffer(width / 2 + 3.8, quad_segs=3))
                self.tags.append((kind, road_dir))
        self.structures = structures
        self.roads = STRtree(structures)

    @lru_cache(maxsize=None)
    def segment(self, a, b):
        a, b = metres(*a), metres(*b)
        dx, dy = b[0] - a[0], b[1] - a[1]
        length_sq = dx * dx + dy * dy
        if not length_sq:
            return ()
        line = LineString([a, b])
        waters = self.water_tree.query(line, predicate="intersects")
        if not len(waters):
            return ()
        wet = line.intersection(union_all([self.water[i] for i in waters]))
        ranges = []

        def fraction(p):
            return max(
                0.0, min(1.0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length_sq)
            )

        for wetted in parts(wet, "LineString"):
            intervals = []
            start, end = sorted(
                fraction(p) for p in (wetted.coords[0], wetted.coords[-1])
            )
            for i in self.roads.query(wetted, predicate="intersects"):
                kind, (rx, ry) = self.tags[i]
                # A tunnel under a perpendicular bridge is a different road.
                if abs(dx * rx + dy * ry) < math.sqrt(length_sq) * 0.82:
                    continue
                for piece in parts(
                    wetted.intersection(self.structures[i]), "LineString"
                ):
                    lo, hi = sorted(
                        fraction(p) for p in (piece.coords[0], piece.coords[-1])
                    )
                    intervals.append((lo, hi, kind))
            limits = sorted(
                {start, end, *(v for lo, hi, _ in intervals for v in (lo, hi))}
            )
            for lo, hi in zip(limits, limits[1:]):
                tags = {kind for s, e, kind in intervals if s < (lo + hi) / 2 < e}
                label = (
                    "bridge"
                    if "bridge" in tags
                    else "tunnel"
                    if tags == {"tunnel"}
                    else "water"
                )
                ranges.append((lo, hi, label))
        return tuple(sorted(ranges))

    def routes(self, rows):
        out = {}
        for key, path in rows:
            spans = []
            for vertex, (a, b) in enumerate(zip(path, path[1:])):
                for lo, hi, kind in self.segment(tuple(a), tuple(b)):
                    start, end = round(vertex + lo, 7), round(vertex + hi, 7)
                    if start == end:
                        continue
                    if spans and spans[-1][1] == start and spans[-1][2] == kind:
                        spans[-1][1] = end
                    else:
                        spans.append([start, end, kind])
            if spans:
                out[str(key)] = spans
        return out


def main():
    pbf = Path(sys.argv[1])
    digest = sha(pbf)
    if digest != PBF_SHA:
        raise ValueError("Use the same OSM extract as the native road centerlines")
    water, ids = water_polygons(pbf)
    index = CrossingIndex(water)
    local = read("local_mover_routes.json.gz")
    selected = sorted(
        {
            g["route"]
            for g in local["groups"]
            if g["mode"] == "car" and g.get("route") is not None
        }
    )
    bundle = {
        "attribution": "© OpenStreetMap contributors",
        "license": "ODbL-1.0",
        "pbf_sha256": digest,
        "water": water,
        "water_ids": ids,
        "bridges": index.decks,
        "local": index.routes((i, local["paths"][i]) for i in selected),
        "od": index.routes(read("car_routes.json").items()),
    }
    raw = json.dumps(bundle, separators=(",", ":"), ensure_ascii=False).encode()
    dest = DATA / "car_water_spans.json.gz"
    dest.write_bytes(gzip.compress(raw, compresslevel=9, mtime=0))
    print(
        "polygons",
        len(water),
        "holes",
        sum(len(p) - 1 for p in water),
        "bridges",
        len(index.decks),
        "routes with water",
        len(bundle["local"]),
        len(bundle["od"]),
        "bytes",
        dest.stat().st_size,
    )


if __name__ == "__main__":
    main()
