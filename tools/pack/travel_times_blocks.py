"""Public-transport minutes between comunas from real blocks, including the diagonal.

`travel_times.csv` routes between one population point for each comuna. That cannot
give a same-comuna time (one point has no distance to itself) and it drops the 17
comunas whose point has no stop within 1 km. This builder routes between real
populated blocks instead, so:

- a same-comuna time exists, as the worker-weighted mean over block pairs of that
  comuna, with the pair origin = destination left out;
- a comuna counts as served when a sampled block of it has a stop within 1 km, which
  recovers 6 of the 17 comunas that the centroid matrix dropped.

Measured first, so the limit is known: of the 467,551 workers of those 17 comunas only
34,039 (7.3%) live within 1 km of a stop. Eleven comunas have no block within 1 km at
all (211,532 workers). The March 2026 Gran Santiago feed does not serve them, so this
file reports no row for them. A fallback there is an assumption, never a measurement.

Same conventions as `travel_times.py`: weekday services, 08:00 departure, access and
egress walk to 1,000 m at 4.8 km/h, transfer walk to 300 m, one boarding wait of half
the 08:00 headway, dwell charged at each stop.

Sampling is deterministic: the blocks of a comuna are sorted by `block_id`, then
`SAMPLE` of them are taken at even steps of the cumulative worker count, so the sample
follows where the workers are and the same input always gives the same rows.

Output: data/mvp/travel_times_blocks.csv
        origin_cut,destination_cut,minutes,origin_blocks,destination_blocks,worker_pairs
Usage:  python3 -I tools/pack/travel_times_blocks.py [sample]
"""
import collections, csv, heapq, io, math, pathlib, sys, zipfile

GTFS = pathlib.Path("data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip")
BLOCKS = pathlib.Path("data/mvp/blocks.csv")
OUT = pathlib.Path("data/mvp/travel_times_blocks.csv")
WEEKDAY_SERVICES = {"L", "LJ"}
DEPART = 8 * 3600
ACCESS_M, TRANSFER_M, WALK_MPS = 1000.0, 300.0, 4.8 * 1000 / 3600
SAMPLE = int(sys.argv[1]) if len(sys.argv) > 1 else 4
KX, KY = 111320 * math.cos(math.radians(-33.45)), 110540

z = zipfile.ZipFile(GTFS)
table = lambda name: csv.DictReader(io.TextIOWrapper(z.open(name), encoding="utf-8-sig"))
seconds = lambda t: sum(int(p) * m for p, m in zip(t.split(":"), (3600, 60, 1)))
metres = lambda a, b, c, d: math.hypot((c - a) * KX, (d - b) * KY)

trips = {r["trip_id"]: r["service_id"] for r in table("trips.txt")}
headways = collections.defaultdict(list)
for row in table("frequencies.txt"):
    if seconds(row["start_time"]) <= DEPART < seconds(row["end_time"]):
        headways[row["trip_id"]].append(int(row["headway_secs"]))
stop_rows = {r["stop_id"]: r for r in table("stops.txt") if r["stop_lat"]}
coords = {k: (float(v["stop_lon"]), float(v["stop_lat"])) for k, v in stop_rows.items()}

sequence = collections.defaultdict(list)
for row in table("stop_times.txt"):
    if trips.get(row["trip_id"]) in WEEKDAY_SERVICES:
        sequence[row["trip_id"]].append((int(row["stop_sequence"]), row["stop_id"],
                                         seconds(row["arrival_time"]),
                                         seconds(row["departure_time"])))
patterns = collections.defaultdict(list)
for trip, rows in sequence.items():
    rows.sort()
    patterns[tuple(s for _, s, _, _ in rows)].append(trip)
pattern_times, pattern_wait = {}, {}
for key, trip_list in patterns.items():
    rows = sequence[min(trip_list, key=lambda t: abs(sequence[t][0][3] - DEPART))]
    pattern_times[key] = ([rows[i + 1][2] - rows[i][3] for i in range(len(rows) - 1)],
                          [max(d - a, 0) for _, _, a, d in rows])
    stated = [min(headways[t]) for t in trip_list if t in headways]
    if stated:
        pattern_wait[key] = min(stated) / 2
        continue
    departures = [sequence[t][0][3] for t in trip_list
                  if DEPART <= sequence[t][0][3] < DEPART + 3600]
    if departures:
        pattern_wait[key] = (3600 / len(departures)) / 2

graph = collections.defaultdict(list)
for index, (key, wait) in enumerate(pattern_wait.items()):
    ride, dwell = pattern_times[key]
    for position, stop in enumerate(key):
        if stop not in coords:
            continue
        graph[("s", stop)].append((("r", index, position), wait))
        graph[("a", index, position)].append((("s", stop), 0.0))
        graph[("a", index, position)].append((("r", index, position), dwell[position]))
        if position + 1 < len(key):
            graph[("r", index, position)].append((("a", index, position + 1),
                                                  max(ride[position], 0)))
CELL = TRANSFER_M / 111320 * 2
grid = collections.defaultdict(list)
for stop, (lon, lat) in coords.items():
    grid[(int(lon / CELL), int(lat / CELL))].append(stop)
