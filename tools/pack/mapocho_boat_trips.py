"""Trip-level test of a Mapocho boat: for weekday EOD 2012 trips whose two ends are within 800 m of two different piers,
does the boat beat the time the person reported (viajes.TiempoViaje)?

Boat minutes = walk to the nearest pier + half the headway + river ride between the two piers + 1 min per intermediate pier
+ walk from the pier. Walk = straight line x 1.3 at 4.5 km/h (as mapocho_boat_minutes.py). River ride = the polyline through
the piers in piers.json order, at the boat speed. Trip weight = FactorLaboralNormal x Factor_LaboralNormal.
Usage, from the repo root: python3 -I tools/pack/mapocho_boat_trips.py [min river km between the piers, default 0]
"""
import csv, json, math, sys
from pathlib import Path

EOD = Path("data/raw/eod2012/x/eod2012/EOD_STGO")
PIER_KM, WALK_KMH, DETOUR, STOP_MIN = 0.8, 4.5, 1.3, 1.0
MIN_RIVER_KM = float(sys.argv[1]) if len(sys.argv) > 1 else 0.0  # minimum distance between the two piers along the pier polyline
SPEEDS, HEADWAYS = (12.0, 17.0, 25.0), (5.0, 10.0, 15.0)


def utm19s_to_lonlat(x, y):  # same transform as mapocho_corridor.py
    a, f, k0 = 6378137.0, 1 / 298.257223563, 0.9996
    e2 = f * (2 - f); ep2 = e2 / (1 - e2)
    mu = (y - 10000000.0) / k0 / (a * (1 - e2 / 4 - 3 * e2**2 / 64 - 5 * e2**3 / 256))
    e1 = (1 - math.sqrt(1 - e2)) / (1 + math.sqrt(1 - e2))
    p1 = (mu + (3 * e1 / 2 - 27 * e1**3 / 32) * math.sin(2 * mu) + (21 * e1**2 / 16 - 55 * e1**4 / 32) * math.sin(4 * mu)
          + (151 * e1**3 / 96) * math.sin(6 * mu) + (1097 * e1**4 / 512) * math.sin(8 * mu))
    s, c, t = math.sin(p1), math.cos(p1), math.tan(p1)
    n = a / math.sqrt(1 - e2 * s * s); r = a * (1 - e2) / (1 - e2 * s * s) ** 1.5
    t2, c1, d = t * t, ep2 * c * c, (x - 500000.0) / (n * k0)
    lat = p1 - (n * t / r) * (d**2 / 2 - (5 + 3 * t2 + 10 * c1 - 4 * c1**2 - 9 * ep2) * d**4 / 24
                              + (61 + 90 * t2 + 298 * c1 + 45 * t2**2 - 252 * ep2 - 3 * c1**2) * d**6 / 720)
    lon = (d - (1 + 2 * t2 + c1) * d**3 / 6 + (5 - 2 * c1 + 28 * t2 - 3 * c1**2 + 8 * ep2 + 24 * t2**2) * d**5 / 120) / c
    return math.degrees(lon) - 69.0, math.degrees(lat)


km = lambda p, q: math.hypot((p[0] - q[0]) * 111.32 * math.cos(math.radians(33.43)), (p[1] - q[1]) * 110.574)
piers = [(c["target"]["longitude"], c["target"]["latitude"]) for c in json.load(open("context/research/mapocho/piers.json"))]
along = [0.0]
for a, b in zip(piers, piers[1:]):
    along.append(along[-1] + km(a, b))
num = lambda s: float(s.replace(",", ".") or 0)
person = {r["Persona"]: num(r["Factor_LaboralNormal"]) for r in csv.DictReader(open(EOD / "personas.csv", encoding="utf-8-sig"), delimiter=";")}

trips = []  # (weight, walk km total, river km, stops, reported minutes, mode)
for r in csv.DictReader(open(EOD / "viajes.csv", encoding="utf-8-sig"), delimiter=";"):
    w = num(r["FactorLaboralNormal"]) * person.get(r["Persona"], 0)
    if w <= 0 or not r["OrigenCoordX"] or not r["DestinoCoordX"] or not r["TiempoViaje"]:
        continue
    o = utm19s_to_lonlat(num(r["OrigenCoordX"]), num(r["OrigenCoordY"]))
    d = utm19s_to_lonlat(num(r["DestinoCoordX"]), num(r["DestinoCoordY"]))
    i = min(range(len(piers)), key=lambda k: km(o, piers[k]))
    j = min(range(len(piers)), key=lambda k: km(d, piers[k]))
    if i == j or km(o, piers[i]) > PIER_KM or km(d, piers[j]) > PIER_KM or abs(along[i] - along[j]) < MIN_RIVER_KM:
        continue
    trips.append((w, km(o, piers[i]) + km(d, piers[j]), abs(along[i] - along[j]), abs(i - j) - 1, int(r["TiempoViaje"]), r["ModoAgregado"]))

total = sum(t[0] for t in trips)
print(f"min river km {MIN_RIVER_KM}; pier-to-pier trips: {len(trips)} records, {total:,.0f} expanded weekday trips")
print("speed_kmh,headway_min,faster_trips,share,median_boat_min")
for v in SPEEDS:
    for h in HEADWAYS:
        boat = [(t[0], t[1] * DETOUR / WALK_KMH * 60 + h / 2 + t[2] / v * 60 + t[3] * STOP_MIN, t[4]) for t in trips]
        faster = sum(w for w, b, rep in boat if b < rep)
        times = sorted((b, w) for w, b, _ in boat); acc = 0
        med = next(b for b, w in times if (acc := acc + w) >= total / 2)
        print(f"{v:.0f},{h:.0f},{faster:.0f},{faster / total:.3f},{med:.0f}")
