"""Minutes changes for a Mapocho river boat: one `minutes` set per comuna pair where the boat is faster.

Usage: python3 -I tools/pack/mapocho_boat_minutes.py > /tmp/boat.json
Reads data/mvp/communes.csv, data/mvp/travel_times.csv and context/research/mapocho/piers.json.
Door-to-door boat minutes = walk to the nearest pier + half the headway + river ride + stops + walk from the pier.
Only comunas whose point is within MAX_WALK_KM of a pier take part. Assumptions are the constants below.
"""
import csv, json, math

SPEED_KMH = 17.0      # Brisbane CityCat average, about 17-18 km/h (Tanko & Burke 2019)
HEADWAY_MIN = 10.0    # one boat every 10 minutes (assumption)
STOP_MIN = 1.0        # dwell and lock time at each intermediate pier (assumption)
WALK_KMH = 4.5
DETOUR = 1.3          # walk path / straight line (assumption)
MAX_WALK_KM = 2.5

km = lambda a, b: math.hypot((a[0] - b[0]) * 92.9, (a[1] - b[1]) * 110.6)  # same km per degree as engine/src/choice.rs
piers = [(c['target']['longitude'], c['target']['latitude']) for c in json.load(open('context/research/mapocho/piers.json'))]
chain = [0.0]
for a, b in zip(piers, piers[1:]):
    chain.append(chain[-1] + km(a, b))
access = {}
for r in csv.DictReader(open('data/mvp/communes.csv')):
    p = (float(r['longitude']), float(r['latitude']))
    i = min(range(len(piers)), key=lambda j: km(p, piers[j]))
    if km(p, piers[i]) <= MAX_WALK_KM:
        access[r['cut']] = (i, km(p, piers[i]) * DETOUR / WALK_KMH * 60)
changes = []
for r in csv.DictReader(open('data/mvp/travel_times.csv')):
    o, d = r['origin_cut'], r['destination_cut']
    if o not in access or d not in access or access[o][0] == access[d][0]:
        continue
    (i, wa), (j, we) = access[o], access[d]
    boat = wa + HEADWAY_MIN / 2 + abs(chain[i] - chain[j]) / SPEED_KMH * 60 + (abs(i - j) - 1) * STOP_MIN + we
    if boat < float(r['minutes_door_to_door']):
        changes.append({"kind": "minutes", "target": {"type": "pair", "origin": o, "destination": d}, "op": "set", "value": round(boat, 1)})
print(json.dumps(changes))
