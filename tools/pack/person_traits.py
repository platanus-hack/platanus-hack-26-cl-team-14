"""Age band and household size of the persons in mode_od.csv and of RM school students.

Source: INE Census 2024 person microdata (`edad`, `id_vivienda`, `id_hogar`).
Workers use the same filters as mode_share.py, so the counts of one
(origin_cut, destination_cut, mode) cell sum to that row of mode_od.csv.
Students: RM residents who attend basic or secondary school
(`asistencia_basica` or `asistencia_media` = 1), the blocks.csv population.
For private dwellings, household size counts the persons who share
`id_vivienda` and `id_hogar`. Other census operations have unknown size.
The file lists the persons of one household on consecutive rows.

Age bands: 0 = under 15, 1 = 15-24, 2 = 25-44, 3 = 45-64, 4 = 65 or more.
Household size: 1 to 5, where 5 means 5 or more. 255 means unknown.

Output: data/mvp/person_traits_workers.csv   origin_cut,destination_cut,mode,age_band,household_size,commuters
        data/mvp/person_traits_students.csv  cut,age_band,household_size,students
        data/mvp/person_traits_residents.csv cut,role,age_band,household_size,persons
          RM residents who are neither an internal commuter of the workers file nor a basic/secondary student.
          role: 0 under 15, 1 higher education (`asistencia_superior`), 2 employed without an internal
          commute (`sit_fuerza_trabajo` 1: works at home, outside the RM or without a fixed place),
          3 unemployed (2), 4 outside the labour force (3), 5 unknown.
"""
import collections, csv, pathlib, subprocess, sys

ZIP = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "data/raw/censo2024-personas/personas_censo2024.zip")
MISSING = {"", "-99", "998", "999"}
PRIVATE = "2"
UNKNOWN = 255


def band(age):
    return 0 if age < 15 else 1 if age < 25 else 2 if age < 45 else 3 if age < 65 else 4


workers = collections.Counter()
residents = collections.Counter()
students = collections.Counter()
tipo = collections.Counter()
sizes = collections.Counter()


def flush(household):
    private = household[0]["tipo_operativo"] == PRIVATE
    size = min(len(household), 5) if private else UNKNOWN
    if private:
        sizes[size] += 1
    for row in household:
        home = f"{int(row['comuna']):05d}"
        age = band(int(row["edad"]))
        dest = row["p44_lug_trab_esp"]
        commuter = row["p44_lug_trab"] in ("2", "3") and dest not in MISSING and not dest.zfill(5).endswith("000")
        if commuter:
            workers[(home, f"{int(dest):05d}", row["p45_medio_transporte"], age, size)] += 1
        student = row["asistencia_basica"] == "1" or row["asistencia_media"] == "1"
        if student:
            students[(home, age, size)] += 1
        if not student and not (commuter and dest.zfill(5).startswith("13")):
            role = 0 if age == 0 else 1 if row["asistencia_superior"] == "1" else \
                {"1": 2, "2": 3, "3": 4}.get(row["sit_fuerza_trabajo"], 5)
            residents[(home, role, age, size)] += 1


proc = subprocess.Popen(["unzip", "-p", str(ZIP), "personas_censo2024.csv"], stdout=subprocess.PIPE)
reader = csv.DictReader((l.decode("utf-8", "replace") for l in proc.stdout), delimiter=";")
household, key = [], None
for row in reader:
    if not row["comuna"].startswith("13"):
        continue
    tipo[row["tipo_operativo"]] += 1
    k = (row["comuna"], row["tipo_operativo"], row["id_vivienda"], row["id_hogar"])
    if k != key and household:
        flush(household)
        household = []
    key = k
    household.append(row)
flush(household)

print("persons by tipo_operativo:", dict(tipo))
print("private households by size (5 = 5+):", dict(sorted(sizes.items())))
print("persons per private household:", tipo[PRIVATE] / sum(sizes.values()))

with open("data/mvp/person_traits_workers.csv", "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["origin_cut", "destination_cut", "mode", "age_band", "household_size", "commuters"])
    for k, n in sorted(workers.items()):
        w.writerow(list(k) + [n])
print("workers", len(workers), "rows", sum(workers.values()))

with open("data/mvp/person_traits_students.csv", "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["cut", "age_band", "household_size", "students"])
    for k, n in sorted(students.items()):
        w.writerow(list(k) + [n])
print("students", len(students), "rows", sum(students.values()))

with open("data/mvp/person_traits_residents.csv", "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["cut", "role", "age_band", "household_size", "persons"])
    for k, n in sorted(residents.items()):
        w.writerow(list(k) + [n])
print("residents", len(residents), "rows", sum(residents.values()))
