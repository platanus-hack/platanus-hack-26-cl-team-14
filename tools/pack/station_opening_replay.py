"""Compare the station-opening access rule with April 2024 weekday entries at the L2 extension.

Run from a full checkout on the test laptop: python3 -I tools/pack/station_opening_replay.py
The remaining network, feeder paths and commuter inputs come from the 2026 model. This
checks the size of the response at four opened stations. It cannot isolate a causal effect.
"""
import csv
import json
import subprocess
import tempfile
from pathlib import Path

source = Path("data/mvp").resolve()
observations = Path("data/validation/metro_extension_2023/line2.csv")
# The four IDs are fixed by the historical opening. No counts enter the synthetic catalog.
opened_ids = {"EB", "OB", "CM", "PI"}
stations = json.loads((source / "metro_training.json").read_text())
current = [s for s in stations if s["id"] not in opened_ids]
opened = [s for s in stations if s["id"] in opened_ids]
features = [
    {
        "type": "Feature",
        "properties": {"kind": "station", "line": "opening", "id": s["id"], "name": s["name"]},
        "geometry": {"type": "Point", "coordinates": [s["longitude"], s["latitude"]]},
    }
    for s in opened
]

with tempfile.TemporaryDirectory() as tmp:
    pack = Path(tmp) / "mvp"
    pack.mkdir()
    rewritten = {"metro_training.json", "metro_planned.geojson", "bus_station_access.csv"}
    for path in source.iterdir():
        if path.name not in rewritten:
            (pack / path.name).symlink_to(path)

    (pack / "metro_training.json").write_text(json.dumps(current))
    (pack / "metro_planned.geojson").write_text(json.dumps({"type": "FeatureCollection", "features": features}))
    with (source / "bus_station_access.csv").open(newline="") as before:
        with (pack / "bus_station_access.csv").open("w", newline="") as after:
            rows = csv.DictReader(before)
            writer = csv.DictWriter(after, fieldnames=rows.fieldnames)
            writer.writeheader()
            writer.writerows(row for row in rows if row["parent_id"] not in opened_ids)

    path = Path(tmp) / "opened.json"
    subprocess.run([
        "cargo", "run", "--locked", "--release", "--manifest-path", "engine/Cargo.toml",
        "--bin", "population", "--", str(pack), str(path), "l9", "line:opening",
    ], check=True)
    predictions = json.loads(path.read_text())

with observations.open(newline="") as f:
    observed = {r["station"]: float(r["april_2024_weekday"]) for r in csv.DictReader(f) if r["group"] == "extension"}
assert {s["name"] for s in opened} == observed.keys()
result = []
for s in opened:
    estimate = sum(predictions[s["id"]])
    actual = observed[s["name"]]
    result.append({"id": s["id"], "name": s["name"], "modeled": estimate, "observed": actual, "difference": estimate - actual})

print(json.dumps({
    "training_stations": sum(not s["holdout"] for s in current),
    "remaining_network_parents": len(current),
    "stations": result,
    "modeled_total": sum(r["modeled"] for r in result),
    "observed_total": sum(observed.values()),
    "absolute_error": sum(abs(r["difference"]) for r in result),
}, indent=2))
