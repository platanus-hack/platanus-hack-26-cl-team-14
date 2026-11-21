"""Build experimental job-point weights from April 15 work arrivals at training stations."""
import csv
import json
import math
from collections import Counter
from pathlib import Path

data = Path("data/mvp")
stations = [s for s in json.loads((data / "metro_training.json").read_text()) if not s["holdout"]]
kx = 111320 * math.cos(math.radians(-33.45))
jobs = list(csv.DictReader((data / "jobs.csv").open()))
assignment = []
mass = Counter()
for row in jobs:
    x, y = float(row["longitude"]), float(row["latitude"])
    distance, station = min(
        (math.hypot((s["longitude"] - x) * kx, (s["latitude"] - y) * 110540), s["id"])
        for s in stations
    )
    assignment.append(station if distance <= 500 else None)
    if distance <= 500:
        mass[station] += int(row["jobs"])

arrivals = Counter()
target = Path("data/validation/trip_purpose_2026_04/metro_arrivals.csv")
for row in csv.DictReader(target.open()):
    station = row["station_id"]
    if station in mass and mass[station] > 0 and row["proposito"] == "TRABAJO":
        arrivals[station] += float(row["expanded_trips"])

scale = sum(mass.values()) / sum(arrivals.values())
census_total = Counter()
weighted_total = Counter()
for row, station in zip(jobs, assignment):
    weight = int(row["jobs"])
    census_total[row["cut"]] += weight
    if station and mass[station] > 0:
        weight *= arrivals[station] * scale / mass[station]
    row["jobs"] = weight
    weighted_total[row["cut"]] += weight

with (data / "jobs_2026.csv").open("w", newline="") as output:
    writer = csv.DictWriter(output, fieldnames=["block_id", "cut", "longitude", "latitude", "jobs"], extrasaction="ignore")
    writer.writeheader()
    for row in jobs:
        cut = row["cut"]
        row["jobs"] = float(row["jobs"]) * census_total[cut] / weighted_total[cut]
        writer.writerow(row)
print(len(stations), "training stations;", sum(v > 0 for v in mass.values()), "with positive job mass")
