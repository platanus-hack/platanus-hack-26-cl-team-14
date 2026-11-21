"""Route the engine's aggregate walk, car and bicycle examples on local OSRM mode graphs."""
import gzip
import hashlib
import json
import math
import statistics
import sys
from collections import Counter
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import urlopen

from roads import rounded_path

SOURCE = Path(sys.argv[1])
PORTS = dict(zip(("walk", "car", "bicycle"), map(int, sys.argv[2:5])))
PROFILES = {"walk": "foot", "car": "driving", "bicycle": "cycling"}
MAX_SNAP_M = 250
OUT = Path("web/data/local_mover_routes.json.gz")


def signature(group):
    return (group["id"], group["mode"], tuple(group["origin"]), tuple(group["destination"]))


def query(mode, origin, destination):
    coords = ";".join(f"{lon},{lat}" for lon, lat in (origin, destination))
    url = (f"http://127.0.0.1:{PORTS[mode]}/route/v1/{PROFILES[mode]}/{coords}"
           f"?geometries=geojson&overview=full&radiuses={MAX_SNAP_M};{MAX_SNAP_M}")
    try:
        with urlopen(url) as response:
            data = json.load(response)
    except HTTPError as e:
        data = json.load(e)
        if data["code"] in {"NoRoute", "NoSegment"}:
            return data["code"], None
        raise
    route = data["routes"][0]
    path = rounded_path(route["geometry"]["coordinates"], 3)
    if len(path) < 2 or route["distance"] == 0:
        return "Stationary", None
    snaps = [round(p["distance"], 1) for p in data["waypoints"]]
    return "Routed", (path, snaps, round(route["distance"]))


data = json.loads(SOURCE.read_bytes())
scenes = {name: data["trips_" + name] for name in ("base", "scenario")}
selected = sorted({signature(g) for groups in scenes.values() for g in groups
                   if g["count"] >= 2 and g["mode"] in PORTS})
results, paths, groups = {}, [], []
for group_id, mode, origin, destination in selected:
    key = mode, origin, destination
    if key not in results:
        status, detail = query(*key)
        if detail:
            path, snaps, length = detail
            results[key] = (status, len(paths), snaps, length)
            paths.append(path)
        else:
            results[key] = (status, None, None, None)
    status, index, snaps, length = results[key]
    if status == "Routed":
        groups.append({"id": group_id, "mode": mode, "origin": origin, "destination": destination,
                       "route": index, "snap_m": snaps, "length_m": length})
content = {"source_sha256": hashlib.sha256(SOURCE.read_bytes()).hexdigest(), "max_snap_m": MAX_SNAP_M,
           "attribution": "© OpenStreetMap contributors", "license": "ODbL-1.0", "groups": groups, "paths": paths}
raw = json.dumps(content, separators=(",", ":"), ensure_ascii=False).encode() + b"\n"
with OUT.open("wb") as base, gzip.GzipFile(filename="", fileobj=base, mode="wb", compresslevel=6, mtime=0) as gz:
    gz.write(raw)
for scene, rows in scenes.items():
    counts = Counter()
    for g in rows:
        if g["count"] < 2 or g["mode"] not in PORTS:
            continue
        key = signature(g)[1:]
        counts[g["mode"], results[key][0], g["origin_cut"] == g["destination_cut"]] += 1
    print(scene, sorted(counts.items()))
for mode in PORTS:
    gaps = sorted(d for key, (status, _, snaps, _) in results.items() if key[0] == mode and snaps for d in snaps)
    if gaps:
        print(mode, "snap metres median / p95 / max:", statistics.median(gaps), gaps[math.ceil(len(gaps)*0.95)-1], gaps[-1])
print(f"{len(selected)} input signatures, {len(groups)} route matches, {len(paths)} paths, "
      f"{sum(map(len, paths))} points, {len(raw)} raw bytes, {OUT.stat().st_size} gzip bytes")
