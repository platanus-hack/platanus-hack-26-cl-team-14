"""Minutes saved from Quilicura by the 2023 Line 3 extension, one method, two networks."""
import csv, collections, heapq, io, math, sys, zipfile
GTFS, COMMUNES = sys.argv[1], sys.argv[2]
WEEKDAY, DEPART = {"L", "LJ"}, 8 * 3600
ACCESS_M, TRANSFER_M, WALK_MPS = 1000.0, 300.0, 4.8 * 1000 / 3600
NEW = ("FERROCARRIL", "LO CRUZAT", "QUILICURA")
z = zipfile.ZipFile(GTFS)
tab = lambda n: csv.DictReader(io.TextIOWrapper(z.open(n), encoding="utf-8-sig"))
def sec(t):
    h, m, s = t.split(":"); return int(h) * 3600 + int(m) * 60 + int(s)
def met(a, b, c, d):
    return math.hypot((c - a) * 111320 * math.cos(math.radians(-33.45)), (d - b) * 110540)
trips = {r["trip_id"]: r["service_id"] for r in tab("trips.txt")}
head = collections.defaultdict(list)
for r in tab("frequencies.txt"):
    if sec(r["start_time"]) <= DEPART < sec(r["end_time"]): head[r["trip_id"]].append(int(r["headway_secs"]))
srows = {r["stop_id"]: r for r in tab("stops.txt") if r["stop_lat"]}
allc = {k: (float(v["stop_lon"]), float(v["stop_lat"])) for k, v in srows.items()}
raw = collections.defaultdict(list)
for r in tab("stop_times.txt"):
    if trips.get(r["trip_id"]) in WEEKDAY:
        raw[r["trip_id"]].append((int(r["stop_sequence"]), r["stop_id"], sec(r["arrival_time"]), sec(r["departure_time"])))
NAMES = {"FERROCARRIL", "LO CRUZAT", "PLAZA QUILICURA"}
l3 = {s for s, r in srows.items()
      if r["stop_name"].strip().upper().split(" DIRECC")[0].strip() in NAMES
      and not r["stop_name"].strip().upper().startswith(("PB", "PI", "AV."))}
print(f"dropping {len(l3)} Metro stops: {sorted(srows[s]['stop_name'] for s in l3)}")

def run(drop):
    coords = {k: v for k, v in allc.items() if k not in drop}
    seq = {}
    for t, rows in raw.items():
        keep = sorted(r for r in rows if r[1] not in drop)
        if len(keep) > 1: seq[t] = keep
    pat = collections.defaultdict(list)
    for t, rows in seq.items(): pat[tuple(s for _, s, _, _ in rows)].append(t)
    ptime, pwait = {}, {}
    for key, tl in pat.items():
        rows = seq[min(tl, key=lambda t: abs(seq[t][0][3] - DEPART))]
        ptime[key] = ([rows[i+1][2] - rows[i][3] for i in range(len(rows)-1)],
                      [max(d - a, 0) for _, _, a, d in rows])
        st = [min(head[t]) for t in tl if t in head]
        if st: pwait[key] = min(st) / 2; continue
        dep = [seq[t][0][3] for t in tl if DEPART <= seq[t][0][3] < DEPART + 3600]
        if dep: pwait[key] = (3600 / len(dep)) / 2
    g = collections.defaultdict(list)
    for i, (key, wait) in enumerate(pwait.items()):
        ride, dwell = ptime[key]
        for p, stop in enumerate(key):
            if stop not in coords: continue
            g[("s", stop)].append((("r", i, p), wait, 0, 1))
            g[("a", i, p)].append((("s", stop), 0.0, 0, 0))
            g[("a", i, p)].append((("r", i, p), dwell[p], dwell[p], 0))
            if p + 1 < len(key): g[("r", i, p)].append((("a", i, p+1), max(ride[p], 0), max(ride[p], 0), 0))
    CELL = TRANSFER_M / 111320 * 2
    grid = collections.defaultdict(list)
    for s, (lo, la) in coords.items(): grid[(int(lo/CELL), int(la/CELL))].append(s)
    for s, (lo, la) in coords.items():
        c = (int(lo/CELL), int(la/CELL))
        for dx in (-1,0,1):
            for dy in (-1,0,1):
                for o in grid[(c[0]+dx, c[1]+dy)]:
                    if o != s:
                        d = met(lo, la, *coords[o])
                        if d <= TRANSFER_M: g[("s", s)].append((("s", o), d/WALK_MPS, 0, 0))
    for s, r in srows.items():
        p = r["parent_station"]
        if s in coords and p in coords:
            g[("s", s)].append((("s", p), 0.0, 0, 0)); g[("s", p)].append((("s", s), 0.0, 0, 0))
    return g, coords

pts = {r["cut"]: (float(r["longitude"]), float(r["latitude"])) for r in csv.DictReader(open(COMMUNES, encoding="utf-8"))}
for label, drop in (("with the extension (GTFS as published)", set()), ("without the 3 stations", l3)):
    g, coords = run(drop)
    lo, la = pts["13125"]  # Quilicura
    near = sorted(((s, met(lo, la, *coords[s])) for s in coords), key=lambda x: x[1])
    start = [(s, d) for s, d in near if d <= ACCESS_M]
    best, q, order = {}, [], 0
    for s, d in start: heapq.heappush(q, (d/WALK_MPS, order, 0, 0, ("s", s))); order += 1
    while q:
        cost, _, onb, brd, node = heapq.heappop(q)
        if node in best: continue
        best[node] = (cost, onb, brd)
        for tgt, e, v, b in g[node]:
            if tgt not in best:
                order += 1; heapq.heappush(q, (cost+e, order, onb+v, brd+b, tgt))
    print(f"\n{label}: {len(start)} access stops, {len(best)} nodes reached")
    for name, cut in (("Santiago (centro)", "13101"), ("Providencia", "13123"), ("Las Condes", "13114")):
        dlo, dla = pts[cut]
        dn = sorted(((s, met(dlo, dla, *coords[s])) for s in coords), key=lambda x: x[1])
        ans = None
        for s, d in [(s, d) for s, d in dn if d <= ACCESS_M]:
            lab = best.get(("s", s))
            if lab:
                tot = lab[0] + d/WALK_MPS
                if ans is None or tot < ans[0]: ans = (tot, lab[1], lab[2])
        print(f"   Quilicura -> {name:20s} " + (f"{ans[0]/60:6.1f} min  ({ans[1]/60:.1f} on board, "
              f"{max(ans[2]-1,0)} transfers)" if ans else "unreachable"))
