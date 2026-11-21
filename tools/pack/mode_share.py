"""Mode of travel to work, by comuna of residence, for the commuters of work_od.csv.

Source: INE Census 2024 person microdata, `p45_medio_transporte`, counted over the
same persons that work_od.csv keeps: home comuna in region 13 and `p44_lug_trab`
2 (works in this comuna) or 3 (works in another comuna), with a known comuna
destination. So the rows of one comuna sum to that comuna's total in work_od.csv.

It also writes mode_share_by_workplace.csv, the same counts split by `p44_lug_trab`:
2 works in this comuna, 3 works in another comuna. The two populations travel very
differently (the RM shares are 32.2% and 61.2% public transport). The code 3 share
is a candidate refinement for cross-comuna flows, to preregister and score before
changing a fitted model. Summing over `work_place` reproduces mode_share.csv exactly.

Output: data/mvp/mode_share.csv               cut,mode,commuters
        data/mvp/mode_share_by_workplace.csv  cut,work_place,mode,commuters
The code labels of `p45` are in the data/mvp/README.md section.
"""
import collections, csv, pathlib, subprocess

ZIP = pathlib.Path("data/raw/censo2024-personas/personas_censo2024.zip")
OUT = pathlib.Path("data/mvp/mode_share.csv")
SPLIT = pathlib.Path("data/mvp/mode_share_by_workplace.csv")
MISSING = {"", "-99", "998", "999"}

counts = collections.Counter()
proc = subprocess.Popen(["unzip", "-p", str(ZIP), "personas_censo2024.csv"], stdout=subprocess.PIPE)
reader = csv.DictReader((l.decode("utf-8", "replace") for l in proc.stdout), delimiter=";")
for row in reader:
    home = row["comuna"]
    if not home.startswith("13") or row["p44_lug_trab"] not in ("2", "3"):
        continue
    dest = row["p44_lug_trab_esp"]
    if dest in MISSING or dest.zfill(5).endswith("000"):
        continue
    counts[(f"{int(home):05d}", row["p44_lug_trab"], row["p45_medio_transporte"])] += 1

OUT.parent.mkdir(parents=True, exist_ok=True)
with SPLIT.open("w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["cut", "work_place", "mode", "commuters"])
    for key, n in sorted(counts.items()):
        w.writerow(list(key) + [n])
print(SPLIT, len(counts), "rows", sum(counts.values()), "commuters")

merged = collections.Counter()
for (cut, _, mode), n in counts.items():
    merged[(cut, mode)] += n
with OUT.open("w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["cut", "mode", "commuters"])
    for (cut, mode), n in sorted(merged.items()):
        w.writerow([cut, mode, n])
print(OUT, len(merged), "rows", sum(merged.values()), "commuters")
print("modes:", sorted({m for _, m in merged}))
