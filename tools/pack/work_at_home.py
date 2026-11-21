"""Workers who work at home, by comuna of residence (INE Census 2024 person microdata, p44_lug_trab).

p44_lug_trab: 1 = works in this dwelling, 2 = in this comuna, 3 = in another comuna (mode_share.py keeps 2 and 3).
Output: data/mvp/work_at_home.csv  cut,at_home,travel,at_home_share
travel counts every person with p44 2 or 3 in a region-13 comuna (also those with an unknown or out-of-region work comuna),
so at_home_share = at_home / (at_home + travel) is over all persons with a workplace answer.
Run from the repo root after downloading personas_censo2024.zip (tools/scrape/censo2024-od.py).
"""
import collections, csv, io, pathlib, tempfile, zipfile

ZIP = pathlib.Path("data/raw/censo2024-personas/personas_censo2024.zip")
OUT = pathlib.Path("data/mvp/work_at_home.csv")

counts = collections.Counter()
with zipfile.ZipFile(ZIP) as archive, archive.open("personas_censo2024.csv") as member:
    with io.TextIOWrapper(member, encoding="utf-8", errors="replace", newline="") as source:
        for row in csv.DictReader(source, delimiter=";"):
            if row["comuna"].startswith("13") and row["p44_lug_trab"] in ("1", "2", "3"):
                counts[(f"{int(row['comuna']):05d}", row["p44_lug_trab"] == "1")] += 1

if not counts:
    raise ValueError("No eligible census records for region 13")

with tempfile.TemporaryDirectory(dir=OUT.parent) as directory:
    pending = pathlib.Path(directory) / OUT.name
    with pending.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["cut", "at_home", "travel", "at_home_share"])
        for cut in sorted({c for c, _ in counts}):
            home, travel = counts[(cut, True)], counts[(cut, False)]
            w.writerow([cut, home, travel, f"{home / (home + travel):.4f}"])
    pending.replace(OUT)
home, travel = sum(n for (_, h), n in counts.items() if h), sum(n for (_, h), n in counts.items() if not h)
print(OUT, f"at home {home}, travel {travel}, share {home / (home + travel):.4f}")
