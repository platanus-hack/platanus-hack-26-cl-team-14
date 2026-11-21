"""Build municipal block floor areas and SII comuna property counts. Run at repo root."""
import csv
import io
import json
import subprocess
import unicodedata
from pathlib import Path
from urllib.parse import urlencode

PACK = Path("data/mvp")
RAW = Path("data/raw/sii-land")
RAW.mkdir(parents=True, exist_ok=True)
LAYER = ("https://services9.arcgis.com/zqDAjElD9y8GPDk4/arcgis/rest/services/"
         "TOTAL_DESTINOS_M2_CONSTRUIDOS_2021_gdb/FeatureServer/0/query")
COUNTS = "https://www.sii.cl/sobre_el_sii/estadisticas/csv/ebbrrc_16.csv"


def download(url, name):
    path = RAW / name
    subprocess.run(["curl", "--fail", "--silent", "--show-error", "--location", url, "-o", path], check=True)
    return path


def comuna_name(name):
    return "".join(c for c in unicodedata.normalize("NFD", name) if c.isalpha())


with (PACK / "communes.csv").open() as f:
    cuts = {comuna_name(r["name"]): r["cut"] for r in csv.DictReader(f)}

# The SII serves the latest semester at the same URL. Read its year from the file.
count_file = download(COUNTS, "ebbrrc_16.csv")
table = list(csv.reader(io.StringIO(count_file.read_text(encoding="utf-8-sig")), delimiter=";"))
year = int(table[1][0].split()[-1])
header = next(i for i, r in enumerate(table) if r[2] == "PREDIOS TOTALES NO AGRÍCOLAS")
counts = {}
for row in table[header + 1:]:
    if row[0].startswith("TOTAL REGIÓN"):
        break
    if row[0]:
        cut = cuts[comuna_name(row[0])]
        counts[cut] = int(row[2].replace(".", ""))

fields = ["OBJECTID_1", "H", "O", "C", "I", "E", "S", "TOTAL_M2"]
params = dict(f="json", where="1=1", outFields=",".join(fields), returnGeometry="false",
              orderByFields="OBJECTID_1", resultRecordCount=2000)
blocks = []
while True:
    query = urlencode(dict(params, resultOffset=len(blocks)))
    page = json.loads(download(LAYER + "?" + query, f"santiago2021-{len(blocks)}.json").read_bytes())
    for feature in page["features"]:
        row = feature["attributes"]
        areas = [row[k] for k in fields[1:-1]]
        other = row["TOTAL_M2"] - sum(areas)
        blocks.append([f"santiago2021-{row['OBJECTID_1']}", "municipal_block", "13101",
                       *areas, other, "", 2021])
    if not page.get("exceededTransferLimit"):
        break

with (PACK / "land_use.csv").open("w", newline="") as f:
    out = csv.writer(f, lineterminator="\n")
    out.writerow(["area_id", "area_type", "cut", "housing_m2", "office_m2", "commerce_m2",
                  "industry_m2", "education_m2", "health_m2", "other_m2", "property_count", "year"])
    out.writerows(blocks)
    for cut, count in sorted(counts.items()):
        out.writerow([cut, "comuna", cut, *([""] * 7), count, year])
