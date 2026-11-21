"""Censo 2024 person microdata -> two full comuna OD matrices for the RM.

od_work_rm.csv      home comuna (`comuna`) -> work place (`p44_lug_trab_esp`), keyed also
                    by `p44_lug_trab`: 1 works at home, 2 same comuna, 3 other comuna,
                    4 other country (the destination is a country code), 5 no fixed place
                    (the destination is 998/999). Keep code 1 out of any trip count.
od_migration_rm.csv comuna of residence in 2019 (`p24_lug_resid5_esp`) -> comuna in 2024

Output: data/raw/censo2024-personas/od_work_rm.csv, od_migration_rm.csv
"""
import collections, csv, io, pathlib, urllib.request, zipfile

URL = "https://storage.googleapis.com/bktdescargascenso2024/personas_censo2024.zip"
DIR = pathlib.Path("data/raw/censo2024-personas")
ZIP = DIR / "personas_censo2024.zip"
MISSING = {"", "-99"}

DIR.mkdir(parents=True, exist_ok=True)
if not ZIP.exists():
    urllib.request.urlretrieve(URL, ZIP)

work = collections.Counter()
migration = collections.Counter()
member = zipfile.ZipFile(ZIP).open("personas_censo2024.csv")
reader = csv.DictReader(io.TextIOWrapper(member, encoding="utf-8", errors="replace"), delimiter=";")
for row in reader:
    home = row["comuna"]
    if not home.startswith("13"):
        continue
    dest = row["p44_lug_trab_esp"]
    if dest not in MISSING:
        work[(home, dest, row["p44_lug_trab"])] += 1
    before = row["p24_lug_resid5_esp"]
    if before not in MISSING:
        migration[(before, home)] += 1

for name, counter, cols in [("od_work_rm.csv", work, ("home_comuna", "work_place", "p44_lug_trab")),
                            ("od_migration_rm.csv", migration, ("comuna_2019", "comuna_2024"))]:
    path = DIR / name
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow([*cols, "personas"])
        for key, n in counter.most_common():
            w.writerow([*key, n])
    print(path, len(counter), "pairs")
