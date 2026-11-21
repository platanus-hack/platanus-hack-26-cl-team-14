"""Export mapped car routes for comuna pairs with workers in the Census 2024 OD table."""
import csv
import json
from pathlib import Path

pack = Path("data/mvp")
with (pack / "work_od.csv").open() as f:
    work_pairs = {(r["origin_cut"], r["destination_cut"]) for r in csv.DictReader(f)
                  if int(r["workers"]) > 0}

routes = {}
for row in json.loads((pack / "car_paths.json").read_text()):
    origin, destination = row["origin_cut"], row["destination_cut"]
    if (origin, destination) in work_pairs:
        routes[f"{origin}-{destination}"] = row["path"]

out = Path("web/data/car_routes.json")
out.write_text(json.dumps(routes, separators=(",", ":")) + "\n")
print(f"{len(routes)} routes, {out.stat().st_size} bytes")
