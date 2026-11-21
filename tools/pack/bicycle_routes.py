"""Export short inter-comuna bicycle routes from a local OSRM bicycle graph."""
import csv
import json
import math
from pathlib import Path
from urllib.request import urlopen

PACK = Path("data/mvp")
ROUTER = "http://127.0.0.1:5196/route/v1/bicycle"


def separation_m(a, b):
    lon1, lat1 = map(math.radians, a)
    lon2, lat2 = map(math.radians, b)
    h = math.sin((lat2 - lat1) / 2) ** 2
    h += math.cos(lat1) * math.cos(lat2) * math.sin((lon2 - lon1) / 2) ** 2
    return 2 * 6371008.8 * math.asin(math.sqrt(h))


def display_path(points):
    """Douglas-Peucker with a 4 m tolerance in a local metric projection."""
    x, y = points[0]
    scale = 111320 * math.cos(math.radians(y))
    dx, dy = (points[-1][0] - x) * scale, (points[-1][1] - y) * 110540
    length2 = dx * dx + dy * dy
    split, largest = 0, 4.0
    for index in range(1, len(points) - 1):
        px, py = (points[index][0] - x) * scale, (points[index][1] - y) * 110540
        t = max(0, min(1, (px * dx + py * dy) / length2)) if length2 else 0
        gap = math.hypot(px - t * dx, py - t * dy)
        if gap > largest:
            split, largest = index, gap
    if split:
        return display_path(points[:split + 1])[:-1] + display_path(points[split:])
    return [points[0], points[-1]]


with (PACK / "communes.csv").open() as file:
    centers = {r["cut"]: (float(r["longitude"]), float(r["latitude"]))
               for r in csv.DictReader(file)}

paths = {}
with (PACK / "bicycle_route_lengths.csv").open("w", newline="") as file:
    table = csv.writer(file)
    table.writerow(["origin_cut", "destination_cut", "direct_m", "route_m",
                    "origin_snap_m", "destination_snap_m"])
    for origin, a in centers.items():
        for destination, b in centers.items():
            direct = separation_m(a, b)
            if origin == destination or direct >= 15000:
                continue
            locations = f"{a[0]},{a[1]};{b[0]},{b[1]}"
            url = f"{ROUTER}/{locations}?overview=full&geometries=geojson"
            with urlopen(url) as response:
                answer = json.load(response)
            route = answer["routes"][0]
            gaps = [round(point["distance"], 1) for point in answer["waypoints"]]
            paths[f"{origin}-{destination}"] = display_path(route["geometry"]["coordinates"])
            table.writerow([origin, destination, round(direct, 1), route["distance"], *gaps])

output = Path("web/data/bicycle_routes.json")
output.write_text(json.dumps(paths, separators=(",", ":")) + "\n")
print(f"{len(paths)} directed bicycle routes, {output.stat().st_size:,} bytes")
