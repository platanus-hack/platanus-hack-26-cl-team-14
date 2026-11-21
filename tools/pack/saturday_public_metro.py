"""Metro use among complete public-transport-only trips in Santiago EOD 2012."""
import argparse
import csv
import json
from collections import defaultdict
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--survey", type=Path, default=ROOT / "data/raw/eod2012/x/eod2012/EOD_STGO")
parser.add_argument("--output", type=Path, default=ROOT / "data/mvp/saturday_public_metro.json")
args = parser.parse_args()


def rows(filename):
    with (args.survey / filename).open(encoding="latin1") as f:
        yield from csv.DictReader(f, delimiter=";")


def weight(text):
    return Decimal(text.replace(",", ".") or "0")


# EOD stage codes: feeder/trunk buses, Metro, colectivo, taxi, rural/intercity bus,
# urban bus paid to its driver, and train. Metro is code 4.
public = {2, 3, 4, 5, 7, 12, 14, 16}
stages = defaultdict(list)
for r in rows("Etapas.csv"):
    stages[r["Viaje"]].append((int(r["Etapa"]), int(r["Modo"])))
person = {r["Persona"]: weight(r["Factor_SabadoNormal"]) for r in rows("personas.csv")}

keys = ("public_only", "metro_first", "metro_any")
counts = dict.fromkeys(keys, 0)
expanded = dict.fromkeys(keys, Decimal(0))
for r in rows("viajes.csv"):
    w = weight(r["FactorSabadoNormal"]) * person[r["Persona"]]
    if w <= 0:
        continue
    trip = stages.get(r["Viaje"], [])
    if not trip or len(trip) != int(r["Etapas"]) or any(mode not in public for _, mode in trip):
        continue
    for key, include in (("public_only", True), ("metro_first", min(trip)[1] == 4),
                         ("metro_any", any(mode == 4 for _, mode in trip))):
        if include:
            counts[key] += 1
            expanded[key] += w

# Exact decimal counts let callers recover the fraction without rounding the weights.
result = {"source": "Santiago EOD 2012", "period": "normal Saturday, all trip purposes",
          "sample_trips": counts, "expanded_trips": {k: str(v) for k, v in expanded.items()},
          "metro_first_share": float(expanded["metro_first"] / expanded["public_only"]),
          "metro_any_share": float(expanded["metro_any"] / expanded["public_only"])}
args.output.write_text(json.dumps(result, indent=2) + "\n")
