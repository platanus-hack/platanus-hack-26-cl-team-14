"""Route aggregate public groups whose comuna pair has no path in public_routes.json."""
import gzip
import hashlib
import heapq
import json
import sys
from collections import Counter
from functools import lru_cache
from pathlib import Path

from public_routes import route_record, router


@lru_cache(None)
def access(point):
    lon, lat = point
    cx, cy = int(lon/router.CELL), int(lat/router.CELL)
    near = {}
    # Three cells cover 1 km in both axes throughout the Santiago region.
    for x in range(cx-3, cx+4):
        for y in range(cy-3, cy+4):
            for stop in router.grid.get((x, y), []):
                distance = router.metres(lon, lat, *router.coords[stop])
                if distance <= router.ACCESS_M:
                    near[("s", stop)] = distance / router.WALK_MPS
    return near


def route(origin, destination):
    starts, ends = access(origin), access(destination)
    if not starts or not ends:
        return None, "no_access"
    parents = {}
    queue = []
    order = 0
    for node, cost in sorted(starts.items(), key=lambda item: item[1]):
        heapq.heappush(queue, (cost, order, node, None))
        order += 1
    winner, total = None, float("inf")
    while queue:
        cost, _, node, parent = heapq.heappop(queue)
        if cost > total:
            break
        if node in parents:
            continue
        parents[node] = parent
        if node in ends and cost + ends[node] < total:
            winner, total = node, cost + ends[node]
        for target, edge, _, _ in router.graph[node]:
            if target not in parents:
                heapq.heappush(queue, (cost + edge, order, target, node))
                order += 1
    if winner is None:
        return None, "disconnected"
    chain = []
    while winner is not None:
        chain.append(winner)
        winner = parents[winner]
    record = route_record(origin, destination, chain[::-1])
    if not any(leg["kind"] == "ride" for leg in record["segments"]):
        return None, "walk_only"
    return record, "routed"


fixture_path = Path(sys.argv[1])
fixture_bytes = fixture_path.read_bytes()
fixture = json.loads(fixture_bytes)
existing = json.loads(Path("web/data/public_routes.json").read_bytes())
paths = []
groups = {"base": {}, "scenario": {}}
by_endpoints = {}
counts = Counter()
for side, ids in groups.items():
    for group in fixture["trips_" + side]:
        if group["mode"] != "public" or group["count"] <= 1:
            continue
        if group["origin_cut"] + "-" + group["destination_cut"] in existing:
            continue
        endpoints = tuple(group["origin"]), tuple(group["destination"])
        if endpoints not in by_endpoints:
            record, reason = route(*endpoints)
            index = None
            if record is not None:
                index = len(paths)
                paths.append(record)
            by_endpoints[endpoints] = index, reason
        index, reason = by_endpoints[endpoints]
        counts[side, reason] += 1
        if index is not None:
            ids[group["id"]] = index
out = Path("web/data/local_public_routes.json.gz")
data = {"fixture_sha256": hashlib.sha256(fixture_bytes).hexdigest(), "groups": groups, "paths": paths}
out.write_bytes(gzip.compress(json.dumps(data, separators=(",", ":")).encode(), mtime=0))
print(dict(counts), f"{len(paths)} distinct paths, {out.stat().st_size:,} gzip bytes")
