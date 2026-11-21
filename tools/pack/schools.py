"""Operating schools of the Region Metropolitana, with a point and the enrolment.

Source: MINEDUC Directorio Oficial de Establecimientos 2026 (cut 2026-04-30).
No script in the repository wrote data/raw/mineduc-directorio/dir2026.rar, so this
builder downloads it when it is not there.

Filters: COD_REG_RBD = 13, ESTADO_ESTAB = 1 (operating), a coordinate pair that
is present and inside the RM box. The directory writes the decimal mark as a comma
and pads the coordinate fields with spaces.

`enrollment` is MAT_TOTAL, the exact sum of MAT_ENS_1 to MAT_ENS_8, so it counts
pre-school. `enrollment_no_preschool` is MAT_TOTAL minus MAT_ENS_1 (parvularia): school enrolment
excluding pre-school. It is not trips and it is no evidence of independent travel. Both
columns come from the same row; neither is derived from the other file.

Output: data/mvp/schools.csv  rbd,name,cut,longitude,latitude,enrollment,enrollment_no_preschool
"""
import csv, io, pathlib, subprocess, urllib.request

URL = "https://datosabiertos.mineduc.cl/wp-content/uploads/2026/11/Directorio-EE-Oficial-2026.rar"
RAR = pathlib.Path("data/raw/mineduc-directorio/dir2026.rar")
MEMBER = "20261013_Directorio_Oficial_EE_2026_20260430_WEB.csv"
OUT = pathlib.Path("data/mvp/schools.csv")
BOX = (-34.4, -32.8, -71.8, -69.7)  # lat min, lat max, lon min, lon max


def number(text):
    text = text.strip().replace(",", ".")
    return float(text) if text else None


if not RAR.exists():
    RAR.parent.mkdir(parents=True, exist_ok=True)
    urllib.request.urlretrieve(URL, RAR)

raw = subprocess.run(["bsdtar", "-xOf", str(RAR), MEMBER], capture_output=True).stdout
rows = csv.DictReader(io.StringIO(raw.decode("utf-8-sig", "replace")), delimiter=";")

schools, no_point, outside = [], 0, 0
for row in rows:
    if row["COD_REG_RBD"] != "13" or row["ESTADO_ESTAB"].strip() != "1":
        continue
    lat, lon = number(row["LATITUD"]), number(row["LONGITUD"])
    if lat is None or lon is None:
        no_point += 1
        continue
    if not (BOX[0] <= lat <= BOX[1] and BOX[2] <= lon <= BOX[3]):
        outside += 1
        continue
    schools.append([row["RBD"], row["NOM_RBD"].strip(), f'{int(row["COD_COM_RBD"]):05d}',
                    f"{lon:.6f}", f"{lat:.6f}", int(row["MAT_TOTAL"]),
                    int(row["MAT_TOTAL"]) - int(row["MAT_ENS_1"] or 0)])

OUT.parent.mkdir(parents=True, exist_ok=True)
with OUT.open("w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["rbd", "name", "cut", "longitude", "latitude", "enrollment",
                "enrollment_no_preschool"])
    w.writerows(sorted(schools, key=lambda r: int(r[0])))

print(OUT, len(schools), "schools", sum(r[5] for r in schools), "enrolment,",
      sum(r[6] for r in schools), "without pre-school")
print("dropped:", no_point, "with no coordinate,", outside, "outside the RM box")
