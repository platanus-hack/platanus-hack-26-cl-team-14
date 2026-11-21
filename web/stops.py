"""Observed bus boardings near the MUT -> web/data/stops.json. Observations only, no model.

DTPM April 2026, LABORAL day, mean per half hour. Bus stops join by exact stop_id.
A half hour with no row is null, not zero. Metro stations are NOT here: the page reads
the three reviewed files (PR #29). Copy them from main until this branch has them:
  for s in tobalaba los_leones el_golf; do git show origin/main:data/mvp/${s}_observations.json > web/data/${s}_observations.json; done
Run from the repo root: python3 web/stops.py
"""
import csv, io, json, math, re, zipfile

MUT = (-70.6009, -33.4180)
RADIUS_M = 1000
XLSX = "data/raw/dtpm/subidas_2026_04.xlsx"
GTFS = "data/raw/verify-transport-metro/gtfs_20261114.zip"

near = {}
for r in csv.DictReader(io.TextIOWrapper(zipfile.ZipFile(GTFS).open("stops.txt"), "utf-8-sig")):
    if not r["stop_lat"] or not re.match(r"P[A-J]\d", r["stop_id"]):
        continue
    lon, lat = float(r["stop_lon"]), float(r["stop_lat"])
    if math.hypot((lon - MUT[0]) * 93200, (lat - MUT[1]) * 110900) <= RADIUS_M:  # metres at 33.4 S
        near[r["stop_id"]] = (lon, lat)

z = zipfile.ZipFile(XLSX)
ss = re.findall(r"<si><t[^>]*>([^<]*)</t></si>", z.read("xl/sharedStrings.xml").decode())
laboral = str(ss.index("LABORAL"))
# columns: Tipo_dia, Modo, Paradero, Paradero Usuario, Comuna, Media_hora, Subidas_Promedio
obs = {}
f, buf = z.open("xl/worksheets/sheet3.xml"), ""
while chunk := f.read(1 << 22):
    buf += chunk.decode()
    rows = buf.split("</row>")
    buf = rows.pop()
    for row in rows:
        v = re.findall(r"<v>([^<]*)</v>", row)
        if v[0] == laboral and ss[int(v[3])] in near:
            o = obs.setdefault(ss[int(v[3])], [None] * 48)
            k = round(float(v[5]) * 48) % 48
            o[k] = round((o[k] or 0) + float(v[6]), 1)

stops = [{"stop_id": k, "lon": near[k][0], "lat": near[k][1], "observed": o} for k, o in obs.items()]
json.dump(stops, open("web/data/stops.json", "w"))
print(len(stops), "bus stops,", round(sum(x or 0 for s in stops for x in s["observed"]), 1), "boardings per work day")
