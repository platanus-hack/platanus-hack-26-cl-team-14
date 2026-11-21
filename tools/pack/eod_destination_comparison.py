"""Compare EOD destination shares with mapped places and April arrivals in training-station comunas."""
import csv
import json
from collections import Counter, defaultdict
from pathlib import Path

from shapely.geometry import Point, shape

mvp = Path("data/mvp")
validation = Path("data/validation/trip_purpose_2026_04")
boundaries = [(f["properties"]["cut"], shape(f["geometry"]))
              for f in json.loads((mvp / "communes.geojson").read_text())["features"]]
station_cut = {}
stations = defaultdict(list)
for s in json.loads((mvp / "metro_training.json").read_text()):
    if s["holdout"]:
        continue
    point = Point(s["longitude"], s["latitude"])
    cut = min(cut for cut, boundary in boundaries if boundary.covers(point))
    station_cut[s["id"]] = cut
    stations[cut].append(s["id"])

arrivals = Counter()
with (validation / "metro_arrivals.csv").open() as source:
    for row in csv.DictReader(source):
        if row["station_id"] in station_cut and row["proposito"] == "OTROS":
            arrivals[station_cut[row["station_id"]]] += float(row["expanded_trips"])

purposes = {"shop": "shopping", "supermarket": "shopping", "mall": "shopping",
            "hospital": "health", "clinic": "health", "public_office": "errands"}
places = defaultdict(Counter)
with (mvp / "activity_points.csv").open() as source:
    for row in csv.DictReader(source):
        if row["cut"] in stations:
            places[purposes[row["kind"]]][row["cut"]] += 1

trips = defaultdict(Counter)
sample = defaultdict(Counter)
with (mvp / "eod_destinations.csv").open() as source:
    for row in csv.DictReader(source):
        cut = row["destination_cut"]
        if cut in stations:
            for purpose in (row["purpose"], "three_purposes"):
                trips[purpose][cut] += float(row["expanded_trips"])
                sample[purpose][cut] += int(row["sample_trips"])

with (validation / "eod_destination_comparison.csv").open("w", newline="") as output:
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["destination_cut", "training_station_ids", "eod_purpose", "proxy",
                     "eod_sample_trips", "eod_expanded_trips", "proxy_count", "eod_share", "proxy_share"])
    for purpose in ("shopping", "health", "errands", "three_purposes"):
        pooled = purpose == "three_purposes"
        counts = arrivals if pooled else places[purpose]
        proxy = "april_otros_expanded_arrivals" if pooled else "osm_objects"
        eod_total = sum(trips[purpose].values())
        proxy_total = sum(counts.values())
        gaps = []
        for cut in sorted(stations):
            eod_share = trips[purpose][cut] / eod_total
            proxy_share = counts[cut] / proxy_total
            gaps.append(abs(eod_share - proxy_share))
            writer.writerow([cut, " ".join(sorted(stations[cut])), purpose, proxy, sample[purpose][cut],
                             round(trips[purpose][cut], 6), round(counts[cut], 6),
                             round(eod_share, 9), round(proxy_share, 9)])
        print(f"{purpose}: EOD {eod_total:.2f}, {proxy} {proxy_total:.2f}, TV {sum(gaps)/2:.6f}")
print(f"{len(station_cut)} training stations in {len(stations)} comunas")
