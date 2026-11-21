"""Aggregate normal-weekday trips in the Santiago EOD 2012 by home comuna."""

import argparse
import csv
import unicodedata
from bisect import bisect_right
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_SURVEY = ROOT / "data/raw/eod2012/x/eod2012/EOD_STGO"
PURPOSE = {
    "1": "work", "2": "work", "3": "study", "4": "study", "5": "health",
    "6": "leisure", "7": "home", "8": "other", "9": "leisure", "10": "other",
    "11": "shopping", "12": "errands", "13": "leisure", "14": "other", "": "other",
}
LENGTHS = ("0-1", "1-3", "3-5", "5-10", "10-20", "20+", "imputed", "unknown")
BINS = {
    "mode_eod": tuple(str(i) for i in range(1, 19)),
    "start_half_hour": tuple(str(i) for i in range(48)) + ("unknown",),
    "distance_km": LENGTHS,
}


def rows(path, delimiter=";"):
    with path.open(encoding="utf-8-sig", newline="") as source:
        yield from csv.DictReader(source, delimiter=delimiter)


def number(value):
    return float(value.replace(",", ".") or 0)


def plain(name):
    return "".join(unicodedata.normalize("NFKD", name.upper()).encode("ascii", "ignore").decode().split())


def cuts_by_name():
    return {plain(r["name"]): r["cut"] for r in rows(ROOT / "data/mvp/communes.csv", ",")}


def comuna_cuts(survey):
    cuts = cuts_by_name()
    return {r["Id"]: cuts[plain(r["Comuna"])]
            for r in rows(survey / "Tablas_parametros/Comunas.csv", ",") if plain(r["Comuna"]) in cuts}


def survey_inputs(survey):
    cuts = cuts_by_name()
    homes = {r["Hogar"]: cuts[plain(r["Comuna"])] for r in rows(survey / "Hogares.csv")}
    person_weights = {}
    residents = Counter()
    for r in rows(survey / "personas.csv"):
        weight = number(r["Factor_LaboralNormal"])
        if weight > 0:
            person_weights[r["Persona"]] = weight
            residents[homes[r["Hogar"]]] += weight
    return homes, person_weights, residents


def iter_weekday_trips(survey, homes, person_weights):
    for r in rows(survey / "viajes.csv"):
        weight = person_weights.get(r["Persona"], 0) * number(r["FactorLaboralNormal"])
        if weight > 0:
            yield r, homes[r["Hogar"]], PURPOSE[r["Proposito"]], weight


def distance_bin(r):
    if r["Imputada"] == "1":
        return "imputed"
    metres = number(r["DistEuclidiana"])
    if metres <= 0:
        return "unknown"
    return LENGTHS[bisect_right((1000, 3000, 5000, 10000, 20000), metres)]


def start_bin(value):
    if not value:
        return "unknown"
    hour, minute = map(int, value.split(":"))
    return str(hour * 2 + minute // 30)


def aggregate(survey, output):
    homes, person_weights, residents = survey_inputs(survey)
    lengths = {r["Viaje"]: distance_bin(r) for r in rows(survey / "DistanciaViaje.csv")}
    counts, expanded = Counter(), Counter()
    for r, home, purpose, weight in iter_weekday_trips(survey, homes, person_weights):
        for key in ((home, purpose), (home, purpose, "mode_eod", r["ModoAgregado"]),
                    (home, purpose, "start_half_hour", start_bin(r["HoraIni"])),
                    (home, purpose, "distance_km", lengths[r["Viaje"]])):
            counts[key] += 1
            expanded[key] += weight

    with (output / "eod_trip_rates.csv").open("w", newline="") as rates_file, \
         (output / "eod_trip_profiles.csv").open("w", newline="") as profiles_file:
        rates = csv.writer(rates_file, lineterminator="\n")
        profiles = csv.writer(profiles_file, lineterminator="\n")
        common = ["sample_trips", "under_30", "expanded_trips"]
        rates.writerow(["home_cut", "purpose", *common, "expanded_residents", "trips_per_resident"])
        profiles.writerow(["home_cut", "purpose", "dimension", "bin", *common, "share"])
        for home, pop in sorted(residents.items()):
            for purpose in sorted(set(PURPOSE.values())):
                key = home, purpose
                n, total = counts[key], expanded[key]
                rates.writerow([*key, n, int(n < 30), f"{total:.6f}", f"{pop:.6f}", f"{total/pop:.9f}"])
                for dimension, bins in BINS.items():
                    for label in bins:
                        cell = home, purpose, dimension, label
                        n, value = counts[cell], expanded[cell]
                        share = f"{value/total:.9f}" if total else ""
                        profiles.writerow([*cell, n, int(n < 30), f"{value:.6f}", share])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--survey", type=Path, default=DEFAULT_SURVEY)
    parser.add_argument("--output", type=Path, default=ROOT / "data/mvp")
    args = parser.parse_args()
    aggregate(args.survey, args.output)
