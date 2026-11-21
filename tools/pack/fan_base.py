"""Colo-Colo fan share by RM comuna: Census 2024 household heads by education level x Cadem 2024 share by group.

Run from the repository root: python3 -I tools/pack/fan_base.py
Downloads the INE Census 2024 person file (355 MB) to data/raw/censo2024/ if missing.
"""
import collections, csv, io, pathlib, urllib.request, zipfile

URL = "https://storage.googleapis.com/bktdescargascenso2024/personas_censo2024.zip"
ZIP = pathlib.Path("data/raw/censo2024/personas_censo2024.zip")
# Head of household CINE 2011 code -> socioeconomic group proxy. Codes 12 and -99 (no level) are left out.
GROUP = {**{c: "c1" for c in "9 10 11".split()}, "8": "c2", "6": "c3", "7": "c3", **{c: "de" for c in "1 2 3 4 5".split()}}
# Cadem, Radiografía del fútbol chileno, October 2024: share of each group that supports Colo-Colo.
CADEM = {"c1": 0.24, "c2": 0.38, "c3": 0.39, "de": 0.49}

if not ZIP.exists():
    ZIP.parent.mkdir(parents=True, exist_ok=True)
    urllib.request.urlretrieve(URL, ZIP)
heads = collections.defaultdict(collections.Counter)
with zipfile.ZipFile(ZIP).open("personas_censo2024.csv") as f:
    for r in csv.DictReader(io.TextIOWrapper(f, encoding="utf-8"), delimiter=";"):
        if r["region"] == "13" and r["parentesco"] == "1" and r["cine11"] in GROUP:
            heads[r["comuna"]][GROUP[r["cine11"]]] += 1

with open("data/mvp/communes.csv") as f:
    cuts = [r["cut"] for r in csv.DictReader(f)]
with open("data/mvp/fan_base.csv", "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["cut", "heads", "share_c1", "share_c2", "share_c3", "share_de", "colo_colo_share"])
    for cut in cuts:
        h = heads[cut]
        n = sum(h.values())
        s = {g: h[g] / n for g in CADEM}
        w.writerow([cut, n, *(round(s[g], 4) for g in CADEM), round(sum(s[g] * CADEM[g] for g in CADEM), 4)])
