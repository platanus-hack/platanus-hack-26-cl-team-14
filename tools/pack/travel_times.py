"""Comuna-to-comuna public-transport travel times at 08:00 on a weekday (layer B).

Output: data/mvp/travel_times.csv
  origin_cut,destination_cut,minutes_door_to_door,minutes_in_vehicle,transfers,
  walk_m_access,walk_m_egress
2,652 directed pairs (52 x 52, the diagonal excluded). Builder: python3 -I tools/pack/travel_times.py

Model, frozen before the run (posted in the MVP thread):
- Network: March GTFS `GTFS_20260321_v3.zip`, sha256 f10945e8..b77b, the release that
  context/mvp.md names. Weekday = Wednesday 2026-04-15, inside the April boardings
  period: services `L` (Mon-Fri) and `LJ` (Mon-Thu). `V` is Friday only. No
  `calendar_dates` exception falls on that date.
- Every bus trip of this feed is in `frequencies.txt`, so its `stop_times` are offsets
  from an unknown trip start. A timetable search would have to invent a departure clock
  for 55.5% of the trips. So this is a HEADWAY model: trips collapse to stop patterns,
  the ride time on an edge is the exact offset difference, and one boarding costs
  headway / 2 at 08:00. A bus headway comes from `frequencies.txt`; a Metro headway
  comes from the count of trips that depart between 08:00 and 09:00. A pattern with no
  service at 08:00 is excluded, never given an invented headway.
- Points: the population-weighted comuna points of `communes.csv`.
- Access and egress: straight line to a stop within ACCESS_M at WALK_KMH. A transfer is
  a walk under TRANSFER_M, or 0 m between a platform and its `parent_station`.
- Left out: fares, crowding, capacity, the real walking network, car, bicycle, delay,
  and the diagonal. A comuna point is not a trip end, so these are times between two
  population centres, not a resident's trip. Nothing here is calibrated against an
  observed travel time.
"""
import csv, collections, heapq, io, math, pathlib, zipfile

GTFS = pathlib.Path("data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip")
COMMUNES = pathlib.Path("data/mvp/communes.csv")
OUT = pathlib.Path("data/mvp/travel_times.csv")
WEEKDAY_SERVICES = {"L", "LJ"}
DEPART = 8 * 3600
ACCESS_M, TRANSFER_M, WALK_KMH = 1000.0, 300.0, 4.8
WALK_MPS = WALK_KMH * 1000 / 3600


def seconds(text):
    h, m, s = text.split(":")
    return int(h) * 3600 + int(m) * 60 + int(s)


def metres(lon1, lat1, lon2, lat2):
    return math.hypot((lon2 - lon1) * math.cos(math.radians(lat1)) * 111320,
                      (lat2 - lat1) * 110574)


z = zipfile.ZipFile(GTFS)


def table(name):
    return csv.DictReader(io.TextIOWrapper(z.open(name), encoding="utf-8-sig"))


trips = {r["trip_id"]: r["service_id"] for r in table("trips.txt")}
headways = collections.defaultdict(list)
for row in table("frequencies.txt"):
    if seconds(row["start_time"]) <= DEPART < seconds(row["end_time"]):
        headways[row["trip_id"]].append(int(row["headway_secs"]))

stop_rows = {r["stop_id"]: r for r in table("stops.txt") if r["stop_lat"]}
coords = {k: (float(v["stop_lon"]), float(v["stop_lat"])) for k, v in stop_rows.items()}

# Stop sequences of the weekday trips, then collapse the trips to stop patterns.
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
# Ride seconds on each edge, from the first trip of the pattern.
pattern_edges = {}
for key, trip_list in patterns.items():
    rows = sequence[trip_list[0]]
    pattern_edges[key] = [rows[i + 1][2] - rows[i][3] for i in range(len(rows) - 1)]

# One boarding wait for each pattern: headway / 2 at 08:00.
pattern_wait = {}
for key, trip_list in patterns.items():
    stated = [min(headways[t]) for t in trip_list if t in headways]
    if stated:
        pattern_wait[key] = min(stated) / 2
        continue
    departures = [sequence[t][0][3] for t in trip_list
                  if DEPART <= sequence[t][0][3] < DEPART + 3600]
    if departures:
        pattern_wait[key] = (3600 / len(departures)) / 2
print(f"{len(patterns)} patterns, {len(pattern_wait)} run at 08:00")

