"""Build municipal block and commune floor areas, plus SII property counts. Run at repo root."""
import csv
import io
import json
import re
import subprocess
import unicodedata
from pathlib import Path
from urllib.parse import urlencode

PACK = Path("data/mvp")
RAW = Path("data/raw/sii-land")
RAW.mkdir(parents=True, exist_ok=True)
LAYER = ("https://services9.arcgis.com/zqDAjElD9y8GPDk4/arcgis/rest/services/"
         "TOTAL_DESTINOS_M2_CONSTRUIDOS_2021_gdb/FeatureServer/0/query")
FLORIDA = "https://appx.laflorida.cl/transparencia/documentacion/Inf_Final_ECVLF_cap3.pdf"
COUNTS = "https://www.sii.cl/sobre_el_sii/estadisticas/csv/ebbrrc_16.csv"


def download(url, name):
    path = RAW / name
    subprocess.run(["curl", "--fail", "--silent", "--show-error", "--location", url, "-o", path], check=True)
    return path


def comuna_name(name):
    return "".join(c for c in unicodedata.normalize("NFD", name.upper()) if c.isalpha())


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

# Tables 3-1 and 3-2 report different years and use different categories.
report = download(FLORIDA, "la-florida-road-capacity-ch3.pdf")
pages = subprocess.check_output(
    ["pdftotext", "-layout", "-f", "6", "-l", "8", report, "-"], text=True
).split("\f")
code_areas = {
    code: int(area.replace(".", ""))
    for code, area in re.findall(r"^\s*([A-Z-])\s+.+?\s{2,}([\d.]+)\s+[\d,]+%\s*$", pages[0], re.M)
}
florida_total = int(re.search(r"^\s*Total\s+([\d.]+)", pages[0], re.M)[1].replace(".", ""))
florida_areas = [code_areas[code] for code in ("H", "O", "C", "I", "E", "S")]
agricultural = sum(code_areas[k] for k in ("A", "B", "F", "Y"))
florida_other = sum(code_areas.values()) - sum(florida_areas) - agricultural

comuna_areas = {}
for line in pages[2].splitlines():
    cells = line.split()
    if cells[:1] == ["Total"]:
        break
    if len(cells) > 8 and all(re.fullmatch(r"[\d.]+", value) for value in cells[-8:]):
        name = " ".join(cells[:-8])
        cut = cuts[comuna_name(name)]
        comuna_areas[cut] = [int(value.replace(".", "")) for value in cells[-8:]]

with (PACK / "land_use.csv").open("w", newline="") as f:
    out = csv.writer(f, lineterminator="\n")
    out.writerow(["area_id", "area_type", "cut", "housing_m2", "office_m2", "commerce_m2",
                  "industry_m2", "education_m2", "health_m2", "other_m2", "property_count", "year",
                  "industry_and_mining_m2", "public_admin_office_health_m2", "other_group_m2",
                  "excluded_group_m2", "agriculture_m2", "published_total_m2"])
    for block in blocks:
        out.writerow([*block, "", "", "", "", "", sum(block[3:10])])
    for cut, count in sorted(counts.items()):
        out.writerow([cut, "comuna", cut, *([""] * 7), count, year, *([""] * 6)])
    for cut, areas in sorted(comuna_areas.items()):
        commerce, education, housing, industry_mining, services, other_group, excluded, total = areas
        out.writerow([cut, "comuna", cut, housing, "", commerce, "", education, "", "", "", 2016,
                      industry_mining, services, other_group, excluded, "", total])
    out.writerow(["13110", "comuna", "13110", *florida_areas, florida_other, "", 2017,
                  "", "", "", "", agricultural, florida_total])
