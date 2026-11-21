"""Named Santiago places -> INE block (manzana) of the mapped OSM building that contains the place point.

Run from the repo root: python3 tools/pack/named_places.py > data/mvp/named_places.csv
"""
import csv
import gzip
import json
import math
import struct
import sys

# id, name, OSM object, point lon/lat, bbox south/north/west/east.
# Malls and campus: Nominatim centre of the OSM object (2026-11-21). Others: vertex mean of the ring in web/data/santiago_landmarks.geojson.
PLACES = [
    ("mut", "Mercado Urbano Tobalaba (MUT)", "relation/18760408", -70.6011321, -33.4172801, (-33.4177644, -33.416812, -70.6017415, -70.6001859)),
    ("costanera_center", "Costanera Center", "way/1149795722", -70.606751, -33.4180283, (-33.4194688, -33.4165285, -70.6083073, -70.605114)),
    ("gran_torre_costanera", "Gran Torre Costanera", "way/1179710949", None, None, None),
    ("parque_arauco", "Parque Arauco", "relation/21382866", -70.5772908, -33.4023225, (-33.4038446, -33.4004181, -70.5805197, -70.5743567)),
    ("mall_plaza_vespucio", "Mall Plaza Vespucio", "way/25448975", -70.5983734, -33.5175543, (-33.5188934, -33.5162211, -70.6001882, -70.5952659)),
    ("uchile_casa_central", "Casa Central Universidad de Chile", "way/25018684", -70.6509537, -33.4445568, (-33.4448965, -33.4442215, -70.6514443, -70.6504662)),
    ("la_moneda", "Palacio de La Moneda", "relation/394238", None, None, None),
    ("estadio_nacional", "Estadio Nacional", "way/28092600", None, None, None),
    ("movistar_arena", "Movistar Arena (Santander Arena)", "way/32864171", None, None, None),
]

mesh = gzip.decompress(open("web/data/city_building_polygons.bin.gz", "rb").read())
side = gzip.decompress(open("web/data/city_building_block_ids.bin.gz", "rb").read())
_, cols, rows, west, south = struct.unpack_from("<4sHHii", mesh, 0)
tiles = struct.unpack_from(f"<{cols * rows + 1}I", mesh, 28)
_, _, _, nb, ne = struct.unpack_from("<4sHHII", side, 0)
starts = struct.unpack_from(f"<{cols * rows + 1}I", side, 16)
at = 16 + 4 * (cols * rows + 1)
block_ids = [str(struct.unpack_from("<QH", side, at + 10 * i)[0]) for i in range(nb)]
entries = struct.unpack_from(f"<{ne}H", side, at + 10 * nb)
jobs = {r["block_id"]: r for r in csv.DictReader(open("data/mvp/jobs.csv"))}
comunas = {r["cut"]: r["name"] for r in csv.DictReader(open("data/mvp/communes.csv"))}

for f in json.load(open("web/data/santiago_landmarks.geojson"))["features"]:
    for i, p in enumerate(PLACES):
        if p[0] == f["properties"]["id"] and p[3] is None:
            ring = f["geometry"]["coordinates"][0]
            xs, ys = [c[0] for c in ring], [c[1] for c in ring]
            PLACES[i] = p[:3] + (sum(xs) / len(xs), sum(ys) / len(ys), (min(ys), max(ys), min(xs), max(xs)))

mapped = []  # (ring, block id) of each mapped (not generated) building
for t, (s, e) in enumerate(zip(tiles, tiles[1:])):
    if s == e:
        continue
    n, nr, nv = struct.unpack_from("<III", mesh, s)
    p = s + 12
    polys = struct.unpack_from(f"<{n + 1}I", mesh, p); p += 4 * (n + 1)
    rings = struct.unpack_from(f"<{nr + 1}I", mesh, p); p += 4 * (nr + 1)
    pos = struct.unpack_from(f"<{2 * nv}H", mesh, p)
    flags = p + 4 * nv + 2 * n
    for i in range(n):
        k = entries[starts[t] + i]
        if mesh[flags + i] & 4 or k == 65535:
            continue
        a, b = rings[polys[i]], rings[polys[i] + 1]
        mapped.append(([((west + pos[2 * j]) / 1e5, (south + pos[2 * j + 1]) / 1e5) for j in range(a, b)], block_ids[k]))


def inside(x, y, ring):
    c = False
    for (x1, y1), (x2, y2) in zip(ring, ring[1:] + ring[:1]):
        if (y1 > y) != (y2 > y) and x < x1 + (y - y1) * (x2 - x1) / (y2 - y1):
            c = not c
    return c


def metres(lon, lat, row):  # the metric of the #154 rule in engine/src/scenario.rs: longitude x 0.835, 111,320 m per degree
    return math.hypot((float(row["longitude"]) - lon) * 0.835, float(row["latitude"]) - lat) * 111320


out = csv.writer(sys.stdout, lineterminator="\n")
out.writerow(["place_id", "name", "osm", "longitude", "latitude", "block_id", "cut", "comuna", "jobs", "other_blocks_in_bbox", "nearest_jobs_row", "nearest_jobs_row_m", "status"])
for pid, name, osm, lon, lat, (s, n, w, e) in PLACES:
    [block] = {b for r, b in mapped if inside(lon, lat, r)}  # exactly one building contains each point
    others = sorted({b for r, b in mapped if s <= sum(q[1] for q in r) / len(r) <= n and w <= sum(q[0] for q in r) / len(r) <= e} - {block})
    near = min(jobs, key=lambda b: metres(lon, lat, jobs[b]))
    row = jobs[block]
    status = "multi_block" if others else "single_block"
    out.writerow([pid, name, osm, f"{lon:.6f}", f"{lat:.6f}", block, row["cut"], comunas[row["cut"]], row["jobs"], " ".join(others), near, round(metres(lon, lat, jobs[near])), status])
