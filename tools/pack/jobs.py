"""Build jobs.csv: jobs for each Census 2024 manzana of the RM.

Run: python3 -I tools/pack/jobs.py [PACK_DIR]

Comuna totals: Census 2024 work matrix, column sums by work_comuna.
Spread inside the comuna:
  1. EOD 2012 workers at their usual workplace point (expanded with Factor).
     A point shared by many workers is a landmark (a Metro station): it spreads over the signal around it.
  2. INE non-residential building permits 2013-2025: floor area / M2_PER_JOB.
  3. Comuna with a thin EOD sample: resident persons of each manzana.
"""
import collections, csv, io, json, math, sqlite3, struct, sys, tarfile

PACK = sys.argv[1] if len(sys.argv) > 1 else "data/mvp"  # output folder

RAW = "data/raw/"
M2_PER_JOB = 40  # assumption: gross permit area (with parking) for one job
SNAP_M = 100     # a point on a street goes to the nearest manzana edge within this distance
CELL = 0.002     # index cell, degrees (about 200 m)
LANDMARK_N = 10  # sampled workers on one identical point
LANDMARK_M = 500 # a landmark spreads over the manzanas with exact-address workers within this distance
MIN_SAMPLE = 50  # a comuna with fewer placed sampled workers uses resident persons
OFFICE_M2 = 10   # usable m2 for one office worker: OGUC art. 4.2.4 occupant load; the MUT claim (4,000 people, 40,000 m2) gives the same
MALL_M2 = 46     # leasable m2 for one mall job: 4.61 million m2 GLA (CCC index, Q3 2024) / about 100,000 jobs (CNC study, 2021)
# Two sites of the demo get jobs from published floor area, not from the 2012 signal. Sources: data/mvp/README.md.
# Rows that are probably too high (README): one small 2012 sample on a residential block, and 2 large permits / 40 m2.
KNOWN_HIGH = {"13120111005004", "13108071002002", "13301031001050"}
AIRPORT = "13124_airport"  # block_id of the airport row: not a manzana
SITE = {  # manzana -> (jobs, quality)
    # Costanera Center: 29,500 m2 of offices occupied in March 2023 + 134,000 m2 of mall. Office occupancy grew later: low.
    "13123081002004": (round(29500 / OFFICE_M2 + 134000 / MALL_M2), "known_low"),
    # MUT: 45,765 m2 of offices, 88% leased + 40,000 m2 of shops, 75% leased (July 2024)
    "13114131009033": (round(45765 * 0.88 / OFFICE_M2 + 40000 * 0.75 / MALL_M2), "ok"),
}


def utm19s_to_lonlat(x, y):
    a, f, k0 = 6378137.0, 1 / 298.257223563, 0.9996
    e2 = f * (2 - f)
    ep2 = e2 / (1 - e2)
    m = (y - 10000000.0) / k0
    mu = m / (a * (1 - e2 / 4 - 3 * e2**2 / 64 - 5 * e2**3 / 256))
    e1 = (1 - math.sqrt(1 - e2)) / (1 + math.sqrt(1 - e2))
    p1 = (mu + (3 * e1 / 2 - 27 * e1**3 / 32) * math.sin(2 * mu)
          + (21 * e1**2 / 16 - 55 * e1**4 / 32) * math.sin(4 * mu)
          + (151 * e1**3 / 96) * math.sin(6 * mu) + (1097 * e1**4 / 512) * math.sin(8 * mu))
    s, c, t = math.sin(p1), math.cos(p1), math.tan(p1)
    n = a / math.sqrt(1 - e2 * s * s)
    r = a * (1 - e2) / (1 - e2 * s * s) ** 1.5
    t2, c1, d = t * t, ep2 * c * c, (x - 500000.0) / (n * k0)
    lat = p1 - (n * t / r) * (d**2 / 2 - (5 + 3 * t2 + 10 * c1 - 4 * c1**2 - 9 * ep2) * d**4 / 24
                              + (61 + 90 * t2 + 298 * c1 + 45 * t2**2 - 252 * ep2 - 3 * c1**2) * d**6 / 720)
    lon = (d - (1 + 2 * t2 + c1) * d**3 / 6
           + (5 - 2 * c1 + 28 * t2 - 3 * c1**2 + 8 * ep2 + 24 * t2**2) * d**5 / 120) / c
    return math.degrees(lon) - 69.0, math.degrees(lat)


