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
import csv, collections, heapq, io, math, pathlib, statistics, sys, unicodedata, zipfile

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
# Ride and dwell seconds, from the trip of the pattern whose first departure is nearest
# 08:00. A frequency trip carries offsets, so every trip of the pattern gives the same
# numbers; a Metro trip has absolute times, so the trip near 08:00 is the right sample.
pattern_times = {}
for key, trip_list in patterns.items():
    rows = sequence[min(trip_list, key=lambda trip: abs(sequence[trip][0][3] - DEPART))]
    pattern_times[key] = (
        [rows[i + 1][2] - rows[i][3] for i in range(len(rows) - 1)],   # ride, dep -> arr
        [max(d - a, 0) for _, _, a, d in rows],                        # dwell at the stop
    )

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
    ride_times, dwells = pattern_times[key]
    for position, stop in enumerate(key):
        if stop not in coords:
            continue
        ready, arrived = ("r", index, position), ("a", index, position)
        # Reach the stop, wait, and be ready to depart.
        graph[("s", stop)].append((ready, wait, 0, 1))
        # Arrived on board: leave the vehicle, or stay on and pay this stop's dwell.
        graph[arrived].append((("s", stop), 0.0, 0, 0))
        graph[arrived].append((ready, dwells[position], dwells[position], 0))
        if position + 1 < len(key):
            on_board = max(ride_times[position], 0)
            graph[ready].append((("a", index, position + 1), on_board, on_board, 0))

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
reachable, covered, nearest = {}, {}, {}
for cut, (lon, lat) in points.items():
    near = sorted(((stop, metres(lon, lat, *coords[stop])) for stop in coords),
                  key=lambda x: x[1])
    reachable[cut] = [(s, d) for s, d in near if d <= ACCESS_M]
    covered[cut] = bool(reachable[cut])
    nearest[cut] = near[0][1]
outside = sorted(c for c in covered if not covered[c])
served = [c for c in covered if covered[c]]
print("stops within 1 km of a comuna point:",
      min(len(reachable[c]) for c in served), "to", max(len(reachable[c]) for c in served))
print(f"{len(outside)} comuna points have NO stop within {ACCESS_M:.0f} m, so they get no "
      f"row: {[(c, round(nearest[c] / 1000, 1)) for c in outside]}")
print(f"pairs left out for that reason: {52 * 51 - len(served) * (len(served) - 1)}")

rows = []
for origin in sorted(points):
    if not covered[origin]:
        continue
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
        if destination == origin or not covered[destination]:
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
                         round(answer[3]), round(answer[4])])
    print(f"  {origin} {len(best)} nodes", end=" ", flush=True)

OUT.parent.mkdir(parents=True, exist_ok=True)
with OUT.open("w", newline="", encoding="utf-8") as fh:
    writer = csv.writer(fh)
    writer.writerow(["origin_cut", "destination_cut", "minutes_door_to_door",
                     "minutes_in_vehicle", "transfers", "walk_m_access", "walk_m_egress"])
    writer.writerows(rows)
print(OUT, len(rows), "directed pairs over", len(served), "comunas")


EOD = pathlib.Path("data/raw/eod2012/x/eod2012/EOD_STGO")
# Bus TS, bus no TS, Metro and their pure combinations. No taxi, no colectivo, no car.
EOD_PUBLIC = {"2", "3", "4", "7", "9", "10"}
EOD_MIN_TRIPS = 10


def compare_with_eod():
    """Descriptive comparison with the observed trip durations of EOD 2012.

    It is NOT a measure of forecast accuracy: the periods, the cohorts and the trip ends
    all differ. It says whether the matrix orders the comuna pairs as the survey did.
    """
    def plain(text):
        text = unicodedata.normalize("NFKD", text.upper()).encode("ascii", "ignore").decode()
        return " ".join(text.split())

    cut_of = {plain(r["name"]): r["cut"] for r in csv.DictReader(COMMUNES.open(encoding="utf-8"))}
    eod_cut = {}
    for row in csv.DictReader((EOD / "Tablas_parametros/Comuna.csv").open(encoding="utf-8")):
        if plain(row["Comuna"]) in cut_of:
            eod_cut[row["Id"]] = cut_of[plain(row["Comuna"])]
    matrix = {(r["origin_cut"], r["destination_cut"]): float(r["minutes_door_to_door"])
              for r in csv.DictReader(OUT.open(encoding="utf-8"))}
    observed = collections.defaultdict(list)
    for row in csv.DictReader((EOD / "viajes.csv").open(encoding="utf-8"), delimiter=";"):
        if (row["Periodo"] != "2" or row["ModoAgregado"] not in EOD_PUBLIC
                or not row["FactorLaboralNormal"]):
            continue
        origin, destination = eod_cut.get(row["ComunaOrigen"]), eod_cut.get(row["ComunaDestino"])
        if origin is None or destination is None or origin == destination:
            continue
        if (origin, destination) not in matrix:
            continue
        minutes = float(row["TiempoViaje"].replace(",", ".") or 0)
        if 0 < minutes <= 300:
            observed[(origin, destination)].append(minutes)
    pairs = [(k, v) for k, v in observed.items() if len(v) >= EOD_MIN_TRIPS]
    survey = [statistics.median(v) for _, v in pairs]
    model = [matrix[k] for k, _ in pairs]
    print(f"{sum(len(v) for v in observed.values())} sampled trips on {len(observed)} pairs; "
          f"{len(pairs)} pairs carry {EOD_MIN_TRIPS} trips or more")
    print(f"EOD 2012 median {statistics.median(survey):.1f} min | this matrix "
          f"{statistics.median(model):.1f} min")
    ratios = sorted(m / s for m, s in zip(model, survey))
    print(f"ratio model/EOD: median {statistics.median(ratios):.3f} "
          f"p10 {ratios[len(ratios) // 10]:.3f} p90 {ratios[9 * len(ratios) // 10]:.3f}")
    print(f"mean absolute error {statistics.mean(abs(m - s) for m, s in zip(model, survey)):.1f}"
          f" min | WAPE {sum(abs(m - s) for m, s in zip(model, survey)) / sum(survey):.1%}")
    ms, mm = statistics.mean(survey), statistics.mean(model)
    pearson = (sum((s - ms) * (m - mm) for s, m in zip(survey, model))
               / (sum((s - ms) ** 2 for s in survey) * sum((m - mm) ** 2 for m in model)) ** 0.5)
    rank = lambda values: {v: i for i, v in enumerate(sorted(range(len(values)),
                                                            key=lambda j: values[j]))}
    rs, rm = rank(survey), rank(model)
    n = len(survey)
    spearman = 1 - 6 * sum((rs[i] - rm[i]) ** 2 for i in range(n)) / (n * (n * n - 1))
    print(f"Pearson {pearson:.3f} | Spearman {spearman:.3f}")


if "eod" in sys.argv:
    compare_with_eod()
