"""Run the V5 evaluations from the repository root."""

import argparse
import csv
import json
import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[2]


def read_json(path):
    return json.loads((ROOT / path).read_text())


def run_scenario(command, changes):
    result = subprocess.run(
        [*command, "data/mvp", "-"],
        input=json.dumps(changes),
        stdout=subprocess.PIPE,
        text=True,
        check=True,
        cwd=ROOT,
    )
    payload = json.loads(result.stdout)
    if payload["not_modeled"] or len(payload["applied"]) != len(changes):
        raise SystemExit(f"Cannot score a partial scenario: {payload['not_modeled']}")
    return payload


def station_bins(payload):
    return {station["id"]: station["scenario_half_hour"] for station in payload["stations"]}


def score_line1(command, baseline):
    case = read_json("tools/v5/line1.json")
    baseline = station_bins(baseline)
    event = station_bins(run_scenario(command, read_json(case["changes"])))
    cells = {}
    print("Line 1 split service, 18 Nov 2024 (development comparison)")
    print("Station  Bin   Modeled change  Observed change  Reference entries")

    for station, observed in case["stations"].items():
        cells[station] = []
        for bin_index, reference, count in zip(case["bins"], observed["reference"], observed["event"]):
            predicted_change = event[station][bin_index] - baseline[station][bin_index]
            observed_change = count - reference
            cells[station].append((predicted_change, observed_change, reference))
            print(f"{station:7} {bin_index:3} {predicted_change:+15.1f} {observed_change:+16} {reference:18}")

    for name, stations in case["panels"].items():
        panel = [cell for station in stations for cell in cells[station]]
        reference_count = sum(reference for _, _, reference in panel)
        error = sum(abs(pred - obs) for pred, obs, _ in panel)
        no_change = sum(abs(obs) for _, obs, _ in panel)
        print(f"{name}: {len(panel)} cells, {error:.1f}/{reference_count} = {error/reference_count:.4f}"
              f" (no change: {no_change}/{reference_count} = {no_change/reference_count:.4f})")

    for station in case["closed_stations"]:
        counts = [event[station][bin_index] for bin_index in case["bins"]]
        print(f"{station}: closed station entries {counts}")


def score_monumental(command):
    for case in read_json("tools/v5/monumental.json")["cases"]:
        egress = run_scenario(command, case["request"])["events"][0]["metro_egress"]
        by_station = {station["id"]: station["entries_by_half_hour"] for station in egress["stations"]}
        score_monumental_reference(case, by_station, case["status"], case["control"])
        for reference in case.get("additional_references", []):
            score_monumental_reference(case, by_station, reference["status"], reference["control"])


def score_monumental_reference(case, by_station, status, control):
    print(f"Monumental {case['id']} ({status})")
    print("Station  Start  Modeled total  Observed entries  Reference entries")
    error = no_event = observed_total = cells = 0
    for station, bins in case["observed"].items():
        for bin_index, observed in bins.items():
            reference = control[station][bin_index]
            predicted = reference + by_station.get(station, {}).get(bin_index, 0)
            error += abs(predicted - observed)
            no_event += abs(reference - observed)
            observed_total += observed
            cells += 1
            half_hour = int(bin_index)
            time = f"{half_hour // 2:02}:{half_hour % 2 * 30:02}"
            print(f"{station:7} {time} {predicted:14.1f} {observed:17.0f} {reference:18.0f}")
    print(f"{cells} cells: error / observed entries = {error / observed_total:.4f}"
          f" (no event: {no_event / observed_total:.4f})")


def score_april_metro(baseline, output_dir):
    bins = {station["id"]: station["base_half_hour"] for station in baseline["stations"]}
    predictions = output_dir / "april-bins.json"
    predictions.write_text(json.dumps(bins))
    print("April 2026 Metro (held-out weekday entries)", flush=True)
    subprocess.run(
        [sys.executable, "-I", "tools/pack/check_score.py", str(predictions)],
        cwd=ROOT,
        check=True,
    )