def rings(blob):
    """Rings of a GeoPackage 2D multipolygon blob (little endian, 32-byte envelope)."""
    o = 8 + 32 + 5
    out = []
    (npoly,) = struct.unpack_from("<I", blob, o); o += 4
    for _ in range(npoly):
        (nring,) = struct.unpack_from("<I", blob, o + 5); o += 9
        for _ in range(nring):
            (npt,) = struct.unpack_from("<I", blob, o); o += 4
            v = struct.unpack_from("<%dd" % (2 * npt), blob, o); o += 16 * npt
            out.append(list(zip(v[::2], v[1::2])))
    return out


def centroid(ring):
    a = cx = cy = 0
    for (x0, y0), (x1, y1) in zip(ring, ring[1:]):
        k = x0 * y1 - x1 * y0
        a, cx, cy = a + k, cx + (x0 + x1) * k, cy + (y0 + y1) * k
    return abs(a), cx / (3 * a), cy / (3 * a)


# manzana polygons and a grid index of their envelopes.
# zones.csv is not the base: most manzanas with 0 residents are not in it, and many jobs are there.
poly, grid, zones = {}, collections.defaultdict(list), {}
db = sqlite3.connect(RAW + "verify-people-work/cartography-rm.gpkg")
for mz, cut, persons, blob in db.execute("select MANZENT, CUT, n_per, SHAPE from Manzanas_CPV24"):
    mz = str(int(mz))
    poly[mz] = rings(blob)
    _, lon, lat = max(centroid(r) for r in poly[mz])  # shortcut: centroid of the largest ring
    zones[mz] = {"comuna_cut": str(cut), "persons": persons or 0, "lon": "%.6f" % lon, "lat": "%.6f" % lat}
    x0, x1, y0, y1 = struct.unpack_from("<4d", blob, 8)
    for i in range(int(x0 // CELL), int(x1 // CELL) + 1):
        for j in range(int(y0 // CELL), int(y1 // CELL) + 1):
            grid[i, j].append(mz)


def locate(lon, lat):
    """Manzana that contains the point, or the nearest manzana edge within SNAP_M."""
    i, j = int(lon // CELL), int(lat // CELL)
    kx = 111320 * math.cos(math.radians(lat))
    best, best_d = None, SNAP_M
    for mz in {m for a in (-1, 0, 1) for b in (-1, 0, 1) for m in grid.get((i + a, j + b), ())}:
        inside = False
        for ring in poly[mz]:
            for (xa, ya), (xb, yb) in zip(ring, ring[1:]):
                if (ya > lat) != (yb > lat) and lon < xa + (lat - ya) * (xb - xa) / (yb - ya):
                    inside = not inside
                ax, ay, bx, by = (xa - lon) * kx, (ya - lat) * 110574, (xb - lon) * kx, (yb - lat) * 110574
                dx, dy = bx - ax, by - ay
                t = max(0, min(1, -(ax * dx + ay * dy) / ((dx * dx + dy * dy) or 1)))
                d = math.hypot(ax + t * dx, ay + t * dy)
                if d < best_d:
                    best, best_d = mz, d
        if inside:
            return mz
    return best


num = lambda s: float(s.replace(",", "."))
eod = collections.Counter()      # manzana -> expanded workers, 2012, exact address
landmark = collections.Counter() # manzana -> expanded workers, 2012, spread from a landmark point
permits = collections.Counter()  # manzana -> jobs from new floor area
sample = collections.Counter()   # comuna -> sampled workers with a place
stats = collections.Counter()

tar = tarfile.open(RAW + "eod2012/eod2012.tgz")
f = io.TextIOWrapper(tar.extractfile("eod2012/EOD_STGO/personas.csv"), encoding="latin-1")
points = collections.defaultdict(list)  # workplace point -> Factor of each sampled worker
for p in csv.DictReader(f, delimiter=";"):
    if p["Ocupacion"]:
        stats["eod_workers"] += num(p["Factor"])
        if num(p["DirActividadCoordX"] or "0"):
            points[p["DirActividadCoordX"], p["DirActividadCoordY"]].append(num(p["Factor"]))
# The EOD geocoder put vague answers ("near Metro Tobalaba") on one landmark point.
# A point with LANDMARK_N or more sampled workers is a landmark, not one address.
landmarks = []
for (x, y), ws in points.items():
    lon, lat = utm19s_to_lonlat(num(x), num(y))
    mz = locate(lon, lat)
    if mz:
        stats["eod_placed"] += sum(ws)
        sample[zones[mz]["comuna_cut"]] += len(ws)
        if len(ws) >= LANDMARK_N:
            landmarks.append((lon, lat, mz, sum(ws)))
        else:
            eod[mz] += sum(ws)
    elif len(ws) > 100:
        # The airport terminal (126 sampled workers on one point) is rural land with no manzana. It gets its own row.
        # shortcut: it is the only point of the RM this large with no manzana; a second one needs the comuna polygons.
        zones[AIRPORT] = {"comuna_cut": "13124", "persons": 0, "lon": "%.6f" % lon, "lat": "%.6f" % lat}
        eod[AIRPORT] += sum(ws)
        sample["13124"] += len(ws)
        stats["eod_placed"] += sum(ws)

# shortcut: a project with many permits (changes of the same project) counts one time,
# as the largest permit of one use text in one manzana; two real buildings with the same use lose the smaller.
site = {}
for r in json.load(open(RAW + "ine_permits/rm_permits.json")):
    a, g = r["attributes"], r["geometry"]
    if a["USO_DESTINO"] == "NO HABITACIONAL" and a["AÑO"] >= 2013 and a["SUPERFICIE"]:
        stats["permit_m2"] += a["SUPERFICIE"]
        mz = locate(g["x"], g["y"])
        if mz:
            k = mz, a["GLOSA_DESTINO"].lower()
            site[k] = max(site.get(k, 0), a["SUPERFICIE"])
for (mz, _), m2 in site.items():
    permits[mz] += m2 / M2_PER_JOB
    stats["permit_m2_used"] += m2

# spread each landmark over the exact-address workers around it (2012 answers can not point to later buildings)
for lon, lat, mz0, w in landmarks:
    kx = 111320 * math.cos(math.radians(lat))
    near = {mz: v for mz, v in eod.items()
            if math.hypot((float(zones[mz]["lon"]) - lon) * kx, (float(zones[mz]["lat"]) - lat) * 110574) < LANDMARK_M}
    near = near or {mz0: 1}
    stats["landmark_workers"] += w
    for mz, v in near.items():
        landmark[mz] += w * v / sum(near.values())

total = collections.Counter()
for r in csv.DictReader(open(RAW + "censo2024-personas/od_work_rm.csv")):
    if r["work_comuna"].startswith("13"):
        total[r["work_comuna"]] += int(r["personas"])

by_comuna = collections.defaultdict(dict)  # comuna -> manzana -> (weight, method)
for mz in set(eod) | set(permits) | set(landmark):
    method = "+".join(n for n, s in (("eod2012", eod), ("eod2012_landmark", landmark), ("permits2013_2025", permits)) if s[mz])
    if sample[zones[mz]["comuna_cut"]] >= MIN_SAMPLE:
        by_comuna[zones[mz]["comuna_cut"]][mz] = (eod[mz] + landmark[mz] + permits[mz], method)

rows = []
for cut, jobs in sorted(total.items()):
    w = by_comuna[cut]
    if not w:
        w = {mz: (z["persons"], "persons_fallback") for mz, z in zones.items() if z["comuna_cut"] == cut and z["persons"]}
        stats["jobs_fallback"] += jobs
    fixed = {mz: j for mz, (j, _) in SITE.items() if zones[mz]["comuna_cut"] == cut}
    w = {mz: v for mz, v in w.items() if mz not in fixed}
    jobs -= sum(fixed.values())  # the comuna total stays: the other manzanas share the rest
    s = sum(v for v, _ in w.values())
    # largest remainder: integer jobs that sum to the Census total
    share = {mz: jobs * v / s for mz, (v, _) in w.items()}
    n = {mz: int(v) for mz, v in share.items()}
    for mz in sorted(sorted(share), key=lambda m: share[m] - n[m], reverse=True)[: jobs - sum(n.values())]:  # sorted first: ties do not depend on the hash seed
        n[mz] += 1
    rows += [(mz, cut, zones[mz]["lon"], zones[mz]["lat"], n[mz], w[mz][1], "known_high" if mz in KNOWN_HIGH else "ok") for mz in sorted(n) if n[mz]]
    rows += [(mz, cut, zones[mz]["lon"], zones[mz]["lat"], j, "floor_area", SITE[mz][1]) for mz, j in fixed.items()]

with open(PACK + "/jobs.csv", "w", newline="") as f:
    out = csv.writer(f)
    out.writerow(["block_id", "cut", "longitude", "latitude", "jobs", "method", "quality"])
    out.writerows(rows)

print(len(rows), "rows,", sum(r[4] for r in rows), "jobs, Census total", sum(total.values()))
print({k: round(v) for k, v in stats.items()}, len(landmarks), "landmarks")
print("sample by comuna", sorted(sample.items(), key=lambda kv: kv[1]))
if len(sys.argv) > 2:  # debug: python3 -I tools/pack/jobs.py PACK_DIR ZONE_ID
    mz = sys.argv[2]
    cut = zones[mz]["comuna_cut"]
    print(mz, "eod", round(eod[mz]), "landmark", round(landmark[mz]), "permits", round(permits[mz]),
          "comuna scale", total[cut] / sum(v for v, _ in by_comuna[cut].values()))
