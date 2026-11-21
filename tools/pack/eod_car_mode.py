"""Weekday trips to work in Santiago EOD 2012 by home comuna, household car status and mode."""
import argparse
import csv
import unicodedata
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--survey", type=Path, default=ROOT / "data/raw/eod2012/x/eod2012/EOD_STGO")
parser.add_argument("--output", type=Path, default=ROOT / "data/mvp/eod_car_mode.csv")
args = parser.parse_args()

# The engine's three groups: Census code 2 = public, 1 = car, all other codes = other.
mode = {1: "car", **{c: "public" for c in (2, 3, 4, 5, 6, 7, 9, 10, 11, 12)}}


def weight(text):
    return float(text.replace(",", ".") or 0)


def key(name):
    return unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode().upper().strip()


def rows(name, encoding="utf-8-sig"):
    with (args.survey / f"{name}.csv").open(encoding=encoding) as f:
        yield from csv.DictReader(f, delimiter=";")


with (ROOT / "data/mvp/communes.csv").open() as f:
    cut = {key(r["name"]): r["cut"] for r in csv.DictReader(f)}
# A car: TipoVeh 1 auto, 3 pickup or SUV, 5 van or minivan. NumVeh also counts motorcycles, trucks, taxis and buses.
cars = {r["Hogar"] for r in rows("Vehiculo", "latin-1") if r["TipoVeh"] in ("1", "3", "5")}
home = {r["Hogar"]: (cut[key(r["Comuna"])], r["Hogar"] in cars, weight(r["Factor"])) for r in rows("Hogares")}
person_weight = {r["Persona"]: weight(r["Factor_LaboralNormal"]) for r in rows("personas")}

households = defaultdict(lambda: [0.0, 0.0])  # cut -> [expanded households, with a car]
for c, car, w in home.values():
    households[c][0] += w
    households[c][1] += w * car
trips = defaultdict(float)  # (cut, has_car, mode) -> expanded trips
samples = defaultdict(int)
for r in rows("viajes"):
    if r["Proposito"] != "1" or not r["ModoAgregado"]:
        continue
    w = weight(r["FactorLaboralNormal"]) * person_weight[r["Persona"]]
    if w <= 0:
        continue
    c, car, _ = home[r["Hogar"]]
    k = (c, car, mode.get(int(r["ModoAgregado"]), "other"))
    trips[k] += w
    samples[k] += 1

with args.output.open("w", newline="") as f:
    out = csv.writer(f)
    out.writerow(["cut", "households", "households_with_car", "household_has_car", "mode", "sample_trips", "expanded_trips"])
    for c in sorted(households):
        for car in (False, True):
            for m in ("public", "car", "other"):
                out.writerow([c, f"{households[c][0]:.1f}", f"{households[c][1]:.1f}", int(car), m, samples[c, car, m], f"{trips[c, car, m]:.1f}"])
