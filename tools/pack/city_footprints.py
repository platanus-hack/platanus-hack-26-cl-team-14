"""Pack OSM building outlines and tagged heights for the central Santiago view."""
import json
import math
import re
from pathlib import Path

SOURCE = Path("data/raw/osm-buildings-city/polygons.geojsonseq")
OUTPUT = Path("web/data/city_footprints.json")
# From the city centre east through Providencia and the MUT site.
BOUNDS = (-70.663, -33.448, -70.583, -33.408)
X_METRES = 111_320 * math.cos(math.radians(-33.43))
Y_METRES = 110_540


def ring_area(ring):
    x, y = ring[0]
    return abs(sum((ax-x)*(by-y) - (bx-x)*(ay-y)
                   for (ax, ay), (bx, by) in zip(ring, ring[1:]))) * X_METRES * Y_METRES / 2


def round_ring(ring):
    rounded = []
    for x, y in ring:
        point = [round(x, 5), round(y, 5)]
        if not rounded or point != rounded[-1]:
            rounded.append(point)
    return rounded


def numeric(text, units=False):
    pattern = r"(\d+(?:[.,]\d+)?)\s*(m|ft)?" if units else r"(\d+(?:[.,]\d+)?)"
    match = re.fullmatch(pattern, text.strip())
    if match:
        value = float(match[1].replace(",", "."))
        if units and match[2] == "ft":
            value *= 0.3048
        if value > 0:
            return round(value, 2)


def main():
    buildings = []
    area = 0
    west, south, east, north = BOUNDS
    with SOURCE.open() as rows:
        for row in rows:
            feature = json.loads(row.lstrip("\x1e"))
            tags = feature["properties"]
            if tags.get("building", "no") == "no":
                continue
            for polygon in feature["geometry"]["coordinates"]:
                if not all(west <= x <= east and south <= y <= north for x, y in polygon[0]):
                    continue
                shell, *holes = map(round_ring, polygon)
                shell_area = ring_area(shell)
                if not shell_area:
                    continue
                holes = [ring for ring in holes if ring_area(ring)]
                area += shell_area - sum(map(ring_area, holes))
                building = {"polygon": [shell, *holes]}
                if height := numeric(tags.get("height", ""), units=True):
                    building["height_m"] = height
                if levels := numeric(tags.get("building:levels", "")):
                    building["levels"] = levels
                buildings.append(building)
    collection = {"credit": "© OpenStreetMap contributors", "license": "ODbL-1.0",
                  "bounds": BOUNDS, "default_height_m": 6, "metres_per_level": 3.5,
                  "buildings": buildings}
    OUTPUT.write_text(json.dumps(collection, ensure_ascii=False, separators=(",", ":")) + "\n")
    tagged = sum("height_m" in b for b in buildings)
    floors_only = sum("height_m" not in b and "levels" in b for b in buildings)
    print(f"{len(buildings)} polygons, {area/1e6:.3f} km² footprint, "
          f"{tagged} tagged heights, {floors_only} levels only, "
          f"{len(buildings)-tagged-floors_only} default heights, {OUTPUT.stat().st_size} bytes")


if __name__ == "__main__":
    main()