for stop, (lon, lat) in coords.items():
    cell = (int(lon / CELL), int(lat / CELL))
    for dx in (-1, 0, 1):
        for dy in (-1, 0, 1):
            for other in grid[(cell[0] + dx, cell[1] + dy)]:
                if other != stop:
                    distance = metres(lon, lat, *coords[other])
                    if distance <= TRANSFER_M:
                        graph[("s", stop)].append((("s", other), distance / WALK_MPS))
for stop, row in stop_rows.items():
    parent = row["parent_station"]
    if parent in coords:
        graph[("s", stop)].append((("s", parent), 0.0))
        graph[("s", parent)].append((("s", stop), 0.0))
print(f"{len(patterns)} patterns, {len(graph)} nodes")

# Stops within ACCESS_M of a block, over a 1 km grid of stops.
ACELL = 1000.0
agrid = collections.defaultdict(list)
for stop, (lon, lat) in coords.items():
    agrid[(int(lon * KX / ACELL), int(lat * KY / ACELL))].append(stop)
def access(lon, lat):
    cx, cy = int(lon * KX / ACELL), int(lat * KY / ACELL)
    out = []
    for dx in (-2, -1, 0, 1, 2):
        for dy in (-2, -1, 0, 1, 2):
            for stop in agrid[(cx + dx, cy + dy)]:
                distance = metres(lon, lat, *coords[stop])
                if distance <= ACCESS_M:
                    out.append((stop, distance))
    return out

by_comuna = collections.defaultdict(list)
for row in csv.DictReader(BLOCKS.open(encoding="utf-8")):
    workers = float(row["workers"] or 0)
    if workers > 0:
        by_comuna[row["cut"]].append((row["block_id"], float(row["longitude"]),
                                      float(row["latitude"]), workers))

# Sample only among blocks that have a stop within ACCESS_M. Where a comuna is partly
# served that conditions the result: the minutes then describe a worker of that comuna
# who lives within 1 km of a stop, not the average worker. `access_share` records it.
sampled, share = {}, {}
for cut, blocks in by_comuna.items():
    blocks.sort()
    with_access = [b + (near,) for b in blocks if (near := access(b[1], b[2]))]
    share[cut] = (sum(b[3] for b in with_access) / sum(b[3] for b in blocks)) if blocks else 0.0
    total = sum(b[3] for b in with_access)
    chosen, step = [], total / SAMPLE if total else 0.0
    for i in range(SAMPLE):
        target, cumulative = step * (i + 0.5), 0.0
        for block in with_access:
            cumulative += block[3]
            if cumulative >= target:
                if block[0] not in {c[0] for c in chosen}:
                    chosen.append(block)
                break
    sampled[cut] = chosen
served = sorted(c for c in sampled if sampled[c])
print(f"{len(served)} of 52 comunas have a sampled block with a stop within 1 km; "
      f"not served: {sorted(set(by_comuna) - set(served))}")

rows = []
for origin in served:
    for block in sampled[origin]:
        best, queue, order = {}, [], 0
        for stop, distance in block[4]:
            heapq.heappush(queue, (distance / WALK_MPS, order, ("s", stop)))
            order += 1
        while queue:
            cost, _, node = heapq.heappop(queue)
            if node in best:
                continue
            best[node] = cost
            for target, edge in graph[node]:
                if target not in best:
                    order += 1
                    heapq.heappush(queue, (cost + edge, order, target))
        block_times = {}
        for destination in served:
            weight = value = 0.0
            pairs = 0
            for other in sampled[destination]:
                if other[0] == block[0]:
                    continue
                answer = min((best[("s", s)] + d / WALK_MPS for s, d in other[4]
                              if ("s", s) in best), default=None)
                if answer is not None:
                    value += answer * other[3]
                    weight += other[3]
                    pairs += 1
            if weight:
                block_times[destination] = (value / weight, pairs)
        for destination, (seconds_mean, pairs) in block_times.items():
            rows.append((origin, destination, block[3], seconds_mean, pairs))
    print(f"  {origin}", end="", flush=True)
print()

totals = collections.defaultdict(lambda: [0.0, 0.0, 0, 0])
for origin, destination, workers, seconds_mean, pairs in rows:
    cell = totals[(origin, destination)]
    cell[0] += seconds_mean * workers
    cell[1] += workers
    cell[2] += 1
    cell[3] += pairs
OUT.parent.mkdir(parents=True, exist_ok=True)
with OUT.open("w", newline="", encoding="utf-8") as fh:
    writer = csv.writer(fh)
    writer.writerow(["origin_cut", "destination_cut", "minutes", "origin_blocks",
                     "destination_blocks", "worker_pairs", "origin_access_share",
                     "destination_access_share"])
    for (origin, destination), cell in sorted(totals.items()):
        writer.writerow([origin, destination, round(cell[0] / cell[1] / 60, 1),
                         cell[2], len(sampled[destination]), cell[3],
                         round(share[origin], 4), round(share[destination], 4)])
print(OUT, len(totals), "pairs,", sum(1 for k in totals if k[0] == k[1]), "diagonal")
