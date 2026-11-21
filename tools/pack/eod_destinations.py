"""Aggregate normal-weekday EOD 2012 destinations by home comuna and purpose."""

import argparse
import csv
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from eod_trip_rates import comuna_cuts, iter_weekday_trips, survey_inputs

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--survey", type=Path, default=ROOT / "data/raw/eod2012/x/eod2012/EOD_STGO")
parser.add_argument("--output", type=Path, default=ROOT / "data/mvp/eod_destinations.csv")
args = parser.parse_args()

cuts = comuna_cuts(args.survey)
with (args.survey / "Tablas_parametros/Comunas.csv").open(encoding="utf-8-sig", newline="") as source:
    names = {r["Id"]: r["Comuna"] for r in csv.DictReader(source)}
homes, person_weights, _ = survey_inputs(args.survey)
samples = Counter()
expanded = Counter()
totals = Counter()
for trip, home, purpose, weight in iter_weekday_trips(args.survey, homes, person_weights):
    if purpose in {"shopping", "health", "errands"}:
        cell = (home, purpose, trip["ComunaDestino"])
        samples[cell] += 1
        expanded[cell] += weight
        totals[home, purpose] += weight

with args.output.open("w", encoding="utf-8", newline="") as output:
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(("home_cut", "purpose", "destination_eod", "destination_cut", "destination_name",
                     "sample_trips", "expanded_trips", "destination_share", "under_30"))
    for cell, count in sorted(samples.items()):
        home, purpose, dest = cell
        writer.writerow((home, purpose, dest, cuts.get(dest, ""), names.get(dest, "Unknown"),
                         count, f"{expanded[cell]:.6f}", f"{expanded[cell] / totals[home, purpose]:.9f}",
                         int(count < 30)))
