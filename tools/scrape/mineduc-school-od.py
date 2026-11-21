"""MINEDUC school OD for the RM, at two granularities.

comuna level: Matricula por estudiante 2026, COD_COM_ALU -> COD_COM_RBD.
point level:  SAE 2025 (admission 2026), applicant home lat/lon with added
              noise -> lat/lon of the school that admitted the applicant.

Output: data/raw/mineduc-od/od_school_comuna_rm.csv, od_school_points_rm.csv
"""
import collections, csv, io, pathlib, subprocess, urllib.request

BASE = "https://datosabiertos.mineduc.cl/wp-content/uploads"
FILES = {
    "matricula.rar": f"{BASE}/2026/10/Matricula-Oficial-2026.rar",
    "sae.rar": f"{BASE}/2026/03/SAE_2025.rar",
}
DIR = pathlib.Path("data/raw/mineduc-od")
BBOX = (-33.75, -33.2, -70.95, -70.4)  # Santiago: lat min, lat max, lon min, lon max


# shortcut: MINEDUC ships RAR and the standard library cannot read it, so this calls
# bsdtar. Upgrade when MINEDUC publishes ZIP, or when the repo accepts a RAR dependency.
def member(archive, pattern):
    names = subprocess.run(["bsdtar", "-tf", str(DIR / archive)],
                           capture_output=True, text=True).stdout.splitlines()
    name = next(n for n in names if pattern in n)
    raw = subprocess.run(["bsdtar", "-xOf", str(DIR / archive), name],
                         capture_output=True).stdout
    return csv.DictReader(io.StringIO(raw.decode("utf-8", "replace")), delimiter=";")


def point(row, lat_key, lon_key):
    return float(row[lat_key].replace(",", ".")), float(row[lon_key].replace(",", "."))


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

school = {}
for row in member("sae.rar", "A1_Oferta"):
    school.setdefault(row["rbd"], point(row, "lat", "lon"))
home = {}
for row in member("sae.rar", "B1_Postulantes"):
    home[row["mrun"]] = point(row, "lat_con_error", "lon_con_error")
path = DIR / "od_school_points_rm.csv"
with path.open("w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["home_lat", "home_lon", "rbd_admitido", "school_lat", "school_lon"])
    n = 0
    for row in member("sae.rar", "D1_Resultados"):
        rbd = row["rbd_admitido"].strip()
        if rbd not in school or row["mrun"] not in home:
            continue
        hlat, hlon = home[row["mrun"]]
        if not (BBOX[0] < hlat < BBOX[1] and BBOX[2] < hlon < BBOX[3]):
            continue
        w.writerow([hlat, hlon, rbd, *school[rbd]])
        n += 1
print(path, n, "pairs")
