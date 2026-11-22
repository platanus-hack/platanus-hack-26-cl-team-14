"""Match illustrative stadium arrivals to the same regular-weekday GTFS runs the viewer draws.

Run from the repo root: python3 tools/pack/event_transit_routes.py GTFS_20260321_v3.zip urban.osm.pbf B-full-result.json
Requires Shapely 2 and osmium-tool. The event result supplies all demand; this file supplies geometry references.
"""

import bisect
import csv
import gzip
import hashlib
import io
import json
import math
import sys
from collections import defaultdict
from functools import lru_cache
from pathlib import Path
from zipfile import ZipFile

from event_gate_walks import GATE, degrees, egress, gate_networks, indoor_routes
from shapely.geometry import Point, shape

DATA = Path("web/data")
VENUES = {
    "mut": ([-70.601132, -33.417280], {"TB-TOB"}),
    "monumental": ([-70.605966, -33.506537], {"PE", "MA", "SJ"}),
}
BUS_EXIT_RADIUS_M = 1200
WINDOW_START = 14 * 3600
WINDOW_END = 15 * 3600


def read(name):
    path = DATA / name
    with gzip.open(path) if name.endswith(".gz") else path.open() as file:
        return json.load(file)


def metres(a, b):
    return math.hypot((a[0] - b[0]) * 92915, (a[1] - b[1]) * 110905)


def clock(value):
    h, m, s = map(int, value.split(":"))
    return h * 3600 + m * 60 + s


feed, pbf, scenario = map(Path, sys.argv[1:])
networks = gate_networks(pbf, json.loads(scenario.read_bytes()), VENUES)
doors = read("metro_entrances.json.gz")["stations"]
surface = read("rider_services.json.gz")
metro = read("metro_schedule.json.gz")
digest = hashlib.sha256(feed.read_bytes()).hexdigest()
if digest != surface["source_sha256"] or digest != metro["source_sha256"]:
    raise ValueError("Use the GTFS feed that built both native schedules.")

with ZipFile(feed) as archive:

    def rows(name):
        return csv.DictReader(
            io.TextIOWrapper(archive.open(name + ".txt"), encoding="utf-8-sig")
        )

    stops = {r["stop_id"]: r for r in rows("stops")}
    coords = {
        k: [float(s["stop_lon"]), float(s["stop_lat"])]
        for k, s in stops.items()
        if s["stop_lon"]
    }
    source_pathways = {r["pathway_id"]: r for r in rows("pathways")}
    station_graph = indoor_routes(source_pathways.values())
    areas = defaultdict(list)
    for sid, stop in stops.items():
        if stop["location_type"] == "4":
            areas[stop["parent_station"]].append(sid)
    weekday = {r["service_id"] for r in rows("calendar") if r["monday"] == "1"}
    trips = {r["trip_id"]: r for r in rows("trips") if r["service_id"] in weekday}
    route_type = {r["route_id"]: r["route_type"] for r in rows("routes")}
    ordered = defaultdict(list)
    for r in rows("stop_times"):
        if r["trip_id"] in trips:
            ordered[r["trip_id"]].append(
                (
                    int(r["stop_sequence"]),
                    r["stop_id"],
                    clock(r["arrival_time"]),
                    clock(r["departure_time"]),
                )
            )
    patterns = defaultdict(list)
    for trip, seq in ordered.items():
        seq.sort()
        t = trips[trip]
        if route_type[t["route_id"]] != "1":
            patterns[
                t["route_id"],
                t["direction_id"],
                t["shape_id"],
                tuple(s[1] for s in seq),
            ].append(trip)

# Rebuild the pattern order from the native producer. Two routes may use the
# same stops but different shape or direction. Keep their templates separate.
published = defaultdict(list)
for service, begin, end, _, candidates in surface["legs"]:
    if candidates:
        published[service].append((tuple(begin), tuple(end)))
active_patterns = []
for key in sorted(patterns):
    service, _, _, stations = key
    positions = defaultdict(list)
    for i, sid in enumerate(stations):
        positions[tuple(round(n, 5) for n in coords[sid])].append(i)
    if any(
        a in positions and b in positions and positions[a][0] < positions[b][-1]
        for a, b in published[service]
    ):
        active_patterns.append(key)
