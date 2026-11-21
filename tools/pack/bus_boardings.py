"""Write stop and comuna validation targets from the joined weekday bus observations."""
import csv
from collections import Counter
from decimal import Decimal
from pathlib import Path

pack = Path("data/mvp")
out = Path("data/validation/bus_boardings_2026_04")
out.mkdir(parents=True, exist_ok=True)

with (pack / "communes.csv").open() as source:
    cuts = {row["name"]: row["cut"] for row in csv.DictReader(source)}

slots = [f"{i // 2:02d}:{i % 2 * 30:02d}" for i in range(48)]
codes = Counter()
observations = Counter()
boardings = Counter()

with (pack / "bus_activity.csv").open() as source, (out / "bus_stops.csv").open("w", newline="") as output:
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["stop_code", "longitude", "latitude", "cut", "half_hour", "boardings"])
    for row in csv.DictReader(source):
        cut = "" if row["comuna"] == "SIN_COMUNA" else cuts[row["comuna"]]
        codes[cut] += 1
        for i, slot in enumerate(slots):
            value = row[f"b{i:02d}"]
            if value:
                # The source averages five weekdays. Remove Excel's binary float artifacts.
                value = Decimal(value).quantize(Decimal("0.1"))
                observations[cut, slot] += 1
                boardings[cut, slot] += value
            writer.writerow([row["user_stop"], row["stop_lon"], row["stop_lat"], cut, slot, value])

with (out / "by_comuna.csv").open("w", newline="") as output:
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["cut", "half_hour", "boardings", "observed_codes", "total_codes"])
    for cut in sorted(codes):
        for slot in slots:
            writer.writerow([cut, slot, boardings.get((cut, slot), ""), observations[cut, slot], codes[cut]])
