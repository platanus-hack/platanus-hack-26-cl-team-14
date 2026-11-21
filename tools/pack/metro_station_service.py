"""Build mean weekday train departures that passengers can board at each station."""

import argparse
import csv
import io
from collections import Counter, defaultdict
from pathlib import Path
from zipfile import ZipFile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("feed", type=Path, nargs="?",
                    default=Path("data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip"))
args = parser.parse_args()
output = Path("data/mvp/metro_station_service.csv")


def rows(feed, name):
    with feed.open(name + ".txt") as source:
        yield from csv.DictReader(io.TextIOWrapper(source, encoding="utf-8-sig"))


with ZipFile(args.feed) as feed:
    # These services operated April 13–17. There were no weekday exceptions.
    weekdays = {"L": 5, "LJ": 4, "V": 1}
    metro = {r["route_id"] for r in rows(feed, "routes") if r["route_type"] == "1"}
    trips = {r["trip_id"]: r for r in rows(feed, "trips")
             if r["route_id"] in metro and r["service_id"] in weekdays}
    parents = {r["stop_id"]: r["parent_station"] for r in rows(feed, "stops")}
    calls = defaultdict(list)
    for row in rows(feed, "stop_times"):
        if row["trip_id"] in trips:
            calls[row["trip_id"]].append((int(row["stop_sequence"]), row["stop_id"],
                                          row["departure_time"], row["pickup_type"]))

counts = Counter()
for trip_id, stops in calls.items():
    trip = trips[trip_id]
    # The last stop offers no onward ride, even if its pickup_type permits boarding.
    for _, stop, departure, pickup in sorted(stops)[:-1]:
        if pickup == "1":
            continue
        hours, minutes, _ = map(int, departure.split(":"))
        period = f"{hours:02d}:{30 * (minutes // 30):02d}"
        key = parents[stop], trip["route_id"], trip["direction_id"], period
        counts[key] += weekdays[trip["service_id"]]

with output.open("w", newline="") as stream:
    writer = csv.writer(stream, lineterminator="\n")
    writer.writerow(["station_id", "line", "direction", "half_hour", "trains"])
    for key, count in sorted(counts.items()):
        writer.writerow([*key, count / 5])
