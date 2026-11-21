"""Extract residential permits by comuna from the MINVU annual table."""
# /// script
# dependencies = ["openpyxl==3.1.5"]
# ///
import csv
import json
import subprocess
from pathlib import Path
from urllib.parse import urlencode

from openpyxl import load_workbook

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "data/mvp"
RAW = ROOT / "data/raw/building_permits/annual.xlsx"
URL = "https://catalogo.minvu.cl/cgi-bin/koha/opac-retrieve-file.pl?id=d40a41546b26b2a7fe5f1440af6e6ff2"
YEARS = (2023, 2024, 2025)

RAW.parent.mkdir(parents=True, exist_ok=True)
subprocess.run(["curl", "-fsSL", URL, "-o", str(RAW)], check=True)
book = load_workbook(RAW, read_only=True, data_only=True)


def by_comuna(sheet):
    rows = sheet.values
    header = next(row for row in rows if row[:2] == ("Región", "Comuna"))
    columns = {str(value).split()[0]: i for i, value in enumerate(header) if value}
    return {
        (row[1].upper(), year): row[columns[str(year)]]
        for row in rows if row[0] == "Metropolitana" and row[1]
        for year in YEARS
    }


# Ask INE for one sum per commune and year. The response contains no individual permits.
service = "https://services5.arcgis.com/hUyD8u3TeZLKPe4T/arcgis/rest/services/CRF_PE_v2/FeatureServer/1/query"
query = urlencode({
    "f": "json",
    "where": "COD_REGION = 13 AND AÑO IN (2023, 2024, 2025)",
    "groupByFieldsForStatistics": "COD_COMUNA,AÑO",
    "outStatistics": json.dumps([{
        "statisticType": "sum", "onStatisticField": "SUPERFICIE", "outStatisticFieldName": "mapped_m2"
    }]),
    "returnGeometry": "false",
})
mapped = {}
for feature in json.loads(subprocess.check_output(["curl", "-fsSL", service + "?" + query]))["features"]:
    row = feature["attributes"]
    mapped[str(row["COD_COMUNA"]), row["AÑO"]] = row["mapped_m2"]

units = by_comuna(book["número_total"])
area = by_comuna(book["m2_total"])
with (DATA / "communes.csv").open() as source, (DATA / "building_permits.csv").open("w", newline="") as dest:
    writer = csv.writer(dest, lineterminator="\n")
    writer.writerow(["cut", "comuna", "year", "dwellings_authorized", "residential_m2_authorized", "minvu_status", "ine_mapped_m2_all_uses"])
    for comuna in csv.DictReader(source):
        for year in YEARS:
            key = (comuna["name"], year)
            status = "preliminary" if year == 2025 else "final"
            writer.writerow([comuna["cut"], comuna["name"], year, units[key], area[key], status, mapped.get((comuna["cut"], year))])
