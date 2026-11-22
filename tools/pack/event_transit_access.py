"""Pack walks from each representative home comuna to its first GTFS boarding stop.

Run with the regional OSRM foot graph from station_access_routes.md:
python -B tools/pack/event_transit_access.py REGION.osm.pbf OSRM_PORT B-full-result.json GTFS.zip
"""

import csv
import gzip
import hashlib
import io
import json
import math
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import urlopen
from xml.etree import ElementTree
from zipfile import ZipFile

from event_gate_walks import egress, indoor_routes

from shapely import LineString, Point, Polygon, STRtree, union_all
from shapely.geometry import shape

DATA = Path("data/mvp")
WEB = Path("web/data")
REGION_SHA = "43596705fc3af495df63c47ed66b10ac19e699a02d54867a7b1786a6049de529"


def read(file):
    with gzip.open(file) if file.suffix == ".gz" else file.open() as source:
        return json.load(source)


def metres(a, b):
    return math.hypot((a[0] - b[0]) * 92915, (a[1] - b[1]) * 110905)


def osm_stops(region):
    by_ref = defaultdict(list)
    doors = {}
    with tempfile.TemporaryDirectory() as tmp:
        selected = Path(tmp) / "stops.osm"
        subprocess.run(
            [
                "osmium",
                "tags-filter",
                region,
                "n/highway=bus_stop",
                "n/public_transport=platform",
                "n/railway=subway_entrance",
                "-o",
                selected,
            ],
            check=True,
        )
        for node in ElementTree.parse(selected).getroot().findall("node"):
            tags = {t.get("k"): t.get("v") for t in node.findall("tag")}
            point = [float(node.get("lon")), float(node.get("lat"))]
            nid = int(node.get("id"))
            if tags.get("railway") == "subway_entrance":
                doors[nid] = point
            elif tags.get("ref"):
                by_ref[tags["ref"]].append((nid, point))
    return by_ref, doors


def terminals(mode, stop, board, platforms, stations, doors):
    if mode == "bus":
        # The OSM platform with this official stop code is the curb where riders wait.
        if platforms[stop]:
            node, point = min(platforms[stop], key=lambda n: metres(n[1], board))
            yield {
                "id": stop,
                "osm_node": node,
                "source": "OSM platform",
                "position": point,
            }
        yield {"id": stop, "source": "GTFS stop", "position": board}
    else:
        for gate in stations[stop]:
            node = gate.get("osm")
            gate_id = gate.get("gtfs", f"osm:{node}")
            if node in doors:
                yield {
                    "id": gate_id,
                    "osm_node": node,
                    "source": "OSM entrance",
                    "position": doors[node],
                }
            if "gtfs" in gate:
                yield {
                    "id": gate_id,
                    "source": "GTFS entrance",
                    "position": gate["position"],
                }


def pedestrian_areas(region):
    with tempfile.TemporaryDirectory() as tmp:
        subset = Path(tmp) / "areas.pbf"
        subprocess.run(
            [
                "osmium",
                "tags-filter",
                region,
                "wr/highway=pedestrian,footway",
                "wr/area:highway=pedestrian,footway",
                "wr/public_transport=platform",
                "wr/railway=platform",
                "-o",
                subset,
            ],
            check=True,
        )
        rows = json.loads(
            subprocess.check_output(
                [
                    "osmium",
                    "export",
                    "--geometry-types",
                    "polygon",
                    "-f",
                    "geojson",
                    subset,
                ]
            )
        )["features"]
    for row in rows:
        t = row["properties"]
        if (
            t.get("foot", t.get("access")) in {"no", "private", "customers"}
            or t.get("indoor") == "yes"
            or t.get("location") == "underground"
            or t.get("layer", "").startswith("-")
        ):
            continue
        if (
            t.get("area") == "yes"
            and t.get("highway") in {"pedestrian", "footway"}
            or t.get("area:highway") in {"pedestrian", "footway"}
            or t.get("public_transport") == "platform"
            or t.get("railway") == "platform"
        ):
            yield shape(row["geometry"])