if len(active_patterns) != len(surface["patterns"]):
    raise ValueError("Native pattern indexing differs from this GTFS feed.")
templates = defaultdict(list)
for template, (pattern, offsets, starts, kind) in enumerate(surface["templates"]):
    key = active_patterns[pattern]
    if len(key[3]) != len(offsets):
        raise ValueError("GTFS and native calls differ.")
    if kind == "3" and starts:
        templates[pattern].append(template)
# Every published service reference must give the source endpoint coordinates.
for service, begin, end, _, candidates in surface["legs"]:
    for template, i, j in candidates:
        key = active_patterns[surface["templates"][template][0]]
        if key[0] != service or any(
            [round(c, 5) for c in coords[key[3][idx]]] != xy
            for idx, xy in [(i, begin), (j, end)]
        ):
            raise ValueError(
                "Native boarding/alighting indices do not match the GTFS stops."
            )

boundaries = [
    (f["properties"]["cut"], shape(f["geometry"]))
    for f in json.loads(Path("data/mvp/communes.geojson").read_bytes())["features"]
]
with Path("data/mvp/communes.csv").open() as census:
    centres = {
        r["cut"]: [float(r["longitude"]), float(r["latitude"])]
        for r in csv.DictReader(census)
    }


@lru_cache(None)
def home(stop):
    point = Point(coords[stop])
    return next((cut for cut, poly in boundaries if poly.covers(point)), None)


tracks = [r for r in read("transit_paths.json") if r["mode"] == "metro"]
schedules = {(r["route"], r["direction"]): r["trips"] for r in metro["lines"]}
rail_runs = [schedules[t["route"], t["direction"]] for t in tracks]


@lru_cache(None)
def rail_times(track, start, end):
    options = []
    for run, (tid, calls) in enumerate(rail_runs[track]):
        board = next((c for c in calls if c[3] == start), None)
        alight = next((c for c in calls if c[3] == end), None)
        if board and alight and board[1] < alight[0]:
            options.append((alight[0], board[1], run))
    return sorted(options)


@lru_cache(None)
def final_walk(venue, stop_id, platform=None):
    network = networks[venue]
    choices = []
    if platform is None:
        for node in network.platforms.get(stop_id, []):
            walk = network.walk(node)
            offset = metres(coords[stop_id], degrees(network.nodes[node]))
            if walk and offset <= 50:
                walk["gtfs_to_osm_platform_offset_m"] = round(offset, 1)
                choices.append((offset, walk))
    else:
        for door in doors.get(stop_id, []):
            if (
                not door.get("osm")
                or venue == "mut"
                and door["gtfs"] in {"TB-TOB:B", "TB-TOB:E", "TB-TOB:F"}
            ):
                continue
            walk = network.walk(door["osm"])
            inside = (
                egress(station_graph, areas[platform], door["gtfs"]) if walk else None
            )
            if inside:
                lengths = [
                    source_pathways[pid]["length"] for pid in inside["pathway_ids"]
                ]
                walk["station_egress"] = {
                    "platform_id": platform,
                    **inside,
                    "known_distance_m": sum(float(m) for m in lengths if m),
                    "distance_complete": all(lengths),
                }
                choices.append((inside["seconds"] + walk["seconds"], walk))
    if not choices:
        return None
    walk = min(choices, key=lambda c: c[0])[1]
    walk["total_seconds"] = walk["seconds"] + walk.get("station_egress", {}).get(
        "seconds", 0
    )
    walk["mode"] = "walk"
    walk["minutes"] = walk["seconds"] / 60
    return walk


@lru_cache(None)
def platform_at(track, index):
    # Every native run that calls here must use the same original GTFS platform.
    parent = tracks[track]["parent_station_ids"][index]
    platforms = {
        sid
        for tid, calls in rail_runs[track]
        if any(c[3] == index for c in calls)
        for _, sid, _, _ in ordered[tid]
        if stops[sid]["parent_station"] == parent
    }
    if len(platforms) == 1:
        return next(iter(platforms))


