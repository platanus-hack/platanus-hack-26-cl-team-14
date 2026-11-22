"""Fit a Huff model of weekday shopping trips (EOD 2012) by comuna, then place the MUT at its site and at the Monumental.

P(d | o) ∝ (shops_d + 1)^alpha × exp(−beta × minutes_od). shops_d = mapped shop, mall and supermarket points in comuna d
(activity_points.csv). minutes = public-transport door-to-door minutes (travel_times.csv), with the straight-line fit of
choice.rs for missing pairs. Trips: viajes.Proposito 11 (De compras), weight FactorLaboralNormal × Factor_LaboralNormal,
home comuna from Hogares, destination comuna from viajes.ComunaDestino.
Run from the repo root: python3 -I tools/pack/shopping_huff.py
Other purposes: python3 -I tools/pack/shopping_huff.py PURPOSE KINDS, e.g. `5 hospital,clinic` (De salud); prints the fit only.
"""
import csv
import math
import sys
import unicodedata
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EOD = ROOT / "data/raw/eod2012/x/eod2012/EOD_STGO"
MVP = ROOT / "data/mvp"
PURPOSE, KINDS = (sys.argv[1], sys.argv[2].split(",")) if len(sys.argv) > 2 else ("11", ["shop", "mall", "supermarket"])
# The MUT: 26 mapped shop points inside its OSM bbox. Points: named_places.csv (MUT) and the Monumental bbox centre.
MUT_SHOPS = 26
SITES = {"mut_site": (-70.601132, -33.417280, "13114"), "monumental_site": (-70.60598, -33.50650, "13118")}


def key(name):
    return unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode().upper().strip()


def rows(path, delimiter=","):
    with path.open(encoding="utf-8-sig") as f:
        yield from csv.DictReader(f, delimiter=delimiter)


def w(text):
    return float(text.replace(",", ".") or 0)


com = list(rows(MVP / "communes.csv"))
cuts = [r["cut"] for r in com]
ix = {c: i for i, c in enumerate(cuts)}
n = len(cuts)
by_name = {key(r["name"]): r["cut"] for r in com}
eod_cut = {r["Id"]: by_name.get(key(r["Comuna"])) for r in rows(EOD / "Tablas_parametros/Comunas.csv")}

# Minutes, as choice.rs: GTFS door-to-door, else a + b × straight-line km (intra-comuna: half the distance to the nearest comuna).
xy = [(float(r["longitude"]), float(r["latitude"])) for r in com]
line = lambda a, b: math.hypot((xy[a][0] - xy[b][0]) * 92.9, (xy[a][1] - xy[b][1]) * 110.6)
km = [[min(line(o, e) for e in range(n) if e != o) / 2 if o == d else line(o, d) for d in range(n)] for o in range(n)]
minutes = [[math.nan] * n for _ in range(n)]
pairs = []
for r in rows(MVP / "travel_times.csv"):
    o, d = ix[r["origin_cut"]], ix[r["destination_cut"]]
    minutes[o][d] = float(r["minutes_door_to_door"])
    pairs.append((km[o][d], minutes[o][d]))
m = len(pairs)
sx, sy = sum(p[0] for p in pairs), sum(p[1] for p in pairs)
slope = (m * sum(p[0] * p[1] for p in pairs) - sx * sy) / (m * sum(p[0] ** 2 for p in pairs) - sx * sx)
intercept = (sy - slope * sx) / m
minutes = [[t if not math.isnan(t) else intercept + slope * km[o][d] for d, t in enumerate(row)] for o, row in enumerate(minutes)]

shops = [0.0] * n
for r in rows(MVP / "activity_points.csv"):
    if r["kind"] in KINDS and r["cut"] in ix:
        shops[ix[r["cut"]]] += 1

home = {r["Hogar"]: by_name.get(key(r["Comuna"])) for r in rows(EOD / "Hogares.csv", ";")}
person = {r["Persona"]: w(r["Factor_LaboralNormal"]) for r in rows(EOD / "personas.csv", ";")}
flows = [[0.0] * n for _ in range(n)]
sample = 0
for r in rows(EOD / "viajes.csv", ";"):
    o, d = home.get(r["Hogar"]), eod_cut.get(r["ComunaDestino"])
    weight = w(r["FactorLaboralNormal"]) * person[r["Persona"]]
    if r["Proposito"] == PURPOSE and o in ix and d in ix and weight > 0:
        flows[ix[o]][ix[d]] += weight
        sample += 1


def shares(alpha, beta, o, size=shops, t=None):
    t = t or minutes[o]
    u = [(size[d] + 1) ** alpha * math.exp(-beta * t[d]) for d in range(len(size))]  # +1: rural comunas outside the OSM extract have trips and no mapped shop
    s = sum(u)
    return [x / s for x in u]


def loglik(alpha, beta):
    return sum(f * math.log(p) for o in range(n) for f, p in zip(flows[o], shares(alpha, beta, o)) if f > 0)


def best_beta(alpha):
    lo, hi = 0.0, 0.5
    for _ in range(40):
        a, b = lo + (hi - lo) / 3, hi - (hi - lo) / 3
        lo, hi = (a, hi) if loglik(alpha, a) < loglik(alpha, b) else (lo, b)
    return (lo + hi) / 2


lo, hi = 0.0, 3.0
for _ in range(30):
    a, b = lo + (hi - lo) / 3, hi - (hi - lo) / 3
    lo, hi = (a, hi) if loglik(a, best_beta(a)) < loglik(b, best_beta(b)) else (lo, b)
alpha = (lo + hi) / 2
beta = best_beta(alpha)
total = sum(map(sum, flows))
fit = sum(abs(sum(flows[o][d] for o in range(n)) / total - sum(sum(flows[o]) * shares(alpha, beta, o)[d] for o in range(n)) / total) for d in range(n)) / 2
print(f"purpose {PURPOSE} trips to {'/'.join(KINDS)}: {sample} sample, {total:,.0f} expanded; alpha {alpha:.3f}, beta {beta:.4f} per minute; destination total variation {fit:.3f}")

if PURPOSE != "11":
    sys.exit()

# The MUT as one more destination, at each site. Minutes from origin o to the site: the comuna-pair minutes to the site's comuna.
out = MVP / "shopping_huff.csv"
with out.open("w", newline="") as f:
    wr = csv.writer(f)
    wr.writerow(["site", "origin_cut", "origin_name", "share_of_mut_visitors", "mut_share_of_origin_shopping"])
    for site, (_, _, cut) in SITES.items():
        d = ix[cut]
        weights = []
        for o in range(n):
            size, t = shops + [MUT_SHOPS], minutes[o] + [minutes[o][d]]
            p = shares(alpha, beta, o, size, t)[-1]
            weights.append((sum(flows[o]) * p, p))
        visitors = sum(v for v, _ in weights)
        for o in sorted(range(n), key=lambda o: -weights[o][0]):
            wr.writerow([site, cuts[o], com[o]["name"], f"{weights[o][0] / visitors:.4f}", f"{weights[o][1]:.5f}"])
        top = sorted(range(n), key=lambda o: -weights[o][0])[:5]
        print(site, f"{visitors:,.0f} weekday shopping trips", ", ".join(f"{com[o]['name']} {weights[o][0] / visitors:.1%}" for o in top))
print(f"wrote {out.relative_to(ROOT)}")
