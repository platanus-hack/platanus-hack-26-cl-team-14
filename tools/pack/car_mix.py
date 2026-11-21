"""Build motorized permit shares for each RM issuing comuna and the region."""
import argparse
import csv
import sys
import unicodedata
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scrape"))
from export_april_boardings import sheet_rows

COLUMNS = {
    "E": "automovil_station_wagon_todoterreno",
    "F": "furgon",
    "G": "minibus_particular",
    "H": "camioneta",
    "I": "motocicleta_y_similares",
    "J": "otros_particulares_con_motor",
    "L": "taxi_basico",
    "M": "taxi_colectivo",
    "N": "taxi_turismo",
    "O": "minibus_transporte_colectivo",
    "P": "bus_transporte_colectivo",
    "Q": "camion_simple",
    "R": "tractocamion",
    "S": "tractor_agricola",
    "T": "otros_carga_con_motor",
}


def name_key(name):
    return unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode().strip().upper()


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("workbook", type=Path, help="INE annual permits workbook for 2024")
args = parser.parse_args()
mvp = Path("data/mvp")
with (mvp / "communes.csv").open() as f:
    cuts = {name_key(row["name"]): row["cut"] for row in csv.DictReader(f)}

fleet = {}
in_rm = False
with zipfile.ZipFile(args.workbook) as book:
    for _, row in sheet_rows(book, "4"):
        level = row.get("B")
        if level == "Región":
            in_rm = row["C"] == "Región Metropolitana de Santiago"
        if in_rm and level in ("Región", "Comuna"):
            cut = "13" if level == "Región" else cuts[name_key(row["C"])]
            fleet[cut] = {label: 0 if row[col] == "-" else int(row[col])
                          for col, label in COLUMNS.items()}

with (mvp / "car_mix.csv").open("w", newline="") as f:
    writer = csv.writer(f, lineterminator="\n")
    writer.writerow(["cut", "year", "source", "class", "share"])
    for cut, counts in sorted(fleet.items()):
        total = sum(counts.values())
        for label, n in counts.items():
            writer.writerow([cut, 2024, "ine_permisos", label, f"{n / total:.12f}"])
