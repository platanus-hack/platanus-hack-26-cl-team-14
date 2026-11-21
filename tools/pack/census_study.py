"""Pack Census study and worker counts for RM residents from local INE archives."""
import argparse
import csv
import io
import zipfile
from collections import Counter
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--raw", type=Path, default=Path("data/raw"))
raw = parser.parse_args().raw
out = Path("data/mvp")
with (out / "communes.csv").open() as f:
    cuts = {row["cut"] for row in csv.DictReader(f)}

levels = {
    "asistencia_parv": "preschool_0_5",
    "asistencia_basica": "basic_6_13",
    "asistencia_media": "secondary_14_17",
    "asistencia_superior": "higher_18_24",
}
counts = Counter()
with zipfile.ZipFile(raw / "censo2024-personas/personas_censo2024.zip") as archive:
    with archive.open("personas_censo2024.csv") as file:
        rows = csv.reader(io.TextIOWrapper(file, encoding="utf-8-sig"), delimiter=";")
        header = next(rows)
        home, place, work = map(header.index, ("comuna", "p44_lug_trab", "p44_lug_trab_esp"))
        columns = [(header.index(name), role) for name, role in levels.items()]
        for row in rows:
            cut = row[home]
            if cut not in cuts:
                continue
            worker = row[place] in ("2", "3") and row[work] in cuts
            role = "no_positive_indicator"
            for column, label in columns:
                value = row[column]
                if value == "1":
                    role = label
                    break
                if value == "-66":
                    role = "suppressed"
                elif value == "-99":
                    role = "no_response"
            counts[cut, role, worker] += 1

with (out / "census_study_by_comuna.csv").open("w", newline="") as f:
    w = csv.writer(f, lineterminator="\n")
    w.writerow(["cut", "study_indicator", "residents", "internal_workers"])
    for cut in sorted(cuts):
        for role in (*levels.values(), "no_positive_indicator", "suppressed", "no_response"):
            workers = counts[cut, role, True]
            w.writerow([cut, role, workers + counts[cut, role, False], workers])

with zipfile.ZipFile(raw / "verify-people-work/census-blocks.zip") as archive:
    with archive.open("Base_manzana_entidad_CPV24.csv") as file:
        rows = csv.DictReader(io.TextIOWrapper(file, encoding="utf-8-sig"), delimiter=";")
        with (out / "higher_ed_home_blocks.csv").open("w", newline="") as f:
            w = csv.writer(f, lineterminator="\n")
            w.writerow(["block_id", "cut", "higher_18_24"])
            for row in rows:
                if row["CUT"] in cuts:
                    value = row["n_asistencia_superior"]
                    w.writerow([int(row["MANZENT"]), row["CUT"], "" if value == "*" else value])
