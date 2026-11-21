"""The two origin-destination matrices of the MVP, at comuna scale.

work_od.csv    Census 2024 person microdata. Home comuna (`comuna`) to work comuna
               (`p44_lug_trab_esp`), only for `p44_lug_trab` 2 (works in this comuna)
               and 3 (works in another comuna). tools/scrape/censo2024-od.py drops
               the other answers and writes the counts to excluded_destinations.csv.
school_od.csv  MINEDUC Matricula Oficial 2026. Student comuna (`COD_COM_ALU`) to
               school comuna (`COD_COM_RBD`). tools/scrape/mineduc-school-od.py.

Both matrices keep only a pair where each code is one of the 346 comunas of the
MINEDUC directory, so a country code or a region-only code cannot pass. Each code
becomes the 5-digit INE code: the microdata write regions 1 to 9 with 4 digits.

The origin is always a comuna of the Region Metropolitana. The destination can be
any comuna of Chile; those trips leave the model area but they still remove a person
from the origin.

Output: data/mvp/work_od.csv    origin_cut,destination_cut,workers
        data/mvp/school_od.csv  origin_cut,destination_cut,students
"""
import csv, io, pathlib, subprocess, urllib.request

DIR_URL = "https://datosabiertos.mineduc.cl/wp-content/uploads/2026/11/Directorio-EE-Oficial-2026.rar"
DIR_RAR = pathlib.Path("data/raw/mineduc-directorio/dir2026.rar")
DIR_MEMBER = "20261013_Directorio_Oficial_EE_2026_20260430_WEB.csv"
OUT = pathlib.Path("data/mvp")

if not DIR_RAR.exists():
    DIR_RAR.parent.mkdir(parents=True, exist_ok=True)
    urllib.request.urlretrieve(DIR_URL, DIR_RAR)

raw = subprocess.run(["bsdtar", "-xOf", str(DIR_RAR), DIR_MEMBER], capture_output=True).stdout
COMUNAS = {int(r["COD_COM_RBD"])
           for r in csv.DictReader(io.StringIO(raw.decode("utf-8-sig", "replace")), delimiter=";")}

JOBS = [
    ("work_od.csv", "data/raw/censo2024-personas/od_work_rm.csv",
     ("home_comuna", "work_comuna", "personas"), "workers"),
    ("school_od.csv", "data/raw/mineduc-od/od_school_comuna_rm.csv",
     ("home_comuna", "school_comuna", "students"), "students"),
]

OUT.mkdir(parents=True, exist_ok=True)
for name, source, (o_col, d_col, n_col), unit in JOBS:
    pairs, dropped = [], 0
    for row in csv.DictReader(open(source, encoding="utf-8")):
        o, d, n = int(row[o_col]), int(row[d_col]), int(row[n_col])
        if o in COMUNAS and d in COMUNAS:
            pairs.append([f"{o:05d}", f"{d:05d}", n])
        else:
            dropped += n
    pairs.sort(key=lambda r: (r[0], r[1]))
    with (OUT / name).open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["origin_cut", "destination_cut", unit])
        w.writerows(pairs)
    inside = sum(r[2] for r in pairs if r[1].startswith("13"))
    total = sum(r[2] for r in pairs)
    print(f"{OUT/name} {len(pairs)} pairs, {total} {unit}, "
          f"{inside} inside the RM, {total - inside} to another region, {dropped} dropped")