def example_metro(legs, deadline):
    evidence = []
    for track, start, end in reversed(legs):
        times = rail_times(track, start, end)
        i = bisect.bisect_right(times, (deadline, math.inf, math.inf)) - 1
        if i < 0:
            return None
        arrive, depart, run = times[i]
        evidence.append(
            {
                "departure": depart,
                "arrival": arrive,
                "run": run,
                "gtfs_trip": rail_runs[track][run][0],
            }
        )
        # Same physical parent station only. Allow five minutes for the illustrative transfer.
        deadline = depart - 300
    return list(reversed(evidence))


def rail_leg(track, i, j):
    t = tracks[track]
    return {
        "mode": "metro",
        "service_id": t["route"],
        "track": track,
        "from": i,
        "to": j,
        "board_stop_id": t["parent_station_ids"][i],
        "alight_stop_id": t["parent_station_ids"][j],
    }


def metro_options(destinations):
    final = defaultdict(list)
    for idx, track in enumerate(tracks):
        stations = track["parent_station_ids"]
        for j, sid in enumerate(stations):
            if sid not in destinations:
                continue
            for i, begin in enumerate(stations[:j]):
                final[begin].append((idx, i, j))
    paths = [(leg,) for choices in final.values() for leg in choices]
    for idx, track in enumerate(tracks):
        stations = track["parent_station_ids"]
        for j, transfer in enumerate(stations):
            for onward in final.get(transfer, []):
                if tracks[onward[0]]["route"] != track["route"]:
                    paths.extend(
                        ((idx, i, j), onward)
                        for i in range(j)
                        if stations[i] not in destinations
                    )
    return paths


def bus_example(choices, i, j, deadline):
    runs = []
    for template in choices:
        _, offsets, starts, _ = surface["templates"][template]
        k = bisect.bisect_right(starts, deadline - offsets[j][0]) - 1
        if k >= 0:
            start = starts[k]
            runs.append((start + offsets[j][0], start + offsets[i][1], template, start))
    if not runs:
        return None
    arrive, depart, template, start = max(runs)
    return {
        "departure": depart,
        "arrival": arrive,
        "template": template,
        "start": start,
    }


# Only use bus-to-station links already present in a published local OD plan.
# The native network establishes the ordered stops. No door-to-door walk is emitted.
def feeder_links():
    at = {
        (s, tuple(a), tuple(b)): choices
        for s, a, b, _, choices in surface["legs"]
        if choices
    }
    links = {}
    for od, plan in read("public_routes.json").items():
        segs = plan["segments"]
        for bus, walk, rail in zip(segs, segs[1:], segs[2:]):
            if [s["mode"] for s in (bus, walk, rail)] != ["bus", "walk", "metro"]:
                continue
            origin, alight = bus["stop_ids"]
            station = stops[rail["stop_ids"][0]]["parent_station"]
            if not station or origin not in coords or alight not in coords:
                continue
            key = (
                bus["service"],
                tuple(plan["path"][bus["start"]]),
                tuple(plan["path"][bus["end"]]),
            )
            candidates = defaultdict(list)
            for template, i, j in at.get(key, []):
                pk = surface["templates"][template][0]
                pattern = active_patterns[pk]
                if pattern[3][i] == origin and pattern[3][j] == alight:
                    candidates[pk, i, j].append(template)
            for (pk, i, j), tids in candidates.items():
                link = {
                    "from_stop": alight,
                    "to_station": station,
                    "source_od": od,
                    "stop_ids": walk["stop_ids"],
                    "distance_m": round(metres(coords[alight], coords[station])),
                }
                leg = {
                    "mode": "bus",
                    "service_id": bus["service"],
                    "direction": active_patterns[pk][1],
                    "templates": sorted(tids),
                    "from": i,
                    "to": j,
                    "board_stop_id": origin,
                    "alight_stop_id": alight,
                }
                links.setdefault((pk, i, j, station), (leg, link))
    return links.values()


