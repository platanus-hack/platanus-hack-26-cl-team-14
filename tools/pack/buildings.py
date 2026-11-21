"""Build data/mvp/buildings.geojson: 3D buildings around Metro Tobalaba.

Input (download once, see data/mvp/README.md):
  data/raw/overture_buildings/tobalaba.geojson  (Overture buildings, bbox below)
  data/raw/overture_buildings/tobalaba_parts.geojson (Overture building parts)
  data/raw/ine_permits/rm_permits.json          (INE permit points with NUM_PISOS)
  data/raw/exposure_santiago/exposure_santiago.* (residential exposure model, >= 5 stories)
Run: python3 tools/pack/buildings.py
"""
import json
import math
import struct
from collections import defaultdict

W, S, E, N = -70.635, -33.445, -70.565, -33.395
FLOOR_M = 3.5
DEFAULT_M = 6.0  # placeholder height: 2 floors
CELL = 0.002
MIN_DEFAULT_M2 = 55  # drop sheds and kiosks with no height data: keeps the file under 10 MB
KX, KY = 111320 * math.cos(math.radians(-33.42)), 110540

src = json.load(open("data/raw/overture_buildings/tobalaba.geojson"))["features"]
permits = [
    (p["geometry"]["x"], p["geometry"]["y"], p["attributes"]["NUM_PISOS"])
    for p in json.load(open("data/raw/ine_permits/rm_permits.json"))
    if W <= p["geometry"]["x"] <= E and S <= p["geometry"]["y"] <= N and p["attributes"]["NUM_PISOS"]
]


def exposure_points():
    # Shapefile polygons (type 5) + DBF `stories`: one point (vertex mean) per modeled building.
    shp = open("data/raw/exposure_santiago/exposure_santiago.shp", "rb").read()
    dbf = open("data/raw/exposure_santiago/exposure_santiago.dbf", "rb").read()
    n, hl, rl = struct.unpack("<IHH", dbf[4:12])
    pts, i = [], 100
    for k in range(n):
        size = struct.unpack(">i", shp[i + 4:i + 8])[0] * 2
        np_ = struct.unpack("<i", shp[i + 48:i + 52])[0]
        npart = struct.unpack("<i", shp[i + 44:i + 48])[0]
        xy = struct.unpack(f"<{2 * np_}d", shp[i + 52 + 4 * npart:i + 52 + 4 * npart + 16 * np_])
        x, y = sum(xy[0::2]) / np_, sum(xy[1::2]) / np_
        stories = int(float(dbf[hl + k * rl + 1 + 24:hl + k * rl + 1 + 48]))
        if W <= x <= E and S <= y <= N:
            pts.append((x, y, stories))
        i += 8 + size
    return pts


def index(points):
    g = defaultdict(list)
    for x, y, f in points:
        g[(int(x / CELL), int(y / CELL))].append((x, y, f))
    return g


exposure = exposure_points()
permit_grid, exposure_grid = index(permits), index(exposure)


def inside(x, y, ring):
    c = False
    for (x1, y1), (x2, y2) in zip(ring, ring[1:]):
        if (y1 > y) != (y2 > y) and x < (x2 - x1) * (y - y1) / (y2 - y1) + x1:
            c = not c
    return c


def max_floors(grid, rings):
    outer = rings[0]
    xs, ys = [p[0] for p in outer], [p[1] for p in outer]
    best = 0
    for i in range(int(min(xs) / CELL) - 1, int(max(xs) / CELL) + 1):
        for j in range(int(min(ys) / CELL) - 1, int(max(ys) / CELL) + 1):
            for x, y, f in grid.get((i, j), ()):
                if f > best and inside(x, y, outer):
                    best = f
    return best


def area(r):
    return abs(sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(r, r[1:]))) / 2 * KX * KY


def r6(rings):
    return [[[round(x, 6), round(y, 6)] for x, y in ring] for ring in rings]


# OSM puts most tower heights on building:part. Draw the parent, then add each ground-level
# part that is taller than the parent (towers on podiums). Parts with min_height > 0 are skipped.
parts = defaultdict(list)
for f in json.load(open("data/raw/overture_buildings/tobalaba_parts.geojson"))["features"]:
    parts[f["properties"]["building_id"]].append(f)


def height(q, rings):
    if q.get("height"):
        return q["height"], "tag"
    if q.get("num_floors"):
        return q["num_floors"] * FLOOR_M, "levels"
    if fl := max_floors(permit_grid, rings):
        return fl * FLOOR_M, "permit"
    if fl := max_floors(exposure_grid, rings):
        return fl * FLOOR_M, "exposure"
    return DEFAULT_M, "default"


def polygons(g):
    return [g["coordinates"]] if g["type"] == "Polygon" else g["coordinates"]


out = []
for f in src:
    p = f["properties"]
    if p.get("is_underground"):
        continue
    names = p.get("names") or {}
    name = names.get("primary")
    alt = " ".join(r["value"] for r in names.get("rules") or [])
    is_mut = "MUT" in f"{name} {alt}".split() or name == "Mercado Urbano Tobalaba"
    pieces = []
    for rings in polygons(f["geometry"]):
        h, hs = height(p, rings)
        if hs != "default" or area(rings[0]) >= MIN_DEFAULT_M2:
            pieces.append((rings, h, hs))
    top = max((h for _, h, _ in pieces), default=0)
    for x in parts[f["id"]]:
        q = x["properties"]
        if q.get("min_height") or not (q.get("height") or q.get("num_floors")):
            continue
        for rings in polygons(x["geometry"]):
            h, hs = height(q, rings)
            if h > top:
                pieces.append((rings, h, hs))
    for rings, h, hs in pieces:
        props = {"height_m": round(h, 1), "height_source": hs}
        if name:
            props["name"] = name
        if is_mut:
            props["is_mut"] = True
        out.append({"type": "Feature", "properties": props,
                    "geometry": {"type": "Polygon", "coordinates": r6(rings)}})

json.dump({"type": "FeatureCollection", "features": out},
          open("data/mvp/buildings.geojson", "w"), separators=(",", ":"), ensure_ascii=False)
print(len(out), "buildings,", len(permits), "permit points,", len(exposure), "exposure buildings in bbox")