def foot_route(port, origin, dest):
    coords = ";".join(f"{p[0]},{p[1]}" for p in (origin, dest))
    try:
        with urlopen(
            f"http://127.0.0.1:{port}/route/v1/foot/{coords}?overview=full&geometries=geojson&annotations=nodes&radiuses=unlimited;250"
        ) as response:
            return json.load(response)["routes"][0]
    except HTTPError as error:
        if json.load(error)["code"] in {"NoSegment", "NoRoute"}:
            return None
        raise


def connected(point, dest, surface):
    if metres(point, dest) <= 0.15:
        return True
    connector = LineString([point, dest])
    hits = surface.query(connector, predicate="intersects")
    # The ground-surface polygon coordinates resolve one millionth of a degree.
    return len(hits) > 0 and union_all(surface.geometries.take(hits)).buffer(
        0.000001
    ).covers(connector)


def indoor_network(feed):
    with ZipFile(feed) as archive:

        def rows(name):
            return csv.DictReader(
                io.TextIOWrapper(archive.open(name + ".txt"), encoding="utf-8-sig")
            )

        graph = indoor_routes(rows("pathways"))
        areas = defaultdict(list)
        for stop in rows("stops"):
            if stop["location_type"] == "4":
                areas[stop["parent_station"]].append(stop["stop_id"])
    # Search backward from the boarding areas, retaining the allowed edge
    # directions and the published traversal times for the inward journey.
    reverse = defaultdict(list)
    for a, edges in graph.items():
        for b, time, pid in edges:
            reverse[b].append((a, time, pid))
    return reverse, areas


def ingress(reverse, areas, door, platforms):
    result = {}
    for platform in sorted(platforms):
        connection = egress(reverse, areas[platform], door)
        if connection:
            result[platform] = {
                "seconds": connection["seconds"],
                "boarding_area": connection["boarding_area"],
                "pathway_ids": list(reversed(connection["pathway_ids"])),
            }
    return result