def connections(legs, transfer, walk):
    """One complete chain per eligible final run, with no fabricated service starts."""
    schedules = []
    for leg in legs:
        i, j = leg["from"], leg["to"]
        if leg["mode"] == "metro":
            trip_times = [
                (arrival, departure, run, 0)
                for arrival, departure, run in rail_times(leg["track"], i, j)
            ]
        else:
            trip_times = []
            for template in leg["templates"]:
                _, offsets, starts, _ = surface["templates"][template]
                trip_times.extend(
                    (start + offsets[j][0], start + offsets[i][1], template, start)
                    for start in starts
                )
        schedules.append(sorted(trip_times))

    options = []
    for last in schedules[-1]:
        gate_arrival = last[0] + walk["total_seconds"]
        if not WINDOW_START <= gate_arrival < WINDOW_END:
            continue
        chain = [last]
        for previous in reversed(range(len(legs) - 1)):
            gap = 300
            if previous == 0 and transfer:
                gap = max(gap, math.ceil(transfer["distance_m"] / 1.4))
            deadline = chain[0][1] - gap
            times = schedules[previous]
            at = (
                bisect.bisect_right(times, (deadline, math.inf, math.inf, math.inf)) - 1
            )
            if at < 0 or deadline - times[at][0] > 1800:
                break
            chain.insert(0, times[at])
        if len(chain) != len(legs):
            continue
        calls = []
        for leg, (arrival, departure, vehicle, start) in zip(legs, chain):
            call = {
                "mode": leg["mode"],
                "from": leg["from"],
                "to": leg["to"],
                "departure_seconds": departure,
                "arrival_seconds": arrival,
            }
            if leg["mode"] == "metro":
                call.update(track=leg["track"], run=vehicle)
            else:
                call.update(template=vehicle, start_seconds=start)
            calls.append(call)
        options.append(
            {
                "departure_seconds": chain[0][1],
                "alight_seconds": last[0],
                "gate_arrival_seconds": gate_arrival,
                "legs": calls,
            }
        )
    return options


def offer(best, center, legs, evidence, walk, transfer=None):
    origin, end = legs[0]["board_stop_id"], legs[-1]["alight_stop_id"]
    cut = home(origin)
    if not cut or evidence[-1]["arrival"] + walk["total_seconds"] < WINDOW_START:
        return
    home_m, exit_m = (
        metres(centres[cut], coords[origin]),
        metres(center, coords[end]),
    )
    mode = legs[0]["mode"]
    # A representative origin inside the comuna, closest to its published center.
    # Break ties by fewest transfers, latest usable boarding, then venue proximity.
    score = (round(home_m), len(legs), -evidence[0]["departure"], round(exit_m))
    if (venue_key := (cut, mode)) not in best or score < best[venue_key][0]:
        best[venue_key] = (
            score,
            {
                "legs": legs,
                "board": coords[origin],
                "alight": coords[end],
                "origin_center_distance_m": round(home_m),
                "venue_distance_m": round(exit_m),
                "walk_path": walk,
            },
        )
        if transfer:
            best[venue_key][1]["transfer"] = transfer


