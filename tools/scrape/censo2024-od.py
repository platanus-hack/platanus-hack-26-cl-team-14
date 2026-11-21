"""Censo 2024 person microdata -> two known-destination comuna OD matrices for the RM.

work.csv      home comuna (`comuna`) -> work comuna (`p44_lug_trab_esp`)
migration.csv comuna of residence in 2019 (`p24_lug_resid5_esp`) -> comuna in 2024

Output: data/raw/censo2024-personas/od_work_rm.csv, od_migration_rm.csv
Excluded response codes and counts: excluded_destinations.csv
Blank and -99 can include non-applicable responses; counts cover all RM persons.
"""
import collections, csv, pathlib, subprocess, urllib.request

URL = "https://storage.googleapis.com/bktdescargascenso2024/personas_censo2024.zip"
DIR = pathlib.Path("data/raw/censo2024-personas")
ZIP = DIR / "personas_censo2024.zip"
MISSING = {"", "-99", "998", "999"}

DIR.mkdir(parents=True, exist_ok=True)
if not ZIP.exists():
    urllib.request.urlretrieve(URL, ZIP)

work = collections.Counter()
migration = collections.Counter()
excluded = collections.Counter()
proc = subprocess.Popen(["unzip", "-p", str(ZIP), "personas_censo2024.csv"], stdout=subprocess.PIPE)
reader = csv.DictReader((l.decode("utf-8", "replace") for l in proc.stdout), delimiter=";")
for row in reader:
    home = row["comuna"]
    if not home.startswith("13"):
        continue
    dest = row["p44_lug_trab_esp"]
    if dest not in MISSING:
        work[(home, dest)] += 1
    else:
        excluded[("work", dest)] += 1
    before = row["p24_lug_resid5_esp"]
    if before not in MISSING:
        migration[(before, home)] += 1
    else:
        excluded[("migration", before)] += 1

for name, counter, cols in [("od_work_rm.csv", work, ("home_comuna", "work_comuna")),
                            ("od_migration_rm.csv", migration, ("comuna_2019", "comuna_2024"))]:
    path = DIR / name
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow([*cols, "personas"])
        for (o, d), n in counter.most_common():
            w.writerow([o, d, n])
    print(path, len(counter), "pairs")

with (DIR / "excluded_destinations.csv").open("w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["matrix", "excluded_code", "personas"])
    for (matrix, code), n in sorted(excluded.items()):
        w.writerow([matrix, code, n])
