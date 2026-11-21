"""Pack OSM footprints and census-derived buildings into two tiled city layers."""
import csv
import gzip
import json
import math
import struct
import sys
from array import array
from collections import Counter
from dataclasses import dataclass, field
from itertools import chain
from pathlib import Path

from city_footprints import numeric

RAW = Path("data/raw/building-fill")
SOURCE = Path("data/raw/osm-buildings-city/polygons.geojsonseq")
OUT = Path("web/data")
WEST, SOUTH, EAST, NORTH = -7085000, -3365000, -7045000, -3330000
STEP, COLS, ROWS = 2000, 20, 18
X_METRES = 1.11320 * math.cos(math.radians(-33.475))
Y_METRES = 1.10540
TAGGED, LEVELS, CENSUS, UNASSIGNED = range(4)
GENERATED = 4


def quantize(ring):
    points = []
    for lon, lat in ring[:-1]:
        point = (round(lon * 100000) - WEST, round(lat * 100000) - SOUTH)
        if not points or point != points[-1]:
            points.append(point)
    if points and points[0] == points[-1]:
        points.pop()
    area = sum(ax * by - bx * ay for (ax, ay), (bx, by) in zip(points, points[1:] + points[:1]))
    return points if area else []


def lonlat(ring):
    return [[(x + WEST) / 100000, (y + SOUTH) / 100000] for x, y in ring + ring[:1]]


def prepare():
    """Use the same rounded polygons for the census intersection and the final mesh."""
    RAW.mkdir(parents=True, exist_ok=True)
    records = []
    with SOURCE.open() as src:
        for row in src:
            feature = json.loads(row.lstrip("\x1e"))
            tags = feature["properties"]
            if tags.get("building", "no") == "no":
                continue
            for part, poly in enumerate(feature["geometry"]["coordinates"]):
                if not all(WEST / 1e5 <= x <= EAST / 1e5 and SOUTH / 1e5 <= y <= NORTH / 1e5 for x, y in poly[0]):
                    continue
                shell, *holes = map(quantize, poly)
                if not shell:
                    continue
                geom = {"type": "Polygon", "coordinates": [lonlat(ring) for ring in [shell, *holes] if ring]}
                props = {"osm_id": feature["id"], "part": part, "height_m": numeric(tags.get("height", ""), units=True),
                         "levels": numeric(tags.get("building:levels", ""))}
                record = json.dumps({"type": "Feature", "geometry": geom, "properties": props}, separators=(",", ":"))
                records.append((int(feature["id"][1:]), part, record))
    # Osmium can emit completed relations between ways. The wire order uses explicit IDs.
    with (RAW / "real_rendered.geojsonseq").open("w") as dst:
        for _, _, record in sorted(records):
            dst.write(record + "\n")
    print(f"Prepared {len(records):,} OSM components")


def cross(o, a, b):
    return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])


def hull_half(points):
    hull = []
    for p in points:
        while len(hull) >= 2 and cross(hull[-2], hull[-1], p) <= 0:
            hull.pop()
        hull.append(p)
    return hull[:-1]


def oriented_box(shell):
    """Find the minimum-area rectangle among the axes of the footprint's convex hull."""
    points = sorted(set(shell))
    hull = hull_half(points) + hull_half(reversed(points))
    hull = [(x * X_METRES, y * Y_METRES) for x, y in hull]
    best = math.inf
    for (ax, ay), (bx, by) in zip(hull, hull[1:] + hull[:1]):
        angle = math.atan2(by - ay, bx - ax)
        cos, sin = math.cos(angle), math.sin(angle)
        u = [x * cos + y * sin for x, y in hull]
        v = [y * cos - x * sin for x, y in hull]
        u0, u1, v0, v1 = min(u), max(u), min(v), max(v)
        width, depth = u1 - u0, v1 - v0
        if width * depth >= best:
            continue
        best = width * depth
        u_mid, v_mid = (u0 + u1) / 2, (v0 + v1) / 2
        cx = round((u_mid * cos - v_mid * sin) / X_METRES)
        cy = round((u_mid * sin + v_mid * cos) / Y_METRES)
        if width < depth:
            width, depth, angle = depth, width, angle + math.pi / 2
        box = (cx, cy, max(1, round(width * 10)), max(1, round(depth * 10)),
               round(math.degrees(angle) * 100) % 18000)
    return box


