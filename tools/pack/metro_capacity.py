"""Count scheduled Metro trip starts on an average weekday, April 13–17, 2026."""

import argparse
import csv
import io
from collections import Counter
from pathlib import Path
from zipfile import ZipFile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("feed", type=Path, nargs="?",
                    default=Path("data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip"))
args = parser.parse_args()

# Calendar for this five-day period, with no service exceptions.
days = {"L": 5, "LJ": 4, "V": 1}
# CAF's nominal rating for the five-car L3/L6 train at six passengers per m².
places = {"L3": 1299, "L6": 1299}


def rows(feed, name):
    with feed.open(name + ".txt") as source:
        yield from csv.DictReader(io.TextIOWrapper(source, encoding="utf-8-sig"))


with ZipFile(args.feed) as feed:
    lines = {r["route_id"] for r in rows(feed, "routes") if r["route_type"] == "1"}
    trips = {r["trip_id"]: r for r in rows(feed, "trips")
             if r["route_id"] in lines and r["service_id"] in days}
    counts = Counter()
    for row in rows(feed, "stop_times"):
        # Every trip in this feed starts at sequence 1, including short turns.
        if row["trip_id"] in trips and row["stop_sequence"] == "1":
            trip = trips[row["trip_id"]]
            hour, minute, _ = map(int, row["departure_time"].split(":"))
            counts[trip["route_id"], trip["direction_id"], 2 * hour + minute // 30] += days[trip["service_id"]]

with Path("data/mvp/metro_capacity.csv").open("w", newline="") as result:
    writer = csv.writer(result, lineterminator="\n")
    writer.writerow(["line", "direction", "half_hour", "trains", "places_per_train", "places_per_half_hour"])
    for line, direction in sorted({(r["route_id"], r["direction_id"]) for r in trips.values()}):
        capacity = places.get(line)
        for bin_id in range(48):
            trains = counts[line, direction, bin_id] / 5
            half_hour = f"{bin_id // 2:02d}:{30 * (bin_id % 2):02d}"
            writer.writerow([line, direction, half_hour, trains, capacity,
                             counts[line, direction, bin_id] * capacity / 5 if capacity else None])
