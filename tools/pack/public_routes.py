"""Export the winning 08:00 GTFS path for each reachable pair of comuna centres."""
import json
import math
from array import array
from collections import defaultdict
from functools import lru_cache
from pathlib import Path

import travel_times as router


X_METRES, Y_METRES = 92900, 110574
trips = {r["trip_id"]: r for r in router.table("trips.txt")}
routes = {r["route_id"]: r["route_type"] for r in router.table("routes.txt")}
patterns = list(router.pattern_wait)
trip_for_pattern = []
for stops in patterns:
    candidates = router.patterns[stops]
    active = [tid for tid in candidates if tid in router.headways]
    if active:
        # The model uses the smallest published 08:00 headway for this stop pattern.
        selected = min(active, key=lambda tid: min(router.headways[tid]))
    else:
        selected = min((tid for tid in candidates if router.DEPART <= router.sequence[tid][0][3]
                        < router.DEPART + 3600), key=lambda tid: router.sequence[tid][0][3])
    trip_for_pattern.append(trips[selected])

shape_points = defaultdict(list)
for row in router.table("shapes.txt"):
    shape_points[row["shape_id"]].append((int(row["shape_pt_sequence"]),
                                          (float(row["shape_pt_lon"]), float(row["shape_pt_lat"]))))
shapes = {key: [point for _, point in sorted(rows)] for key, rows in shape_points.items()}
del shape_points


@lru_cache(None)
def shape_indices(pattern):
    """Match ordered stops to ordered vertices with minimum total squared distance."""
    shape = shapes[trip_for_pattern[pattern]["shape_id"]]
    costs = [0] * len(shape)
    back = []
    for stop in patterns[pattern]:
        lon, lat = router.coords[stop]
        previous = array("H")
        best = 0
        next_costs = []
        for j, (x, y) in enumerate(shape):
            if costs[j] < costs[best]:
                best = j
            previous.append(best)
            next_costs.append(costs[best] + ((x-lon)*X_METRES)**2 + ((y-lat)*Y_METRES)**2)
        back.append(previous)
        costs = next_costs
    j = min(range(len(shape)), key=costs.__getitem__)
    indices = []
    for previous in reversed(back):
        indices.append(j)
        j = previous[j]
    # A Metro shape can run several kilometres between two vertices. Place each
    # platform on an adjacent segment so a boarding does not double back to a vertex.
    positions = []
    for stop, j in zip(patterns[pattern], reversed(indices)):
        lon, lat = router.coords[stop]
        options = []
        for k in range(max(0, j-1), min(len(shape)-1, j+1)):
            x, y = shape[k]
            dx = (shape[k+1][0]-x)*X_METRES
            dy = (shape[k+1][1]-y)*Y_METRES
            px, py = (lon-x)*X_METRES, (lat-y)*Y_METRES
            length2 = dx*dx + dy*dy
            t = max(0, min(1, (px*dx + py*dy)/length2)) if length2 else 0
            options.append(((px-t*dx)**2 + (py-t*dy)**2, k+t))
        position = min(options)[1]
        positions.append(max(position, positions[-1]) if positions else position)
    return positions


def simplify(points):
    """Douglas-Peucker in local metres. Preserve the endpoints of each leg."""
    if len(points) <= 2:
        return points
    (lon, lat), (end_lon, end_lat) = points[0], points[-1]
    dx, dy = (end_lon-lon)*X_METRES, (end_lat-lat)*Y_METRES
    length2 = dx*dx + dy*dy
    split, worst = 0, 25**2
    for i, (x, y) in enumerate(points[1:-1], start=1):
        x, y = (x-lon)*X_METRES, (y-lat)*Y_METRES
        t = max(0, min(1, (x*dx + y*dy)/length2)) if length2 else 0
        error = (x-t*dx)**2 + (y-t*dy)**2
        if error > worst:
            split, worst = i, error
    if split:
        return simplify(points[:split+1])[:-1] + simplify(points[split:])
    return [points[0], points[-1]]


@lru_cache(None)
def ride(pattern, first, last):
    stops = patterns[pattern]
    shape = shapes.get(trip_for_pattern[pattern]["shape_id"])
    if shape:
        positions = shape_indices(pattern)
        start, end = positions[first], positions[last]
        points = [router.coords[stops[first]], *shape[math.ceil(start):math.floor(end)+1],
                  router.coords[stops[last]]]
    else:
        points = [router.coords[stop] for stop in stops[first:last+1]]
    return simplify(points)


paths = {}


def route_record(origin, destination, chain):
    path = [[round(value, 5) for value in origin]]
    segments = []
    modes = {"metro_lines": [], "bus_routes": []}

    def add_segment(mode, service, kind, points):
        start = len(path) - 1
        for lon, lat in points:
            point = [round(lon, 5), round(lat, 5)]
            if path[-1] != point:
                path.append(point)
        segments.append({"start": start, "end": len(path)-1, "mode": mode, "service": service, "kind": kind})

    add_segment("walk", None, "access", [router.coords[chain[0][1]]])
    i = 0
    while i < len(chain):
        node = chain[i]
        if node[0] == "s":
            # A run of street nodes represents one walk between GTFS stops.
            start = i
            while i < len(chain) and chain[i][0] == "s":
                i += 1
            if i > start + 1:
                add_segment("walk", None, "transfer", (router.coords[n[1]] for n in chain[start+1:i]))
            continue
        pattern, first = node[1:]
        last = first
        while i < len(chain) and chain[i][0] != "s":
            last = chain[i][2]
            i += 1
        if last == first:
            continue
        route = trip_for_pattern[pattern]["route_id"]
        mode, field = {"1": ("metro", "metro_lines"), "3": ("bus", "bus_routes"),
                       "0": ("rail", "rail_routes")}[routes[route]]
        add_segment(mode, route, "ride", ride(pattern, first, last))
        values = modes.setdefault(field, [])
        if route not in values:
            values.append(route)
    add_segment("walk", None, "egress", [destination])
    return {"path": path, **modes, "segments": segments}


def add_path(origin, destination, chain):
    paths[f"{origin}-{destination}"] = route_record(router.points[origin], router.points[destination], chain)


if __name__ == "__main__":
    rows = router.solve(add_path)
    out = Path("web/data/public_routes.json")
    out.write_text(json.dumps(paths, separators=(",", ":")) + "\n", encoding="utf8")
    print(f"\n{len(rows)} routes, {out.stat().st_size:,} bytes")
