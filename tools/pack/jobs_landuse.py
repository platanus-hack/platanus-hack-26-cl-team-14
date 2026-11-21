"""Use 2021 Santiago floor area to allocate jobs in covered census blocks."""
import csv
from collections import Counter
from pathlib import Path

pack = Path("data/mvp")
source = {}
for r in csv.DictReader((pack / "land_use.csv").open()):
    if r["area_type"] == "municipal_block" and r["cut"] == "13101" and r["year"] == "2021":
        source[r["area_id"]] = (float(r["office_m2"]) / (1.175 * 12)
                                + float(r["commerce_m2"]) / (1.175 * 19)
                                + float(r["industry_m2"]) / 36)
weights = Counter()
locations = {}
for r in csv.DictReader((pack / "land_use_blocks.csv").open()):
    if r["area_id"] in source:
        block = r["block_id"]
        weights[block] += source[r["area_id"]] * float(r["source_area_share"])
        locations[block] = r

rows = {r["block_id"]: r for r in csv.DictReader((pack / "jobs.csv").open())}
covered_stock = sum(int(rows[k]["jobs"]) for k in weights if k in rows)
total = sum(weights.values())
for block, weight in weights.items():
    if block not in rows and weight == 0:
        continue
    point = rows.setdefault(block, locations[block])
    point["jobs"] = covered_stock * weight / total

with (pack / "jobs_landuse.csv").open("w", newline="") as output:
    writer = csv.DictWriter(output, fieldnames=["block_id", "cut", "longitude", "latitude", "jobs"], extrasaction="ignore")
    writer.writeheader()
    writer.writerows(rows.values())
