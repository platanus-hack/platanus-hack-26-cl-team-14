"""Build RM campus enrollment from aggregate SIES program counts and public teaching sites."""

import csv
import io
import json
import math
import re
import subprocess
import unicodedata
import urllib.parse
import zipfile
from collections import Counter, defaultdict
from pathlib import Path

RAW = Path("data/raw/higher_ed")
MVP = Path("data/mvp")
ENROLLMENT = "https://www.mifuturo.cl/wp-content/uploads/2026/07/Matricula_2026_WEB_10_07_2026.zip"
SITES = "https://ideserver.sma.gob.cl/arcgis/rest/services/IDE/Infraestructura_rural_urbana/MapServer/6"
MODES = ("Presencial", "Semipresencial", "No Presencial")


def normalized(text):
    return "".join(c for c in unicodedata.normalize("NFD", text.upper())
                   if not unicodedata.combining(c)).strip()


def campus_name(text):
    text = re.sub(r"^(SEDE |CAMPUS |INACAP )", "", normalized(text))
    return text.removesuffix(" (SANTIAGO)")


def distance(a, b):
    # Local east/north distance in metres at Santiago's latitude.
    return math.hypot((a[0] - b[0]) * 93000, (a[1] - b[1]) * 111000)


def campus_point(sites, campus):
    name = campus_name(campus)
    matches = []
    for site in sites:
        label = campus_name(site["attributes"]["NOMBRE_INM"])
        if label == name or re.fullmatch(re.escape(name) + r" (EDIFICIO .+|[0-9]+|[IVX-]+)", label):
            geometry = site["geometry"]
            matches.append((geometry["x"], geometry["y"]))
    points = sorted(set(matches))
    if points and max(distance(a, b) for a in points for b in points) <= 500:
        return min(points, key=lambda a: sum(distance(a, b) for b in points))
    return None


RAW.mkdir(parents=True, exist_ok=True)
archive = RAW / "matricula.zip"
if not archive.exists():
    subprocess.run(["curl", "-fsSL", "-o", archive, ENROLLMENT], check=True)
with zipfile.ZipFile(archive) as z:
    rar = z.read(z.namelist()[0])
data = subprocess.check_output(["bsdtar", "-xOf", "-"], input=rar).decode("latin1")
with (MVP / "communes.csv").open() as f:
    cuts = {normalized(row["name"]): row["cut"] for row in csv.DictReader(f)}

counts = defaultdict(Counter)
for row in csv.DictReader(io.StringIO(data), delimiter=";"):
    if row["REGIÓN"] == "Metropolitana":
        key = (int(row["CÓDIGO DE INSTITUCIÓN"]), row["NOMBRE INSTITUCIÓN"],
               row["NOMBRE SEDE"], cuts[normalized(row["COMUNA"])])
        counts[key][row["MODALIDAD"]] += int(row["TOTAL MATRÍCULA"])

query = urllib.parse.urlencode(dict(where="COD_REGION=13", outFields="COD_INST,NOMBRE_INM,COD_COMUNA",
                                    outSR=4326, f="json"))
site_file = RAW / "sites.json"
if not site_file.exists():
    subprocess.run(["curl", "-fsSL", "-o", site_file, f"{SITES}/query?{query}"], check=True)
sites = defaultdict(list)
institution_points = defaultdict(list)
for feature in json.loads(site_file.read_text())["features"]:
    attrs = feature["attributes"]
    sites[attrs["COD_INST"], str(attrs["COD_COMUNA"])].append(feature)
    geometry = feature["geometry"]
    institution_points[attrs["COD_INST"]].append((geometry["x"], geometry["y"]))

physical_campuses = Counter(code for (code, _, _, _), modes in counts.items()
                            if modes["Presencial"] + modes["Semipresencial"])

with Path("tools/pack/higher_ed_addresses.csv").open() as f:
    addresses = {(int(row["institution_code"]), row["campus"], row["cut"]): row
                 for row in csv.DictReader(f)}

with (MVP / "higher_ed.csv").open("w", newline="") as f:
    writer = csv.writer(f, lineterminator="\n")
    writer.writerow(["institution", "campus", "cut", "longitude", "latitude", "enrollment",
                     "year", "in_person", "hybrid", "remote", "point_source"])
    for (code, institution, campus, cut), modes in sorted(counts.items()):
        point, method = None, ""
        if modes["Presencial"] + modes["Semipresencial"]:
            address = addresses.get((code, campus, cut))
            if address:
                point = float(address["longitude"]), float(address["latitude"])
                method = "verified_address"
            else:
                point = campus_point(sites[code, cut], campus)
                # A single administrative campus can report registrations for distant sites.
                if (point and physical_campuses[code] == 1
                        and campus_name(campus) in {"CASA CENTRAL", "SANTIAGO"}
                        and any(distance(point, other) > 500 for other in institution_points[code])):
                    point = None
                if point:
                    method = "mineduc_2020_campus"
        lon, lat = (f"{v:.6f}" for v in point) if point else ("", "")
        writer.writerow([institution, campus, cut, lon, lat, sum(modes.values()), 2026,
                         *(modes[m] for m in MODES), method])
