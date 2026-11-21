"""Build weekday Metro and bus paths for the Santiago city view."""

import csv
import io
import json
from collections import defaultdict
from pathlib import Path
from zipfile import ZipFile


FEED = Path("data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip")
OUT = Path("web/data/transit_paths.json")
HALF_HOUR = 1800


def seconds(value):
    hours, minutes, secs = map(int, value.split(":"))
    return hours * 3600 + minutes * 60 + secs


def table(feed, name):
    with feed.open(name + ".txt") as source:
        yield from csv.DictReader(io.TextIOWrapper(source, encoding="utf-8-sig"))


with ZipFile(FEED) as feed:
    services = {r["service_id"] for r in table(feed, "calendar") if r["monday"] == "1"}
    routes = {r["route_id"]: r for r in table(feed, "routes") if r["route_type"] in {"1", "3"}}
    trips = {r["trip_id"]: r for r in table(feed, "trips")
             if r["service_id"] in services and r["route_id"] in routes}
    metro = {tid: trip for tid, trip in trips.items() if routes[trip["route_id"]]["route_type"] == "1"}
    parents = {r["stop_id"]: r["parent_station"] for r in table(feed, "stops")}
    sequences = defaultdict(list)
    for row in table(feed, "stop_times"):
        tid = row["trip_id"]
        if tid in metro:
            sequences[tid].append((int(row["stop_sequence"]), parents[row["stop_id"]], row["departure_time"]))

    # Each Metro trip has a timetable. Some start at an intermediate station.
    departures = defaultdict(lambda: [0] * 48)
    stations = {}
    for tid, calls in sequences.items():
        calls.sort()
        shape = metro[tid]["shape_id"]
        departures[shape][seconds(calls[0][2]) // HALF_HOUR] += 1
        if len(calls) > len(stations.get(shape, [])):
            stations[shape] = [station for _, station, _ in calls]

    # Bus start times are approximate. Integrate the service rate over each bin.
    for row in table(feed, "frequencies"):
        if row["trip_id"] not in trips:
            continue
        shape = trips[row["trip_id"]]["shape_id"]
        start, end = seconds(row["start_time"]), seconds(row["end_time"])
        headway = int(row["headway_secs"])
        for period in range(start // HALF_HOUR, (end - 1) // HALF_HOUR + 1):
            overlap = min(end, (period + 1) * HALF_HOUR) - max(start, period * HALF_HOUR)
            departures[shape][period] += overlap / headway

    points = defaultdict(list)
    for row in table(feed, "shapes"):
        shape = row["shape_id"]
        if shape in departures:
            points[shape].append((int(row["shape_pt_sequence"]), round(float(row["shape_pt_lon"]), 5),
                                  round(float(row["shape_pt_lat"]), 5)))


paths = []
for route, direction, shape in sorted({(t["route_id"], t["direction_id"], t["shape_id"]) for t in metro.values()}):
    paths.append({
        "mode": "metro", "route": routes[route]["route_short_name"],
        "color": "#" + routes[route]["route_color"], "direction": int(direction),
        "parent_station_ids": stations[shape],
        "path": [[lon, lat] for _, lon, lat in sorted(points[shape])],
        "trips": departures[shape],
    })

# Use one actual bus shape per route. Both directions contribute departures.
bus_shapes = defaultdict(set)
for trip in trips.values():
    if routes[trip["route_id"]]["route_type"] == "3":
        bus_shapes[trip["route_id"]].add(trip["shape_id"])

for route, shapes in sorted(bus_shapes.items()):
    shape = max(sorted(shapes), key=lambda s: sum(departures[s]))
    coords = [[lon, lat] for _, lon, lat in sorted(points[shape])]
    paths.append({
        "mode": "bus", "route": routes[route]["route_short_name"],
        "color": "#" + routes[route]["route_color"],
        "path": coords[:-1:3] + coords[-1:],
        "trips": [round(sum(departures[s][i] for s in sorted(shapes)), 3) for i in range(48)],
    })

OUT.write_text(json.dumps(paths, separators=(",", ":")) + "\n", encoding="utf-8")
print(f"{len(paths)} paths, {OUT.stat().st_size:,} bytes")
