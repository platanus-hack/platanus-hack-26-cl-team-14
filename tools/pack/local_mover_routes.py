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
# Manual review found regional walking detours of 126.6 km and 92.3 km.
EXCLUDED_WALKS = {
    (1648, "walk", (-71.327463, -33.799936), (-71.287494, -33.711995)),
    (1688, "walk", (-70.905731, -33.400158), (-70.846571, -33.449351)),
}
WALK_ONLY = "--walk-only" in sys.argv[6:]
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


source = SOURCE.read_bytes()
source_sha = hashlib.sha256(source).hexdigest()
baseline = json.loads(gzip.decompress(Path(sys.argv[5]).read_bytes())) if len(sys.argv) > 5 else None
data = json.loads(source)
scenes = {name: data["trips_" + name] for name in ("base", "scenario")}
selected = sorted({signature(g) for groups in scenes.values() for g in groups
                   if g["count"] >= 2 and g["mode"] in PORTS})
results, paths, groups = {}, [], []
if baseline:
    if baseline["max_snap_m"] != MAX_SNAP_M:
        raise ValueError("Baseline must use the same snap radius")
    if baseline["source_sha256"] != source_sha and not WALK_ONLY:
        raise ValueError("A new scenario export requires --walk-only")
    paths.extend(baseline["paths"])
    groups.extend(baseline["groups"])
    known = {signature(g) for g in groups}
    modes = {"walk"} if WALK_ONLY else {"walk", "bicycle"}
    selected = [s for s in selected if s not in known and s[1] in modes]
for group_id, mode, origin, destination in selected:
    key = mode, origin, destination
    if (group_id, mode, origin, destination) in EXCLUDED_WALKS:
        results[key] = ("ReviewedDetour", None, None, None)
        continue
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
content = {"source_sha256": baseline["source_sha256"] if baseline else source_sha, "max_snap_m": MAX_SNAP_M,
           "attribution": "© OpenStreetMap contributors", "license": "ODbL-1.0", "groups": groups, "paths": paths}
if baseline and (WALK_ONLY or "walk_extension_sources" in baseline):
    sources = baseline.get("walk_extension_sources", [])
    content["walk_extension_sources"] = sources + ([source_sha] if WALK_ONLY and source_sha not in sources else [])
raw = json.dumps(content, separators=(",", ":"), ensure_ascii=False).encode() + b"\n"
with OUT.open("wb") as base, gzip.GzipFile(filename="", fileobj=base, mode="wb", compresslevel=6, mtime=0) as gz:
    gz.write(raw)
routed = {signature(row) for row in groups}
for scene, rows in scenes.items():
    counts = Counter()
    for g in rows:
        if g["count"] < 2 or g["mode"] not in PORTS:
            continue
        key = signature(g)[1:]
        status = "Routed" if signature(g) in routed else results.get(key, ("Missing",))[0]
        counts[g["mode"], status, g["origin_cut"] == g["destination_cut"]] += 1
    print(scene, sorted(counts.items()))
for mode in PORTS:
    gaps = sorted(d for key, (status, _, snaps, _) in results.items() if key[0] == mode and snaps for d in snaps)
    if gaps:
        print(mode, "snap metres median / p95 / max:", statistics.median(gaps), gaps[math.ceil(len(gaps)*0.95)-1], gaps[-1])
print(f"{len(selected)} selected signatures, {len(groups)} route matches, {len(paths)} paths, "
      f"{sum(map(len, paths))} points, {len(raw)} raw bytes, {OUT.stat().st_size} gzip bytes")
