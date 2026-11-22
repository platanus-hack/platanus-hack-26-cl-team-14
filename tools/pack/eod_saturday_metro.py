"""Share of Saturday trips that use Metro, by directed EOD sector pair (EOD 2012).

Weight = viajes.FactorSabadoNormal x personas.Factor_SabadoNormal. Metro = ModoAgregado 4, 8-13, 15
(any trip with a Metro stage). Sectors 1-7 only. Feeds the event egress of engine/src/event_egress.rs.

Usage: python3 -I tools/pack/eod_saturday_metro.py /path/to/EOD_STGO > data/mvp/eod_saturday_metro_share.csv
"""
import collections, csv, sys

EOD = sys.argv[1]
METRO = {4, 8, 9, 10, 11, 12, 13, 15}
num = lambda x: float(x.replace(",", ".")) if x else 0.0
person = {r["Persona"]: num(r["Factor_SabadoNormal"])
          for r in csv.DictReader(open(f"{EOD}/personas.csv", encoding="latin-1"), delimiter=";")}
trips = collections.defaultdict(lambda: [0.0, 0.0])
for r in csv.DictReader(open(f"{EOD}/viajes.csv", encoding="latin-1"), delimiter=";"):
    w = num(r["FactorSabadoNormal"]) * person.get(r["Persona"], 0.0)
    o, d = r["SectorOrigen"], r["SectorDestino"]
    if w <= 0 or o not in list("1234567") or d not in list("1234567"):
        continue
    trips[(o, d)][0] += w
    trips[(o, d)][1] += w * (int(r["ModoAgregado"] or 0) in METRO)
print("origin_sector,destination_sector,saturday_trips,metro_share")
for (o, d), (t, m) in sorted(trips.items()):
    print(f"{o},{d},{t:.1f},{m / t:.4f}")
