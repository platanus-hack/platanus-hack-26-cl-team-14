"""Write the Mapocho boat line (two directed `line add` changes) along the river centerline.

Piers: context/research/mapocho/piers.json (assumed positions). River: OSM relation 7548664, main_stream ways
(ODbL), fetched from Overpass. Each pier snaps to the nearest point of the centerline; segment minutes use the
distance along the river at 17 km/h. Usage: python3 -I tools/pack/mapocho_boat_line.py
"""
import json
import math
import urllib.parse
import urllib.request
from pathlib import Path

SPEED_KMH = 17  # Brisbane CityCat mean operating speed (context/research/boat-line-entity.md)
HEADWAY, CAPACITY, DWELL = 10, 162, 1
OUT = Path("context/research/mapocho/boat_line.json")
QUERY = "[out:json][timeout:120];rel(7548664);out body;way(r);out geom;"


def metres(a, b):
    lat = math.radians((a[1] + b[1]) / 2)
    return math.hypot((a[0] - b[0]) * math.cos(lat) * 111_320, (a[1] - b[1]) * 110_574)


def centerline():
    body = urllib.parse.urlencode({"data": QUERY}).encode()
    data = json.load(urllib.request.urlopen(urllib.request.Request(
        "https://overpass-api.de/api/interpreter", body, {"User-Agent": "team14-sim/1.0"}), timeout=180))
    rel = next(e for e in data["elements"] if e["type"] == "relation")
    geom = {e["id"]: [(p["lon"], p["lat"]) for p in e["geometry"]] for e in data["elements"] if e["type"] == "way"}
    parts = [geom[m["ref"]] for m in rel["members"] if m["type"] == "way" and m["role"] == "main_stream"]
    # Stitch the main-stream ways end to end (the relation lists them in flow order, some reversed).
    line = list(parts.pop(0))
    while parts:
        best = min(((min(metres(line[-1], p[0]), metres(line[-1], p[-1]), metres(line[0], p[0]), metres(line[0], p[-1])), i)
                    for i, p in enumerate(parts)))[1]
        p = parts.pop(best)
        ends = {"tail_head": metres(line[-1], p[0]), "tail_tail": metres(line[-1], p[-1]),
                "head_tail": metres(line[0], p[-1]), "head_head": metres(line[0], p[0])}
        case = min(ends, key=ends.get)
        if case == "tail_head": line += p[1:]
        elif case == "tail_tail": line += p[::-1][1:]
        elif case == "head_tail": line = p[:-1] + line
        else: line = p[::-1][:-1] + line
    return line


def snap(line, point):
    """Index of the nearest centerline vertex and the distance along the line to it."""
    along, best = 0.0, (float("inf"), 0, 0.0)
    for i, v in enumerate(line):
        if i: along += metres(line[i - 1], v)
        d = metres(v, point)
        if d < best[0]: best = (d, i, along)
    return best


def change(piers, line, snaps, lid):
    stops = [p["id"] for p in piers]
    seg = [round(abs(snaps[b][2] - snaps[a][2]) / 1000 / SPEED_KMH * 60, 2) for a, b in zip(stops, stops[1:])]
    i, j = snaps[stops[0]][1], snaps[stops[-1]][1]
    path = line[i:j + 1] if i <= j else line[j:i + 1][::-1]
    return {"kind": "line", "op": "add", "target": {"type": "line", "id": lid},
            "geometry": {"type": "LineString", "coordinates": [[round(x, 6), round(y, 6)] for x, y in path]},
            "properties": {"mode": "boat", "stop_ids": stops,
                           "new_stops": {p["id"]: [p["longitude"], p["latitude"]] for p in piers},
                           "segment_minutes": seg, "dwell_minutes": [0] + [DWELL] * (len(stops) - 2) + [0],
                           "headway_minutes": HEADWAY, "pickup_type": [0] * len(stops), "drop_off_type": [0] * len(stops),
                           "vehicle_capacity": CAPACITY}}


def main():
    piers = [c["target"] for c in json.load(open("context/research/mapocho/piers.json"))]
    line = centerline()
    snaps = {p["id"]: snap(line, (p["longitude"], p["latitude"])) for p in piers}
    for p in piers:
        print(p["id"], f"{snaps[p['id']][0]:.0f} m from the centerline, {snaps[p['id']][2] / 1000:.2f} km along")
    changes = [change(piers, line, snaps, "MAPOCHO-E"), change(piers[::-1], line, snaps, "MAPOCHO-W")]
    print("end to end minutes", sum(changes[0]["properties"]["segment_minutes"]))
    OUT.write_text(json.dumps(changes, indent=1) + "\n")


if __name__ == "__main__":
    main()
