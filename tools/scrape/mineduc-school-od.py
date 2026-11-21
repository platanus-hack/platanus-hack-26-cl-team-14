"""MINEDUC school OD aggregates for the RM.

comuna level: Matricula por estudiante 2026, COD_COM_ALU -> COD_COM_RBD.

Output: data/raw/mineduc-od/od_school_comuna_rm.csv
"""
import collections, csv, io, pathlib, subprocess, urllib.request

BASE = "https://datosabiertos.mineduc.cl/wp-content/uploads"
FILES = {
    "matricula.rar": f"{BASE}/2026/10/Matricula-Oficial-2026.rar",
}
DIR = pathlib.Path("data/raw/mineduc-od")


def member(archive, pattern):
    names = subprocess.run(["bsdtar", "-tf", str(DIR / archive)],
                           capture_output=True, text=True).stdout.splitlines()
    name = next(n for n in names if pattern in n)
    raw = subprocess.run(["bsdtar", "-xOf", str(DIR / archive), name],
                         capture_output=True).stdout
    return csv.DictReader(io.StringIO(raw.decode("utf-8", "replace")), delimiter=";")


DIR.mkdir(parents=True, exist_ok=True)
for name, url in FILES.items():
    if not (DIR / name).exists():
        urllib.request.urlretrieve(url, DIR / name)

pairs = collections.Counter()
for row in member("matricula.rar", "unica_2026"):
    if row["COD_REG_ALU"] == "13":
        pairs[(row["COD_COM_ALU"], row["NOM_COM_ALU"], row["COD_COM_RBD"], row["NOM_COM_RBD"])] += 1
path = DIR / "od_school_comuna_rm.csv"
with path.open("w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["home_comuna", "home_name", "school_comuna", "school_name", "students"])
    for key, n in pairs.most_common():
        w.writerow([*key, n])
print(path, len(pairs), "pairs")
