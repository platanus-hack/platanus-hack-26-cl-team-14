"""Build the Cut 6 car graph from the original members of each displayed OSM way."""
import csv
import hashlib
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from tempfile import TemporaryDirectory

from car_paths import SPEEDS
from roads import PACK, VIEW, clipped_segments, distance, read_roads, rounded_path, write_json

CAR_ACCESS = ("motorcar", "motor_vehicle", "vehicle", "access")
PUBLIC = {"yes", "permissive", "designated", "official"}


def travel_direction(tags):
    for key in ("oneway:motorcar", "oneway:motor_vehicle", "oneway:vehicle", "oneway"):
        if key in tags:
            value = tags[key].lower()
            if value in {"yes", "true", "1"}:
                return "yes", key
            if value in {"no", "false", "0"}:
                return "no", key
            if value == "-1":
                return "-1", key
            return "unknown", key
    if tags.get("junction") == "roundabout" or tags["highway"] == "motorway":
        return "yes", "osm_implied"
    return "no", "assumed_two_way"


def car_allowed(tags, direction):
    for key in CAR_ACCESS:
        for field in (f"{key}:{direction}", key):
            if field in tags:
                return tags[field].lower() in PUBLIC
    return True


def posted_cap(tags, direction):
    for key in ("maxspeed:" + direction, "maxspeed"):
        value = tags.get(key, "").strip().lower()
        match = re.fullmatch(r"(\d+(?:\.\d+)?)\s*(km/h|kmh|kph|mph)?", value)
        if match:
            kmh = float(match[1]) * (1.609344 if match[2] == "mph" else 1)
            if kmh > 0:
                return round(kmh, 3), key
    return None, None


def clipped_parts(way_id, members, points):
    """Keep original node IDs inside the box and unique IDs where each segment is cut."""
    part = []
    for i, (first, last) in enumerate(zip(members, members[1:])):
        if first == last:
            continue
        a, b = points[first], points[last]
        clipped = next(clipped_segments((a, b)), None)
        if not clipped:
            if part:
                yield part
                part = []
            continue
        start, end = clipped
        left = str(first) if start == a else f"clip/{way_id}/{i}/start"
        right = str(last) if end == b else f"clip/{way_id}/{i}/end"
        if part and left != part[-1][2]:
            yield part
            part = []
        part.append((i, left, right, start, end))
    if part:
        yield part


def commune_anchors(candidates, nodes):
    anchors, gaps = {}, {}
    west, south, east, north = VIEW
    candidates = sorted(candidates, key=int)
    with (PACK / "communes.csv").open() as source:
        for row in csv.DictReader(source):
            cut = row["cut"]
            lon, lat = float(row["longitude"]), float(row["latitude"])
            if west <= lon <= east and south <= lat <= north:
                node = min(candidates, key=lambda n: distance((lon, lat), nodes[n]))
                anchors[cut] = node
                gaps[cut] = round(distance((lon, lat), nodes[node]), 1)
            else:
                anchors[cut] = gaps[cut] = None
    return anchors, gaps


def source_record(pbf):
    sha256, md5 = hashlib.sha256(), hashlib.md5()
    with pbf.open("rb") as source:
        for chunk in iter(lambda: source.read(1 << 20), b""):
            sha256.update(chunk)
            md5.update(chunk)
    digest = sha256.hexdigest()
    if md5.hexdigest() == "fef09ad7ee6f28c7afc925ef68c737a4":
        provenance = "geofabrik_chile_published_md5"
    elif digest == "4a1cfdb3ffdfdb58cbc6247ab36cac38ad2dc35509b5c1e102a7e53461234cb3":
        provenance = "cached_urban_extract_sha256"
    else:
        provenance = "unverified_snapshot_date"
    return {"sha256": digest, "bytes": pbf.stat().st_size, "provenance": provenance,
            "osm_timestamp": None if provenance == "unverified_snapshot_date" else "2026-11-20T21:21:21Z"}


def build(pbf):
    visible = defaultdict(list)
    for feature in json.loads((PACK / "roads.geojson").read_bytes())["features"]:
        visible[feature["properties"]["id"]].append(feature["geometry"]["coordinates"])
    with TemporaryDirectory() as tmp:
        ids = Path(tmp) / "ways.txt"
        ids.write_text("".join(f"w{way}\n" for way in sorted(visible)))
        raw = Path(tmp) / "ways.osm"
        subprocess.run(["osmium", "getid", "-r", "-i", ids, "-o", raw, pbf], check=True)
        points, source = read_roads(raw)

    nodes, ways, anchor_nodes = {}, {}, set()
    for number, tags, members in source:
        way_id = str(number)
        oneway, direction_source = travel_direction(tags)
        direction_speeds, edit_properties = {}, {}
        assumed = SPEEDS[tags["highway"]]
        for suffix, direction, permitted in (("f", "forward", {"no", "yes"}),
                                             ("r", "backward", {"no", "-1"})):
            cap, cap_source = posted_cap(tags, direction)
            access = car_allowed(tags, direction)
            edit_properties["speed_cap_" + direction + "_kmh"] = cap
            edit_properties["car_access_" + direction] = access
            if oneway in permitted and access:
                if cap is not None and cap <= assumed:
                    direction_speeds[suffix] = cap, cap_source
                else:
                    direction_speeds[suffix] = assumed, "class_assumption"
        parts, edges, drawn = [], [], []
        for part in clipped_parts(way_id, members, points):
            display = rounded_path([part[0][3], *(segment[4] for segment in part)], 8)
            if len(display) < 2:
                continue
            drawn.append(display)
            parts.append([part[0][1], *(segment[2] for segment in part)])
            for index, left, right, a, b in part:
                nodes[left], nodes[right] = a, b
                length = round(distance(a, b), 3)
                for suffix, (kmh, provenance) in direction_speeds.items():
                    start, end = (left, right) if suffix == "f" else (right, left)
                    edges.append({"id": f"{way_id}/{index}/{suffix}", "from": start, "to": end,
                                  "length_m": length, "speed_kmh": kmh, "speed_source": provenance})
        if drawn != visible[number]:
            raise ValueError(f"Clipped geometry differs from roads.geojson on OSM way {way_id}")
        if tags["highway"] in {"primary", "secondary"}:
            anchor_nodes.update(e[k] for e in edges for k in ("from", "to") if e[k].isdecimal())
        ways[way_id] = {"highway": tags["highway"], "class": tags["highway"].removesuffix("_link"), "name": tags.get("name", ""),
                        "oneway": oneway, "oneway_source": direction_source, **edit_properties, "parts": parts, "edges": edges}
    missing = set(map(str, visible)) - ways.keys()
    if missing:
        raise ValueError(f"Source lacks {len(missing)} displayed OSM ways; use the pinned Chile extract")
    anchors, gaps = commune_anchors(anchor_nodes, nodes)
    output = PACK / "road_graph.json"
    write_json(output, {"attribution": "© OpenStreetMap contributors", "license": "ODbL-1.0",
                        "source": source_record(pbf), "nodes": nodes, "ways": ways,
                        "anchors": anchors, "anchor_distance_m": gaps})
    print(f"{len(ways)} ways, {sum(len(w['parts']) for w in ways.values())} parts, {len(nodes)} nodes, "
          f"{sum(len(w['edges']) for w in ways.values())} directed edges, {output.stat().st_size} bytes")


if __name__ == "__main__":
    build(Path(sys.argv[1]))
