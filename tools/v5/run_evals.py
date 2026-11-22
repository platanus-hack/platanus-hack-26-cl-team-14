"""Run the V5 evaluations from the repository root."""

import argparse
import json
import subprocess
from pathlib import Path

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
    return {station["id"]: station["scenario_half_hour"] for station in payload["stations"]}


def score_line1(command):
    case = read_json("tools/v5/line1.json")
    baseline = run_scenario(command, [])
    event = run_scenario(command, read_json(case["changes"]))
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", type=Path, help="Use a prebuilt scenario binary.")
    args = parser.parse_args()
    command = ["cargo", "run", "--quiet", "--release", "--locked",
               "--manifest-path", "engine/Cargo.toml", "--bin", "scenario", "--"]
    if args.scenario:
        command = [str(args.scenario.resolve())]

    score_line1(command)
    print("Monumental, April 2025: pending a case on the current engine.")
    print("Monumental, November 2025: pending a comparable Saturday reference.")
    print("Mapocho: pending a measured service comparison.")


if __name__ == "__main__":
    main()
