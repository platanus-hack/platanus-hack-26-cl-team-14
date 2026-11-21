"""Direct bus access to Metro parents on a weekday at 08:00 (March 2026 GTFS)."""
import argparse
import csv
import io
import zipfile
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--gtfs", type=Path, default=ROOT / "data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip")
parser.add_argument("--data", type=Path, default=ROOT / "data/mvp")
args = parser.parse_args()


def seconds(clock):
    hour, minute, second = map(int, clock.split(":"))
    return hour * 3600 + minute * 60 + second


with zipfile.ZipFile(args.gtfs) as feed:
    def table(name):
        return csv.DictReader(io.TextIOWrapper(feed.open(name), encoding="utf-8-sig"))

    bus = {r["route_id"] for r in table("routes.txt") if r["route_type"] == "3"}
    weekday = {r["trip_id"] for r in table("trips.txt")
               if r["route_id"] in bus and r["service_id"] in {"L", "LJ"}}
    wait = {}
    for r in table("frequencies.txt"):
        trip = r["trip_id"]
        if trip in weekday and seconds(r["start_time"]) <= 8 * 3600 < seconds(r["end_time"]):
            wait[trip] = min(wait.get(trip, float("inf")), int(r["headway_secs"]) / 2)
    coords = {r["stop_id"]: (r["stop_lon"], r["stop_lat"]) for r in table("stops.txt")}
    trips = defaultdict(list)
    for r in table("stop_times.txt"):
        if r["trip_id"] in wait:
            trips[r["trip_id"]].append((int(r["stop_sequence"]), r["stop_id"],
                                        seconds(r["arrival_time"]), seconds(r["departure_time"]),
                                        r["pickup_type"] != "1", r["drop_off_type"] != "1"))

# Match the transfer-walk speed in travel_times.py. The link file contains all overlaps.
interchange = defaultdict(list)
with (args.data / "bus_metro_300m.csv").open() as source:
    for r in csv.DictReader(source):
        interchange[r["stop_id"]].append((r["parent_id"], float(r["distance_m"]) / (4.8 / 3.6)))

best = {}
for trip, stops in trips.items():
    stops.sort()
    for j, (_, alight, arrival, _, _, can_alight) in enumerate(stops):
        if not can_alight or alight not in interchange:
            continue
        for _, board, _, departure, can_board, _ in stops[:j]:
            if not can_board or arrival <= departure or board == alight:
                continue
            for parent, walk in interchange[alight]:
                key = (board, parent)
                cost = wait[trip] + arrival - departure + walk
                best[key] = min(best.get(key, float("inf")), cost)

output = args.data / "bus_station_access.csv"
with output.open("w", newline="") as result:
    writer = csv.writer(result, lineterminator="\n")
    writer.writerow(["stop_id", "longitude", "latitude", "parent_id", "seconds"])
    for (stop, parent), cost in sorted(best.items()):
        lon, lat = coords[stop]
        writer.writerow((stop, f"{float(lon):.7f}", f"{float(lat):.7f}", parent, f"{cost:.3f}"))
print(f"{len(trips)} weekday bus trips at 08:00, {len(best)} stop-to-parent links.")
