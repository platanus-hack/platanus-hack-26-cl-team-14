"""Group normal-weekday trips to work in Santiago EOD 2012 by mode and departure time."""
import argparse
import csv
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--survey", type=Path, default=ROOT / "data/raw/eod2012/x/eod2012/EOD_STGO")
parser.add_argument("--output", type=Path, default=ROOT / "data/mvp/work_departures.csv")
args = parser.parse_args()

groups = {
    "car": {1},
    "public": {2, 3, 4, 5, 6, 7, 9, 10, 11, 12},
    "walk": {17},
    "bicycle": {18},
    "other": {8, 13, 14, 15, 16},
}
mode_group = {code: group for group, codes in groups.items() for code in codes}


def weight(text):
    return float(text.replace(",", ".") or 0)


with (args.survey / "personas.csv").open(encoding="utf-8-sig") as f:
    person_weight = {r["Persona"]: weight(r["Factor_LaboralNormal"])
                     for r in csv.DictReader(f, delimiter=";")}

bins = Counter()
samples = Counter()
missing_time = 0
with (args.survey / "viajes.csv").open(encoding="utf-8-sig") as f:
    for row in csv.DictReader(f, delimiter=";"):
        if row["Proposito"] != "1":
            continue
        expanded = weight(row["FactorLaboralNormal"]) * person_weight[row["Persona"]]
        if expanded <= 0:
            continue
        if not row["HoraIni"]:
            missing_time += 1
            continue
        hour, minute = map(int, row["HoraIni"].split(":"))
        group = mode_group[int(row["ModoAgregado"])]
        bins[group, hour * 2 + minute // 30] += expanded
        samples[group] += 1

with args.output.open("w", newline="") as f:
    out = csv.writer(f, lineterminator="\n")
    out.writerow(["mode", "half_hour", "expanded_trips", "share"])
    for group in groups:
        total = sum(bins[group, b] for b in range(48))
        for b in range(48):
            value = bins[group, b]
            out.writerow([group, b, f"{value:.6f}", f"{value / total:.12f}"])
        print(f"{group}: {samples[group]} observed, {total:.1f} expanded trips")
print(f"{missing_time} eligible trips had no departure time.")
