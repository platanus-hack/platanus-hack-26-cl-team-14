"""Pack OSM destinations from a point and polygon export of the urban view."""
import csv
import json
import math
import sys
from pathlib import Path

from shapely.geometry import box, shape
from shapely.strtree import STRtree

URBAN = box(-70.85, -33.65, -70.45, -33.30)
CLOSED = {"no", "vacant", "empty", "closed", "disused", "abandoned", "construction"}


def activity(tags):
    for key in ("healthcare", "amenity"):
        if tags.get(key) in {"hospital", "clinic"}:
            return tags[key]
    if tags.get("office") == "government" or tags.get("amenity") == "townhall":
        return "public_office"
    shop = tags.get("shop")
    if shop and shop not in CLOSED:
        return shop if shop in {"mall", "supermarket"} else "shop"


def main(source):
    features = json.loads(Path("data/mvp/communes.geojson").read_text())["features"]
    boundaries = STRtree([shape(feature["geometry"]) for feature in features])
    columns = ["kind", "name", "longitude", "latitude", "cut", "mapped_area_m2",
               "building_levels", "osm_id"]
    missing_cut = 0
    with open(source) as rows, open("data/mvp/activity_points.csv", "w", newline="") as out:
        writer = csv.writer(out, lineterminator="\n")
        writer.writerow(columns)
        for row in rows:
            feature = json.loads(row.lstrip("\x1e"))
            tags = feature["properties"]
            kind = activity(tags)
            if not kind:
                continue
            geometry = shape(feature["geometry"])
            point = geometry.representative_point()
            if not URBAN.covers(point):
                continue
            matches = boundaries.query(point, predicate="intersects")
            if not len(matches):
                missing_cut += 1
                continue
            cut = min(features[i]["properties"]["cut"] for i in matches)
            area = ""
            osm_id = feature["id"]
            if osm_id.startswith("a"):
                area_id = int(osm_id[1:])
                osm_id = ("r" if area_id % 2 else "w") + str(area_id // 2)
                # Local metre scale for WGS84 degrees in Santiago. Includes courtyard holes.
                area = round(geometry.area * 110_910 * 111_320 * math.cos(math.radians(point.y)), 1)
            writer.writerow([kind, tags.get("name", ""), f"{point.x:.6f}", f"{point.y:.6f}",
                             cut, area, tags.get("building:levels", ""), osm_id])
    print(f"{missing_cut} inside the box without a comuna polygon")


if __name__ == "__main__":
    main(sys.argv[1])
