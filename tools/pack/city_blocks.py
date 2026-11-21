"""Build display squares for all Census home blocks from INE areas and local job weights."""
import csv
import json
import math
from pathlib import Path
import sqlite3
import sys

PACK = Path("data/mvp")
GPKG = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("data/raw/verify-people-work/cartography-rm.gpkg")
METRES_PER_DEGREE = math.pi * 6371008.8 / 180

with sqlite3.connect(f"{GPKG.resolve().as_uri()}?mode=ro", uri=True) as db:
    area_degrees2 = {
        str(int(code)): area
        for layer in ("Manzanas_CPV24", "Entidades_CPV24")
        for code, area in db.execute(f"SELECT MANZENT, SHAPE_Area FROM {layer}")
    }

with (PACK / "jobs.csv").open() as f:
    jobs = {r["block_id"]: int(r["jobs"]) for r in csv.DictReader(f)}

volumes = []
with (PACK / "blocks.csv").open() as f:
    for row in csv.DictReader(f):
        block = row["block_id"]
        lon, lat = float(row["longitude"]), float(row["latitude"])
        if block in area_degrees2:
            area = area_degrees2[block] * METRES_PER_DEGREE ** 2 * math.cos(math.radians(lat))
        else:
            # INE gives comuna containers a location only at comuna scale.
            area = 625
        side = round(min(120, 0.8 * math.sqrt(area)), 1)
        activity = int(row["dwellings"]) + 0.5 * jobs.get(block, 0)
        height = round(1 + 5 * math.log2(1 + activity * 10000 / area), 1)
        volumes.append([lon, lat, side, height, row["cut"], block])

output = Path("web/data/city_blocks.json")
output.write_text(json.dumps(volumes, separators=(",", ":")) + "\n")
print(f"{len(volumes):,} blocks in {output.stat().st_size:,} bytes")