def score_line2_opening(command, output_dir):
    opened_ids = ("CM", "EB", "PI", "OB")
    stations = {station["id"]: station for station in read_json("data/mvp/metro_training.json")}
    opened = [stations.pop(station_id) for station_id in opened_ids]
    features = [
        {
            "type": "Feature",
            "properties": {
                "kind": "station",
                "line": "opening",
                "id": station["id"],
                "name": station["name"],
            },
            "geometry": {
                "type": "Point",
                "coordinates": [station["longitude"], station["latitude"]],
            },
        }
        for station in opened
    ]

    pack = output_dir / "mvp"
    pack.mkdir()
    rewritten = {"metro_training.json", "metro_planned.geojson", "bus_station_access.csv"}
    for path in (ROOT / "data/mvp").iterdir():
        if path.name not in rewritten:
            (pack / path.name).symlink_to(path)
    (pack / "metro_training.json").write_text(json.dumps(list(stations.values())))
    (pack / "metro_planned.geojson").write_text(
        json.dumps({"type": "FeatureCollection", "features": features})
    )
    with (ROOT / "data/mvp/bus_station_access.csv").open(newline="") as source:
        with (pack / "bus_station_access.csv").open("w", newline="") as target:
            rows = csv.DictReader(source)
            writer = csv.DictWriter(target, fieldnames=rows.fieldnames)
            writer.writeheader()
            writer.writerows(row for row in rows if row["parent_id"] not in opened_ids)

    predictions = output_dir / "line2-predictions.json"
    print("Line 2 opening, April 2024 observations (2026 network)", flush=True)
    subprocess.run(
        [*command, str(pack), str(predictions), "l9", "line:opening"],
        cwd=ROOT,
        check=True,
    )
    by_station = json.loads(predictions.read_text())
    with (ROOT / "data/validation/metro_extension_2023/line2.csv").open(newline="") as source:
        observed = {
            row["station"]: float(row["april_2024_weekday"])
            for row in csv.DictReader(source)
            if row["group"] == "extension"
        }

    print("Station  Modeled entries  Observed entries")
    modeled_total = station_error = 0.0
    for station in opened:
        modeled = sum(by_station[station["id"]])
        actual = observed[station["name"]]
        modeled_total += modeled
        station_error += abs(modeled - actual)
        print(f"{station['id']:7} {modeled:15,.1f} {actual:17,.0f}")

    observed_total = sum(observed.values())
    aggregate_error = abs(modeled_total - observed_total)
    print(f"Total: {modeled_total:,.1f} modeled, {observed_total:,.0f} observed")
    print(f"Aggregate error: {aggregate_error:,.1f}/{observed_total:,.0f} = "
          f"{aggregate_error / observed_total:.4f}")
    print(f"Station error: {station_error:,.1f}/{observed_total:,.0f} = "
          f"{station_error / observed_total:.4f} (closed-station reference: 1.0000)")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", type=Path, help="Use a prebuilt scenario binary.")
    parser.add_argument("--metro", action="store_true", help="Include April Metro and Line 2.")
    parser.add_argument("--station-disruptions", action="store_true",
                        help="Include Universidad de Santiago and Baquedano.")
    parser.add_argument("--population", type=Path, help="Use a prebuilt population binary for Line 2.")
    args = parser.parse_args()
    cargo = ["cargo", "run", "--quiet", "--release", "--locked",
             "--manifest-path", "engine/Cargo.toml", "--bin"]
    command = [*cargo, "scenario", "--"]
    if args.scenario:
        command = [str(args.scenario.resolve())]

    baseline = run_scenario(command, [])
    score_line1(command, baseline)
    score_monumental(command)
    if args.metro:
        population = [str(args.population.resolve())] if args.population else [*cargo, "population", "--"]
        with TemporaryDirectory() as temp:
            score_april_metro(baseline, Path(temp))
            score_line2_opening(population, Path(temp))
    if args.station_disruptions:
        from station_disruptions import score_station_disruptions

        score_station_disruptions(command)
    print("Mapocho: pending a measured service comparison.")


if __name__ == "__main__":
    main()
