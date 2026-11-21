"""Compact the verified April export to all LABORAL BUS stops, without rounding."""
import csv
import json
import math
import sys
from decimal import Decimal
from pathlib import Path

from export_april_boardings import file_record

source, metro, out = map(Path, sys.argv[1:])
manifest = json.loads((source / "manifest.json").read_text())
stops = {}
total = Decimal(0)
unmatched_total = Decimal(0)
rows = unmatched_rows = 0
fields = ["user_stop", "internal_stop", "comuna", "gtfs_stop_id", "stop_lat", "stop_lon", "join_status"]
with (source / "boardings.csv").open() as stream:
    for r in csv.DictReader(stream):
        if r["day_type"] != "LABORAL" or r["mode"] != "BUS":
            continue
        key = tuple(r[f] for f in fields)
        bins = stops.setdefault(key, [""] * 48)
        hour, minute = map(int, r["half_hour"].split(":"))
        bins[hour * 2 + minute // 30] = r["boardings"]
        rows += 1
        total += Decimal(r["boardings"])
        if r["join_status"] != "matched":
            unmatched_rows += 1
            unmatched_total += Decimal(r["boardings"])
out.mkdir(parents=True, exist_ok=True)
csv_path = out / "bus_activity.csv"
with csv_path.open("w", newline="") as stream:
    writer = csv.writer(stream)
    writer.writerow(fields + [f"b{i:02d}" for i in range(48)])
    writer.writerows([*key, *bins] for key, bins in sorted(stops.items()))
parents = [s["station"] for s in json.loads(metro.read_text())["stations"]]
links = []
for key in sorted(stops):
    if not key[3]:
        continue
    lat, lon = map(math.radians, map(float, key[4:6]))
    for parent in parents:
        plat, plon = map(math.radians, (parent["latitude"], parent["longitude"]))
        a = math.sin((plat - lat) / 2) ** 2 + math.cos(lat) * math.cos(plat) * math.sin((plon - lon) / 2) ** 2
        distance = 6371000 * 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
        if distance <= 300:
            links.append((key[3], parent["id"], distance))
join_path = out / "bus_metro_300m.csv"
with join_path.open("w", newline="") as stream:
    writer = csv.writer(stream)
    writer.writerow(["stop_id", "parent_id", "distance_m"])
    writer.writerows(links)
report = {
    "sources": manifest["sources"],
    "input_export": file_record(source / "boardings.csv"),
    "period": manifest["period"]["LABORAL"], "day_type": "LABORAL", "mode": "BUS",
    "timezone": manifest["timezone"], "unit": manifest["unit"],
    "bins": "b00=00:00, b01=00:30, ..., b47=23:30; average boardings per half hour",
    "missing": "Empty bin = no source observation, not zero. Empty coordinates = unresolved stop.",
    "join": manifest["join"], "source_observations": rows, "stop_rows": len(stops),
    "empty_bins": 48 * len(stops) - rows, "observed_total": str(total),
    "unmatched_observations": unmatched_rows, "unmatched_total": str(unmatched_total),
    "unmatched_stop_rows": sum(key[-1] != "matched" for key in stops),
    "scope": "Nearby bus boarding activity proxy, not feeder alightings or observed transfers. Metro and Tren excluded.",
    "output": file_record(csv_path),
    "metro_source": file_record(metro),
    "nearby_join": {"output": file_record(join_path), "rows": len(links),
                    "bus_stops": len({r[0] for r in links}), "metro_parents": len({r[1] for r in links}),
                    "rule": "Haversine <=300m, radius 6371000m, BUS GTFS point to verified Metro parent. All overlaps retained; no mass split or transfer inference."},
}
(out / "bus_activity_manifest.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