def u16():
    return array("H")


@dataclass
class Tile:
    polygon_offsets: array = field(default_factory=lambda: array("I", [0]))
    ring_offsets: array = field(default_factory=lambda: array("I", [0]))
    positions: array = field(default_factory=u16)
    heights: array = field(default_factory=u16)
    flags: array = field(default_factory=lambda: array("B"))
    boxes: array = field(default_factory=u16)

    def add(self, rings, height, flags, box):
        for ring in rings:
            self.positions.extend(chain.from_iterable(ring))
            self.ring_offsets.append(len(self.positions) // 2)
        self.polygon_offsets.append(len(self.ring_offsets) - 1)
        self.heights.append(height)
        self.flags.append(flags)
        self.boxes.extend((*box, height))


def little_endian(values):
    if sys.byteorder == "little":
        return values.tobytes()
    copy = values[:]
    copy.byteswap()
    return copy.tobytes()


def binary(tiles, boxes):
    magic = b"SGBX" if boxes else b"SGB1"
    blob = bytearray(struct.pack("<4sHH4iI", magic, COLS, ROWS, WEST, SOUTH, EAST, NORTH, STEP))
    directory = len(blob)
    blob.extend(bytes(4 * (len(tiles) + 1)))
    for index, tile in enumerate(tiles):
        struct.pack_into("<I", blob, directory + 4 * index, len(blob))
        n = len(tile.flags)
        if not n:
            continue
        if boxes:
            blob.extend(struct.pack("<I", n))
            fields = (tile.boxes, tile.flags)
        else:
            blob.extend(struct.pack("<III", n, len(tile.ring_offsets) - 1, len(tile.positions) // 2))
            fields = (tile.polygon_offsets, tile.ring_offsets, tile.positions, tile.heights, tile.flags)
        for values in fields:
            blob.extend(little_endian(values))
        blob.extend(bytes(-len(blob) % 4))
    struct.pack_into("<I", blob, directory + 4 * len(tiles), len(blob))
    return blob


def real_features():
    with (RAW / "real_blocks.csv").open() as f:
        density = {(r["osm_id"], int(r["part"])): float(r["density_height_m"])
                   for r in csv.DictReader(f) if r["block_id"]}
    with (RAW / "real_rendered.geojsonseq").open() as f:
        for row in f:
            feature = json.loads(row)
            p = feature["properties"]
            if p["height_m"]:
                height, flag = p["height_m"], TAGGED
            elif p["levels"]:
                height, flag = 3.5 * p["levels"], LEVELS
            elif (key := (p["osm_id"], p["part"])) in density:
                height, flag = density[key], CENSUS
            else:
                height, flag = 3, UNASSIGNED
            yield feature["geometry"]["coordinates"], height, flag


def generated_features():
    with (RAW / "generated.geojsonseq").open() as f:
        for row in f:
            feature = json.loads(row.lstrip("\x1e"))
            yield feature["geometry"]["coordinates"], feature["properties"]["height_m"], GENERATED | CENSUS


def pack():
    tiles = [Tile() for _ in range(COLS * ROWS)]
    counts = Counter()
    for polygon, height, flag in chain(real_features(), generated_features()):
        shell, *holes = map(quantize, polygon)
        box = oriented_box(shell)
        x, y = box[:2]
        col = min(COLS - 1, max(0, x // STEP))
        row = min(ROWS - 1, max(0, y // STEP))
        tiles[row * COLS + col].add([shell, *filter(None, holes)], round(height * 10), flag, box)
        counts[flag] += 1
    for name, boxes in (("city_building_polygons.bin.gz", False), ("city_building_boxes.bin.gz", True)):
        raw = binary(tiles, boxes)
        file = OUT / name
        with file.open("wb") as f, gzip.GzipFile(filename="", mode="wb", fileobj=f, compresslevel=6, mtime=0) as gz:
            gz.write(raw)
        print(f"{name}: {len(raw):,} raw bytes, {file.stat().st_size:,} gzip bytes")
    print(f"Footprints per source flag: {dict(sorted(counts.items()))}")


if __name__ == "__main__":
    {"prepare": prepare, "pack": pack}[sys.argv[1]]()
