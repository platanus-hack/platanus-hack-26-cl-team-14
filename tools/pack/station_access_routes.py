"""Map station-entry walk origins to real entrances on a local OSRM foot graph."""
import argparse
import gzip
import hashlib
import json
from collections import Counter
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import urlopen


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("snapshots", nargs="+", type=Path)
parser.add_argument("--port", type=int, default=19443)
parser.add_argument("--stations", nargs="+", default=["TB-TOB", "GO", "LE-LEN"])
parser.add_argument("--pbf", type=Path, required=True)
parser.add_argument("--profile", type=Path, required=True)
parser.add_argument("--plan", action="store_true", help="Print workload without routing")
args = parser.parse_args()
sha = lambda data: hashlib.sha256(data).hexdigest()
entrance_bytes = Path("web/data/metro_entrances.json.gz").read_bytes()
entrances = json.loads(gzip.decompress(entrance_bytes))["stations"]
sources, selected = [], set()
for file in args.snapshots:
    raw = file.read_bytes()
    scene = json.loads(gzip.decompress(raw) if file.suffix == ".gz" else raw)
    sources.append({"file": file.name, "sha256": sha(raw), "revision": scene["revision"]})
    for station in scene["stations"]:
        if station["id"] not in args.stations:
            continue
        for half in ("home", "job"):
            for lon, lat, mode, share in station["access_origins"][half]:
                if mode == "walk" and share > 0:
                    selected.add((station["id"], lon, lat))

# Duplicate entrance coordinates share geometry, but retain their source IDs.
doors = {}
for station in args.stations:
    doors[station] = {}
    for entrance in entrances[station]:
        doors[station].setdefault(tuple(entrance["position"]), []).append(entrance["gtfs"])
print(f"{len(selected)} walk origins; at most {sum(len(doors[s]) for s, _, _ in selected)} route queries", flush=True)
if args.plan:
    raise SystemExit


def query(origin, destination):
    coords = ";".join(f"{lon},{lat}" for lon, lat in (origin, destination))
    url = (f"http://127.0.0.1:{args.port}/route/v1/foot/{coords}"
           "?geometries=geojson&overview=full&radiuses=250;250")
    try:
        with urlopen(url) as response:
            data = json.load(response)
    except HTTPError as error:
        data = json.load(error)
        if data["code"] not in {"NoRoute", "NoSegment"}:
            raise
    if data["code"] != "Ok":
        return {"status": data["code"]}
    route = data["routes"][0]
    return {"status": "Routed", "path": route["geometry"]["coordinates"],
            "length_m": route["distance"], "duration_s": route["duration"],
            "snap_m": [waypoint["distance"] for waypoint in data["waypoints"]]}


cache, paths, routes = {}, [], []
for station, lon, lat in sorted(selected):
    origin = (lon, lat)
    candidates = []
    failures = []
    for destination, ids in doors[station].items():
        key = origin, destination
        if key not in cache:
            cache[key] = query(*key)
        result = cache[key]
        if result["status"] == "Routed":
            candidates.append((result["length_m"], ids, result))
        else:
            failures.append({"entrances": ids, "status": result["status"]})
    row = {"station": station, "origin": origin}
    if candidates:
        _, ids, result = min(candidates, key=lambda candidate: (candidate[0], candidate[1]))
        row.update({key: value for key, value in result.items() if key != "path"})
        row.update(entrance=ids[0], entrance_ids=ids, route=len(paths))
        paths.append(result["path"])
    else:
        row.update(status="Unrouted", entrances=failures)
    routes.append(row)

pack = {"sources": sources, "entrances_sha256": sha(entrance_bytes),
        "graph": {"pbf_sha256": sha(args.pbf.read_bytes()), "profile_sha256": sha(args.profile.read_bytes())},
        "attribution": "© OpenStreetMap contributors", "license": "ODbL-1.0", "max_snap_m": 250,
        "door_choice": "Shortest returned OSRM foot route among mapped entrances; modeled, not observed.",
        "routes": routes, "paths": paths}
raw = json.dumps(pack, separators=(",", ":"), ensure_ascii=False).encode() + b"\n"
out = Path("web/data/station_access_routes.json.gz")
with out.open("wb") as file, gzip.GzipFile(filename="", fileobj=file, mode="wb", mtime=0) as compressed:
    compressed.write(raw)
print(dict(Counter((row["station"], row["status"]) for row in routes)))
print(f"{len(cache)} queries, {len(paths)} paths, {out.stat().st_size} gzip bytes, SHA256 {sha(out.read_bytes())}")