# Graph. A street node is a stop. A ride node is one place in one pattern.
# cost, in-vehicle seconds, boardings, access metres travel with each edge.
graph = collections.defaultdict(list)
for index, (key, wait) in enumerate(pattern_wait.items()):
    for position, stop in enumerate(key):
        if stop not in coords:
            continue
        ride = ("r", index, position)
        graph[("s", stop)].append((ride, wait, 0, 1))
        graph[ride].append((("s", stop), 0.0, 0, 0))
        if position + 1 < len(key):
            on_board = max(pattern_edges[key][position], 0)
            graph[ride].append((("r", index, position + 1), on_board, on_board, 0))

# Walk transfers: a grid keeps this to the near stops only.
CELL = TRANSFER_M / 111320 * 2
grid = collections.defaultdict(list)
for stop, (lon, lat) in coords.items():
    grid[(int(lon / CELL), int(lat / CELL))].append(stop)
walk_edges = 0
for stop, (lon, lat) in coords.items():
    cell = (int(lon / CELL), int(lat / CELL))
    for dx in (-1, 0, 1):
        for dy in (-1, 0, 1):
            for other in grid[(cell[0] + dx, cell[1] + dy)]:
                if other == stop:
                    continue
                distance = metres(lon, lat, *coords[other])
                if distance <= TRANSFER_M:
                    graph[("s", stop)].append((("s", other), distance / WALK_MPS, 0, 0))
                    walk_edges += 1
# A platform and its parent station are the same place.
for stop, row in stop_rows.items():
    parent = row["parent_station"]
    if parent in coords:
        graph[("s", stop)].append((("s", parent), 0.0, 0, 0))
        graph[("s", parent)].append((("s", stop), 0.0, 0, 0))
print(f"{len(graph)} nodes, {sum(len(v) for v in graph.values())} edges "
      f"({walk_edges} transfer walks)")

communes = list(csv.DictReader(COMMUNES.open(encoding="utf-8")))
points = {r["cut"]: (float(r["longitude"]), float(r["latitude"])) for r in communes}
# Stops within ACCESS_M of each comuna point.
reachable, covered = {}, {}
for cut, (lon, lat) in points.items():
    near = sorted(((stop, metres(lon, lat, *coords[stop])) for stop in coords),
                  key=lambda x: x[1])
    inside = [(s, d) for s, d in near if d <= ACCESS_M]
    reachable[cut] = inside or near[:1]
    covered[cut] = bool(inside)
outside = sorted(c for c in covered if not covered[c])
print("stops within 1 km of a comuna point:",
      min(len(v) for v in reachable.values()), "to", max(len(v) for v in reachable.values()))
print(f"{len(outside)} comuna points have NO stop within {ACCESS_M:.0f} m: {outside}")

rows = []
for origin in sorted(points):
    best = {}
    queue = []
    order = 0
    for stop, distance in reachable[origin]:
        heapq.heappush(queue, (distance / WALK_MPS, order, 0, 0, distance, ("s", stop)))
        order += 1
    while queue:
        cost, _, on_board, boardings, access, node = heapq.heappop(queue)
        if node in best:
            continue
        best[node] = (cost, on_board, boardings, access)
        for target, edge, vehicle, board in graph[node]:
            if target not in best:
                order += 1
                heapq.heappush(queue, (cost + edge, order, on_board + vehicle,
                                       boardings + board, access, target))
    for destination in sorted(points):
        if destination == origin:
            continue
        answer = None
        for stop, distance in reachable[destination]:
            label = best.get(("s", stop))
            if label:
                total = label[0] + distance / WALK_MPS
                if answer is None or total < answer[0]:
                    answer = (total, label[1], label[2], label[3], distance)
        if answer:
            rows.append([origin, destination, round(answer[0] / 60, 1),
                         round(answer[1] / 60, 1), max(answer[2] - 1, 0),
                         round(answer[3]), round(answer[4]),
                         "in_feed" if covered[origin] and covered[destination]
                         else "outside_feed"])
    print(f"  {origin} {len(best)} nodes", end=" ", flush=True)

OUT.parent.mkdir(parents=True, exist_ok=True)
with OUT.open("w", newline="", encoding="utf-8") as fh:
    writer = csv.writer(fh)
    writer.writerow(["origin_cut", "destination_cut", "minutes_door_to_door",
                     "minutes_in_vehicle", "transfers", "walk_m_access", "walk_m_egress",
                     "coverage"])
    writer.writerows(rows)
in_feed = sum(1 for r in rows if r[7] == "in_feed")
print(OUT, len(rows), "directed pairs of the 2,652 possible;",
      in_feed, "with coverage in_feed")
