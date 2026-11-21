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

OUT.write_text(json.dumps(paths, separators=(",", ":")) + "\n", encoding="utf-8")
print(f"{len(paths)} paths, {OUT.stat().st_size:,} bytes")