def make_venues():
    feeders = list(feeder_links())
    venues = {}
    for venue, (center, stations) in VENUES.items():
        best = {}

        rail_by_origin = defaultdict(list)
        for path in metro_options(stations):
            track, _, end = path[-1]
            platform = platform_at(track, end)
            if not platform:
                continue
            walk = final_walk(venue, tracks[track]["parent_station_ids"][end], platform)
            if not walk:
                continue
            evidence = example_metro(path, WINDOW_END - 1 - walk["total_seconds"])
            if evidence:
                legs = [rail_leg(*leg) for leg in path]
                offer(best, center, legs, evidence, walk)
                rail_by_origin[legs[0]["board_stop_id"]].append((legs, evidence, walk))
        for pattern, candidates in templates.items():
            service, direction, _, stop_ids = active_patterns[pattern]
            for j, end in enumerate(stop_ids):
                if metres(center, coords[end]) > BUS_EXIT_RADIUS_M:
                    continue
                walk = final_walk(venue, end)
                if not walk:
                    continue
                for i, start in enumerate(stop_ids[:j]):
                    # Do not use a loop to carry people away from a venue stop and back.
                    if start == end or metres(center, coords[start]) <= metres(
                        center, coords[end]
                    ):
                        continue
                    example = bus_example(
                        candidates, i, j, WINDOW_END - 1 - walk["total_seconds"]
                    )
                    if example:
                        leg = {
                            "mode": "bus",
                            "service_id": service,
                            "direction": direction,
                            "templates": candidates,
                            "from": i,
                            "to": j,
                            "board_stop_id": start,
                            "alight_stop_id": end,
                        }
                        offer(best, center, [leg], [example], walk)
        direct = {cut for cut, _ in best}
        for bus, link in feeders:
            # Expand geographic coverage without replacing direct options.
            cut = home(bus["board_stop_id"])
            if not cut or cut in direct:
                continue
            for rail, timings, walk in rail_by_origin.get(link["to_station"], []):
                gap = max(300, math.ceil(link["distance_m"] / 1.4))
                deadline = timings[0]["departure"] - gap
                example = bus_example(
                    bus["templates"], bus["from"], bus["to"], deadline
                )
                if example and deadline - example["arrival"] <= 1800:
                    offer(best, center, [bus, *rail], [example, *timings], walk, link)
        by_cut = {}
        for (cut, mode), (_, option) in sorted(best.items()):
            departures = connections(
                option["legs"], option.get("transfer"), option["walk_path"]
            )
            if departures:
                if mode == "metro":
                    first, *onward = option["legs"]
                    platform = platform_at(first["track"], first["from"])
                    option["legs"] = [{**first, "board_platform_id": platform}, *onward]
                option["departure_options"] = departures
                by_cut.setdefault(cut, []).append(option)
        venues[venue] = {
            "center": center,
            "gate": degrees(networks[venue].gate.coords[0]),
            "communes": by_cut,
            "unresolved": sorted(set(centres) - by_cut.keys()),
        }
    return venues


result = {
    "source": surface["source_url"],
    "source_sha256": digest,
    "native_pack_sha256": {
        name: hashlib.sha256((DATA / name).read_bytes()).hexdigest()
        for name in (
            "rider_services.json.gz",
            "metro_schedule.json.gz",
            "transit_paths.json",
            "metro_entrances.json.gz",
            "estadio_monumental_parts.geojson",
        )
    },
    "walk_source": {
        "osm_source": read("metro_entrances.json.gz")["osm_source"],
        "osm_credit": "© OpenStreetMap contributors, ODbL 1.0",
        "extract_sha256": hashlib.file_digest(pbf.open("rb"), "sha256").hexdigest(),
        "scenario_sha256": hashlib.sha256(scenario.read_bytes()).hexdigest(),
        "gate_osm_node": GATE,
        "walk_kmh": 4.8,
    },
    "service_day": "regular Monday",
    "gate_arrival_window_seconds": [WINDOW_START, WINDOW_END],
    "time_unit": "seconds since midnight in Santiago; end exclusive",
    "maximum_bus_exit_distance_m": BUS_EXIT_RADIUS_M,
    "method": "Illustrative inbound options. Origin stop inside the comuna nearest its census center. Direct bus or up to two Metro lines at a common station. Else use a feeder bus and a transfer already listed in a published OD plan. Each option reaches the grounds gate from 14:00 inclusive to 15:00 exclusive. Outdoor walking follows connected OSM pedestrian ways and walkable street centerlines. A physical GTFS-matched platform or exit lies on or within one metre of a pedestrian way. Indoor Metro egress uses directed GTFS pathways and source traversal times; no indoor line is drawn. Match day was Sunday; the native regular Monday is a replay assumption. Connections allow at least five minutes to transfer. Distances to centers are straight-line. No fan assignment or walk from home to first boarding stop.",
    "venues": make_venues(),
}
Path("data/mvp/event_transit_routes.json").write_text(
    json.dumps(result, ensure_ascii=False, separators=(",", ":")) + "\n"
)