def pack(region, port, result, feed):
    if hashlib.sha256(region.read_bytes()).hexdigest() != REGION_SHA:
        raise ValueError(
            "Use the pinned regional OSM extract from the local walking route pack"
        )
    transit_path = DATA / "event_transit_routes.json"
    transit = read(transit_path)
    if hashlib.sha256(feed.read_bytes()).hexdigest() != transit["source_sha256"]:
        raise ValueError("Use the same GTFS feed as the scheduled transit routes")
    reverse, areas = indoor_network(feed)
    with (DATA / "communes.csv").open() as f:
        centers = {
            r["cut"]: [float(r["longitude"]), float(r["latitude"])]
            for r in csv.DictReader(f)
        }
    platforms, doors = osm_stops(region)
    stations = read(WEB / "metro_entrances.json.gz")["stations"]
    ground = [
        shape(f["geometry"])
        for f in read(WEB / "road_surface.geojson.gz")["features"]
        if f["properties"]["use"] in {"foot", "sidewalk"}
        and f["properties"]["structure"] == "ground"
    ]
    ground.extend(pedestrian_areas(region))
    surface = STRtree(ground)
    boundaries = {
        f["properties"]["cut"]: shape(f["geometry"])
        for f in read(DATA / "communes.geojson")["features"]
    }
    move = next(
        ed
        for ed in result["city_edits"]
        if ed["op"] == "move" and ed["id"] == "mesh:152:317"
    )
    bowls = {
        venue: Polygon(shape(move[k]["geometry"]).exterior)
        for venue, k in [("monumental", "before"), ("mut", "after")]
    }
    road_graph = read(DATA / "road_graph.json")["ways"]
    closed = set()
    for ed in result["city_edits"]:
        if ed["kind"] == "road" and ed["op"] == "remove":
            for part in road_graph[ed["id"]]["parts"]:
                for a, b in zip(part, part[1:]):
                    if a.isdecimal() and b.isdecimal():
                        closed.update([(int(a), int(b)), (int(b), int(a))])
    stops = {}
    for venue, scene in transit["venues"].items():
        for cut, options in scene["communes"].items():
            for option in options:
                leg = option["legs"][0]
                key = cut, leg["board_stop_id"]
                if key not in stops:
                    stops[key] = (leg["mode"], option["board"], set(), set())
                stops[key][2].add(venue)
                if leg["mode"] == "metro":
                    stops[key][3].add(leg["board_platform_id"])
    routes, missing = defaultdict(dict), defaultdict(dict)
    for (cut, stop), (mode, board, venues, board_platforms) in sorted(stops.items()):
        choices = []
        failures = []
        for terminal in terminals(mode, stop, board, platforms, stations, doors):
            origin, dest = centers[cut], terminal["position"]
            route = foot_route(port, origin, dest)
            if not route:
                failures.append("no regional foot route")
                continue
            path = route["geometry"]["coordinates"]
            if not boundaries[cut].covers(Point(path[0])):
                failures.append("origin foot-network snap outside its comuna")
                continue
            gap = metres(path[-1], dest)
            if not connected(path[-1], dest, surface):
                failures.append("unmapped platform or entrance connection")
                continue
            if gap > 0.15:
                path.append(dest)
            else:
                path[-1] = dest
            line = LineString(path)
            if any(line.intersects(bowls[v]) for v in venues):
                failures.append("walk crosses the stadium footprint")
                continue
            nodes = route["legs"][0]["annotation"]["nodes"]
            if "mut" in venues and any(
                edge in closed for edge in zip(nodes, nodes[1:])
            ):
                failures.append("walk follows a closed stadium road")
                continue
            seconds = math.ceil(route["duration"] + gap / (4.8 / 3.6))
            indoor = ingress(reverse, areas, terminal["id"], board_platforms)
            if mode == "metro" and not indoor:
                failures.append("no directed GTFS entrance-to-platform path")
                continue
            total = seconds + min((r["seconds"] for r in indoor.values()), default=0)
            choices.append(
                (
                    (-len(indoor), total),
                    {
                        "mode": mode,
                        "origin_reference": origin,
                        "origin_snap_m": round(metres(origin, path[0]), 2),
                        "board": board,
                        "terminal": terminal,
                        "terminal_connector_m": round(gap, 2),
                        "path": path,
                        **({"station_ingress": indoor} if indoor else {}),
                        "seconds": seconds,
                        "metres": round(route["distance"] + gap, 1),
                    },
                )
            )
        if choices:
            routes[cut][stop] = min(choices, key=lambda c: c[0])[1]
        else:
            missing[cut][stop] = sorted(set(failures))
    return {
        "source": "© OpenStreetMap contributors, ODbL-1.0",
        "region_sha256": REGION_SHA,
        "transit_routes_sha256": hashlib.sha256(transit_path.read_bytes()).hexdigest(),
        "method": "Start on the foot network nearest the comuna population reference point. The walk excludes origin_snap_m. Use the shortest OSRM foot time among connected physical boarding points. A final connector must lie on mapped ground foot surfaces. The Metro path and seconds end at the exterior entrance. Add the selected platform’s station_ingress seconds from directed GTFS pathways before boarding.",
        "communes": routes,
        "unresolved": missing,
    }


if __name__ == "__main__":
    source = read(Path(sys.argv[3]))
    data = pack(
        Path(sys.argv[1]), sys.argv[2], source.get("result", source), Path(sys.argv[4])
    )
    dest = DATA / "event_transit_access.json.gz"
    dest.write_bytes(
        gzip.compress(
            json.dumps(data, ensure_ascii=False, separators=(",", ":")).encode(),
            mtime=0,
        )
    )
    print(
        "access walks",
        sum(map(len, data["communes"].values())),
        "unresolved",
        sum(map(len, data["unresolved"].values())),
        "bytes",
        dest.stat().st_size,
    )
