"""Pack published Metro and bus patterns from the March 2026 Santiago GTFS."""
import argparse
import csv
import hashlib
import io
import json
from collections import defaultdict
from itertools import groupby
from pathlib import Path
from statistics import median_low
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip"
OUTPUT = ROOT / "data/mvp/editable_transit.json"
URL = "https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip"
REFERENCE_DATE = "20260413"  # Monday.
SHA256 = "f10945e80beebc39ea2ac70de997d3d69714897b575de6fbf4e3ca643013b77b"


def table(feed, name):
    with feed.open(name + ".txt") as source:
        yield from csv.DictReader(io.TextIOWrapper(source, encoding="utf-8-sig"))


def seconds(clock):
    hours, minutes, secs = map(int, clock.split(":"))
    return 3600 * hours + 60 * minutes + secs


def pack(source):
    digest = hashlib.sha256(source.read_bytes()).hexdigest()
    if digest != SHA256:
        raise ValueError("Use the March 21 v3 feed. This pack uses its trip-grouped stop times.")
    with ZipFile(source) as feed:
        info = next(table(feed, "feed_info"))
        calendar = list(table(feed, "calendar"))
        exceptions = list(table(feed, "calendar_dates"))
        active = {r["service_id"] for r in calendar if r["monday"] == "1" and r["start_date"] <= REFERENCE_DATE <= r["end_date"]}
        for r in exceptions:
            if r["date"] == REFERENCE_DATE:
                if r["exception_type"] == "1":
                    active.add(r["service_id"])
                else:
                    active.discard(r["service_id"])
        routes = {r["route_id"]: r for r in table(feed, "routes") if r["route_type"] in {"1", "3"}}
        trips = {r["trip_id"]: r for r in table(feed, "trips") if r["route_id"] in routes}
        source_stops = {r["stop_id"]: r for r in table(feed, "stops")}
        windows = defaultdict(list)
        for row in table(feed, "frequencies"):
            tid = row["trip_id"]
            if tid in trips:
                windows[tid].append([trips[tid]["service_id"], tid, row["start_time"], row["end_time"],
                                     int(row["headway_secs"]), int(row["exact_times"])])
        patterns = defaultdict(list)
        for tid, rows in groupby(table(feed, "stop_times"), key=lambda r: r["trip_id"]):
            if tid not in trips:
                continue
            trip = trips[tid]
            route = trip["route_id"]
            calls = sorted(rows, key=lambda r: int(r["stop_sequence"]))
            is_metro = routes[route]["route_type"] == "1"
            stops = tuple(source_stops[r["stop_id"]]["parent_station"] if is_metro else r["stop_id"] for r in calls)
            start = calls[0]["departure_time"]
            origin = seconds(start)
            profile = tuple((seconds(r["arrival_time"]) - origin, seconds(r["departure_time"]) - origin)
                            for r in calls)
            total = profile[-1][0]
            pickup = tuple(int(r["pickup_type"] or 0) for r in calls)
            dropoff = tuple(int(r["drop_off_type"] or 0) for r in calls)
            key = (route, int(trip["direction_id"]), trip["shape_id"], stops, pickup, dropoff)
            patterns[key].append((total, tid, profile, start))

        lines = {}
        used_stops = set()
        used_shapes = set()
        for (route, direction, shape, stops, pickup, dropoff), records in sorted(patterns.items()):
            calls_text = "\n".join(f"{sid}|{p}|{d}" for sid, p, d in zip(stops, pickup, dropoff))
            seq_hash = hashlib.sha256(calls_text.encode()).hexdigest()[:16]
            key = f"{route}:{direction}:{shape}:{seq_hash}"
            middle = median_low(record[0] for record in records)
            _, representative, sample, _ = min(record for record in records if record[0] == middle)
            minutes = [round((b[0] - a[1]) / 60, 3) for a, b in zip(sample, sample[1:])]
            dwell = [round((departure - arrival) / 60, 3) for arrival, departure in sample]
            profiles = {}
            trip_profile = {}
            departures, frequencies = [], []
            for _, tid, profile, start in records:
                trip_profile[tid] = profiles.setdefault(profile, len(profiles))
                if tid in windows:
                    frequencies.extend(windows[tid])
                else:
                    departures.append([trips[tid]["service_id"], tid, start])
            starts = sum(1 for service, _, clock in departures
                         if service in active and 28800 <= seconds(clock) < 32400)
            for service, _, start, end, headway, _ in frequencies:
                if service in active:
                    overlap = max(0, min(32400, seconds(end)) - max(28800, seconds(start)))
                    starts += overlap / headway
            lines[key] = {
                "mode": "metro" if routes[route]["route_type"] == "1" else "bus",
                "route_id": route, "route_name": routes[route]["route_short_name"],
                "direction": direction, "shape_id": shape, "stop_ids": stops, "segment_minutes": minutes,
                "dwell_minutes": dwell, "pickup_type": pickup, "drop_off_type": dropoff,
                "headway_minutes": round(60 / starts, 3) if starts else None,
                "representative_trip_id": representative, "fixed_departures": sorted(departures),
                "frequency_windows": sorted(frequencies),
                "timing_profiles": list(profiles), "trip_profile": trip_profile,
            }
            used_stops.update(stops)
            used_shapes.add(shape)

        stops = {sid: {"name": source_stops[sid]["stop_name"], "parent_id": source_stops[sid]["parent_station"] or None,
                       "coordinates": [round(float(source_stops[sid][k]), 6) for k in ("stop_lon", "stop_lat")]}
                 for sid in sorted(used_stops)}
        vertices = defaultdict(list)
        for r in table(feed, "shapes"):
            if r["shape_id"] in used_shapes:
                vertices[r["shape_id"]].append((int(r["shape_pt_sequence"]), round(float(r["shape_pt_lon"]), 6),
                                                round(float(r["shape_pt_lat"]), 6)))
        shapes = {sid: [[lon, lat] for _, lon, lat in sorted(vertices[sid])] for sid in sorted(used_shapes)}
    return {
        "source_url": URL, "source_sha256": digest, "feed": info, "calendar": calendar,
        "calendar_dates": exceptions,
        "definitions": {
            "line_id": "GTFS route_id:direction_id:shape_id:SHA256(ordered stop_id|pickup_type|drop_off_type records separated by LF)[:16].",
            "coordinates": "WGS84 longitude, latitude in degrees rounded to six decimals. Full GTFS shape; short trips use a subsection.",
            "stop_ids": "Metro uses GTFS parent stations. Bus uses individual GTFS stops.",
            "segment_minutes": "Scheduled departure to next arrival for one trip template; excludes dwell. Choose lower median end-to-end elapsed time among trip records in this pattern, then lexically first tied trip_id. Each record counts once regardless of headway. Round to 0.001 min.",
            "dwell_minutes": "Departure minus arrival at each stop of the same representative trip, in minutes. Round to 0.001 min.",
            "pickup_type/drop_off_type": "Per-stop GTFS codes; 0 regular, 1 unavailable, 2 phone agency, 3 contact driver. These flags distinguish patterns.",
            "headway_minutes": "Reciprocal service density at the first stop on Monday 2026-04-13 during [08:00,09:00), after calendar exceptions. Divide 60 by fixed starts plus expected frequency starts (overlap seconds / headway seconds). Null when there are no starts. Round to 0.001 min. Do not infer a waiting time from this rate. For fixed trips use scheduled calls. For frequency trips use active windows.",
            "timing_profiles": "Each profile lists [arrival_offset_secs, departure_offset_secs] in stop order, relative to first-stop departure. Add fixed-trip departure time to reconstruct scheduled calls. For frequency trips shift the service window using the relevant call offsets.",
            "trip_profile": "GTFS trip_id to zero-based index in this pattern's timing_profiles. Includes each scheduled trip and frequency template.",
            "fixed_departures": ["service_id", "trip_id", "first_stop_departure_time"],
            "frequency_windows": ["service_id", "trip_id", "start_time", "end_time", "headway_secs", "exact_times"],
            "service_time": "America/Santiago GTFS service-day times; hours may exceed 23. Windows include start and exclude end. exact_times=0 gives approximate headways. Use calendar dates and exceptions.",
            "coverage": "GTFS route types 1 and 3. Published service; actual travel times and passenger counts unavailable in this feed.",
        },
        "stops": stops, "shapes": shapes, "lines": lines,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gtfs", type=Path, default=SOURCE)
    parser.add_argument("--output", type=Path, default=OUTPUT)
    args = parser.parse_args()
    bundle = pack(args.gtfs)
    args.output.write_text(json.dumps(bundle, ensure_ascii=False, separators=(",", ":")) + "\n")
    print(f"{len(bundle['lines'])} patterns, {len(bundle['stops'])} stops, {len(bundle['shapes'])} shapes")
