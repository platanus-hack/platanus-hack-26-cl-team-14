"""Intersect Santiago's 2021 cadastral aggregates with 2024 INE urban blocks.

Run with shapely >= 2.1 and pyproj. Pass the INE RM GeoPackage and the
municipal GeoJSON (see context/research/land-use-blocks.md).
"""
import csv
import json
import sqlite3
import sys
from pathlib import Path

import shapely
from pyproj import Transformer
from shapely.geometry import shape

census_path, municipal_path = sys.argv[1:]
to_metric_census = Transformer.from_crs(4674, 32719, always_xy=True)
to_metric_city = Transformer.from_crs(4326, 32719, always_xy=True)
codes, geometries, locations = [], [], []
with sqlite3.connect(census_path) as census:
    for code, blob in census.execute("select MANZENT, SHAPE from Manzanas_CPV24 where CUT = 13101"):
        # These 2D GeoPackage geometries have an 8-byte header and a 32-byte XY envelope.
        polygon = shapely.from_wkb(blob[40:])
        largest = max(shapely.get_parts(polygon), key=lambda part: part.area)
        codes.append(str(int(code)))
        locations.append(largest.centroid)
        projected = shapely.transform(polygon, to_metric_census.transform, interleaved=False)
        geometries.append(shapely.make_valid(projected))

tree = shapely.STRtree(geometries)
rows = []
for feature in json.loads(Path(municipal_path).read_text())["features"]:
    area_id = f"santiago2021-{feature['properties']['OBJECTID_1']}"
    polygon = shape(feature["geometry"])
    polygon = shapely.make_valid(shapely.transform(polygon, to_metric_city.transform, interleaved=False))
    for index in tree.query(polygon, predicate="intersects"):
        overlap = polygon.intersection(geometries[index]).area
        if overlap > 0:
            point = locations[index]
            rows.append((area_id, "13101", codes[index], f"{overlap / polygon.area:.10f}",
                         f"{point.x:.6f}", f"{point.y:.6f}"))

with open("data/mvp/land_use_blocks.csv", "w", newline="") as out:
    writer = csv.writer(out, lineterminator="\n")
    writer.writerow(["area_id", "cut", "block_id", "source_area_share", "longitude", "latitude"])
    writer.writerows(sorted(rows))
