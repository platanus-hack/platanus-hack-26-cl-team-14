"""Fit and validate a conditional bicycle/walk distance model on Santiago EOD 2012."""
import argparse
import csv
import hashlib
import json
import math
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--survey", type=Path, default=ROOT / "data/raw/eod2012/x/eod2012/EOD_STGO")
parser.add_argument("--output", type=Path, default=ROOT / "data/mvp/active_mode_distance.json")
args = parser.parse_args()


def rows(filename):
    with (args.survey / filename).open(encoding="utf-8-sig") as f:
        yield from csv.DictReader(f, delimiter=";")


def number(text):
    return float(text.replace(",", ".") or 0)


person_weight = {r["Persona"]: number(r["Factor_LaboralNormal"]) for r in rows("personas.csv")}
distance = {r["Viaje"]: (number(r["DistEuclidiana"]), r["Imputada"]) for r in rows("DistanciaViaje.csv")}
folds = [[] for _ in range(5)]
excluded = Counter()
for r in rows("viajes.csv"):
    if r["Proposito"] != "1" or r["ModoAgregado"] not in {"17", "18"}:
        continue
    w = number(r["FactorLaboralNormal"]) * person_weight[r["Persona"]]
    if w <= 0:
        continue
    metres, imputed = distance[r["Viaje"]]
    if imputed != "0" or metres <= 0:
        excluded["imputed" if imputed != "0" else "nonpositive"] += 1
        continue
    digest = hashlib.sha256(r["Hogar"].encode("utf-8")).digest()
    fold = int.from_bytes(digest[:8], "big") % 5
    folds[fold].append((math.log1p(metres / 1000), int(r["ModoAgregado"] == "18"), w))


def log_odds(sample):
    bike = sum(w for _, y, w in sample if y)
    walk = sum(w for _, y, w in sample if not y)
    return math.log(bike / walk)


def fit(sample):
    intercept, slope = log_odds(sample), 0.0
    for _ in range(25):
        g0 = g1 = h00 = h01 = h11 = 0.0
        for x, y, w in sample:
            p = 1 / (1 + math.exp(-(intercept + slope * x)))
            error, curvature = w * (y - p), w * p * (1 - p)
            g0 += error
            g1 += error * x
            h00 += curvature
            h01 += curvature * x
            h11 += curvature * x * x
        det = h00 * h11 - h01 * h01
        da, db = (h11 * g0 - h01 * g1) / det, (h00 * g1 - h01 * g0) / det
        intercept += da
        slope += db
        if max(abs(da), abs(db)) < 1e-10:
            break
    return intercept, slope


def loss(sample, intercept, slope):
    # Weighted sum. Divide once by total held-out weight for the pooled score.
    return sum(w * math.log1p(math.exp((1 - 2 * y) * (intercept + slope * x))) for x, y, w in sample)


results = []
total_distance = total_constant = total_weight = 0.0
for i, held in enumerate(folds):
    train = [trip for j, fold in enumerate(folds) if j != i for trip in fold]
    distance_loss = loss(held, *fit(train))
    constant_loss = loss(held, log_odds(train), 0.0)
    weight = sum(w for _, _, w in held)
    bike = sum(y for _, y, _ in held)
    results.append(dict(fold=i, walk=len(held)-bike, bicycle=bike, weight=weight,
                        distance_loss=distance_loss/weight, constant_loss=constant_loss/weight))
    total_distance += distance_loss
    total_constant += constant_loss
    total_weight += weight

sample = [trip for fold in folds for trip in fold]
out = dict(source="Santiago EOD 2012", registration="1573676569355092079",
           outcome="bicycle conditional on walk or bicycle", excluded=dict(excluded),
           folds=results, weight=total_weight, distance_loss=total_distance/total_weight,
           constant_loss=total_constant/total_weight, accepted=total_distance < total_constant)
if out["accepted"]:
    out["intercept"], out["slope_log1p_km"] = fit(sample)
args.output.write_text(json.dumps(out, indent=2) + "\n")
print(f"{len(sample)} trips, weighted held-out log loss: distance {out['distance_loss']:.6f}, constant {out['constant_loss']:.6f}")
