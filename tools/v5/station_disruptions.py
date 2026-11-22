"""Run the frozen Universidad de Santiago and Baquedano comparisons."""

import argparse
from pathlib import Path

USACH_CHANGE = {
    "kind": "station",
    "op": "set",
    "target": {"type": "station", "id": "US"},
    "value": 0,
    "half_hours": [39, 41],
    "temporary": True,
}
# Dated DTPM stages documented in context/research/v5-usach-skip.md.
USACH_COUNTS = {
    "EL": {"reference": [1257, 1107], "event": [1619, 1407]},
    "AH": {"reference": [362, 309], "event": [429, 406]},
}


def print_score(name, bins, counts, baseline, scenario):
    error = no_change = denominator = 0
    print(name)
    print("Station  Start  Modeled change  Observed change  Reference entries")
    for station, observed in counts.items():
        for bin_index, reference, event in zip(bins, observed["reference"], observed["event"]):
            predicted = scenario[station][bin_index] - baseline[station][bin_index]
            change = event - reference
            error += abs(predicted - change)
            no_change += abs(change)
            denominator += reference
            time = f"{bin_index // 2:02}:{bin_index % 2 * 30:02}"
            print(f"{station:7} {time} {predicted:+15.1f} {change:+16} {reference:18}")
    print(f"Model: {error:.1f}/{denominator} = {error / denominator:.4%}")
    print(f"No change: {no_change}/{denominator} = {no_change / denominator:.4%}")


def score_station_disruptions(command):
    from run_evals import read_json, run_scenario, station_bins

    usach = run_scenario(command, [USACH_CHANGE])
    usach_baseline = {row["id"]: row["base_half_hour"] for row in usach["stations"]}
    usach_scenario = station_bins(usach)
    print_score("Universidad de Santiago, 21 Apr 2025 (descriptive, no refit)",
                (39, 40), USACH_COUNTS, usach_baseline, usach_scenario)
    print(f"US: closed station entries {usach_scenario['US'][39:41]} (outside score)")

    case = read_json("context/research/v5_baquedano_l5_case.json")
    baseline = station_bins(run_scenario(command, []))
    for variant, request in (
        ("two-direction cut", case["changes"]),
        ("one-direction skip", "context/research/v5_baquedano_l5_skip.json"),
    ):
        scenario = station_bins(run_scenario(command, read_json(request)))
        print_score(f"Baquedano L5, 19 Nov 2024 ({variant}, no refit)",
                    case["bins"], case["stations"], baseline, scenario)
    print("Baquedano's observed changes (-4, -62) are inside the reference range (-64 to +64).")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", type=Path, default=Path("engine/target/release/scenario"),
                        help="Path to a prebuilt release scenario binary.")
    score_station_disruptions([str(parser.parse_args().scenario.resolve())])
