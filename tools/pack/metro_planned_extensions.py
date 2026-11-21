"""L6 extensions from Metro's public environmental filings. Requires curl and bsdtar."""

import io
import subprocess
import xml.etree.ElementTree as ET
from pathlib import Path
from zipfile import ZipFile

RAW = Path(__file__).resolve().parents[2] / "data/raw/metro_extension"
WEST = "https://seia.sea.gob.cl/archivos/2021/12/07/Anexo_1-2.zip"
EAST = "https://seia.sea.gob.cl/archivos/2019/06/20/Apendice_1.2.4_KMZ.zip?download=1"
WEST_PROJECT = "https://www.gob.cl/noticias/inicio-obras-extension-linea-6-metro-santiago-poniente-estacion-lo-errazuriz/"
UPDATED_WEST = "https://es.linkedin.com/posts/metro-de-santiago-s-a-_la-extensi%C3%B3n-de-l%C3%ADnea-6-hacia-el-poniente-activity-7500647166386913280-LViT"
EAST_SCHEDULE = "https://acen.it/wp-content/uploads/2026/04/Proyectos-Metro-de-Santiago-2026.pdf"


def project_kml(archive, url, suffix):
    path = RAW / archive
    if not path.exists():
        RAW.mkdir(parents=True, exist_ok=True)
        subprocess.run(["curl", "-fsSL", url, "-o", path], check=True)
    with ZipFile(path) as outer:
        data = outer.read(next(name for name in outer.namelist() if name.endswith(suffix)))
    if suffix == ".rar":
        data = subprocess.run(
            ["bsdtar", "-xOf", "-", "*.kmz"], input=data, capture_output=True, check=True
        ).stdout
    with ZipFile(io.BytesIO(data)) as kmz:
        return ET.fromstring(kmz.read("doc.kml"))


def coordinates(placemark):
    return [
        [float(v) for v in point.split(",")[:2]]
        for point in placemark.findtext(".//{*}coordinates").split()
    ]


def west_path(ring):
    # The annex supplies a tunnel footprint. It runs east-west without a switchback.
    # Take the middle of its north and south edges at intervals of about 25 m.
    lo, hi = min(x for x, y in ring), max(x for x, y in ring)
    steps = round((hi - lo) / 0.00027)
    path = []
    for i in range(steps):
        x = hi - (i + 0.5) * (hi - lo) / steps
        cross = [
            y1 + (x - x1) * (y2 - y1) / (x2 - x1)
            for (x1, y1), (x2, y2) in zip(ring, ring[1:])
            if min(x1, x2) <= x < max(x1, x2)
        ]
        path.append([x, (min(cross) + max(cross)) / 2])
    return path


def features():
    west = project_kml("west.zip", WEST, ".kmz")
    names = {p.findtext("{*}name"): p for p in west.findall(".//{*}Placemark")}
    footprint = coordinates(names["Estación Lo Errázuriz"])
    west_station = [(min(axis) + max(axis)) / 2 for axis in zip(*footprint)]
    east = project_kml("east.zip", EAST, ".rar")
    folders = {f.findtext("{*}name"): f for f in east.findall(".//{*}Folder")}
    labels = folders["Etiquetas Estaciones"].findall("{*}Placemark")
    label = next(p for p in labels if p.findtext("{*}name") == "Estacion Isidora Goyenechea")
    # The first two alignments form the separate L6 branch, from its existing end
    # to the new terminal and its turnback. The remaining alignments belong to L7.
    east_path = []
    for part in folders["Eje Trazado"].findall("{*}Placemark")[:2]:
        points = coordinates(part)
        east_path.extend(points[::25])
        east_path.append(points[-1])

    corridors = [
        ("west", "Lo Errázuriz", west_station, west_path(coordinates(names["Túnel"])), WEST,
         "Bounding-box centre of the station footprint in Metro's 2021 DIA, annex 1-2, appendix 2.",
         "Midpoints of tunnel footprint cross-sections about 25 m apart; covers the new tunnel including turnback.",
         WEST_PROJECT, UPDATED_WEST, "Metro gives 2028 in its video after the first tunnel breakthrough. The 2024 government release gave 2027."),
        ("east", "Isidora Goyenechea", coordinates(label)[0], east_path, EAST,
         "Station label point in Metro's 2019 L7 EIA, appendix 1.2.4; represents the L6/L7 interchange.",
         "L6 track axis in Metro's 2019 L7 EIA, appendix 1.2.4, sampled about every 25 m; includes turnback.",
         EAST_SCHEDULE, EAST_SCHEDULE, "Metro's March 2026 presentation groups L6 east with L7 and projects service in 2028."),
    ]
    result = []
    for side, name, point, route, source, point_note, route_note, project, schedule, schedule_note in corridors:
        corridor = "L6-" + side
        common = dict(line="L6", corridor=corridor, status="under_construction", source=project,
                      opening_year=2028, opening_year_source=schedule, opening_year_note=schedule_note,
                      geometry_source=source)
        for kind, label, shape, precision in [
            ("line", "Extensión Línea 6 " + ("poniente" if side == "west" else "oriente"),
             {"type": "LineString", "coordinates": [[round(x, 6), round(y, 6)] for x, y in route]}, route_note),
            ("station", name, {"type": "Point", "coordinates": [round(v, 6) for v in point]}, point_note),
        ]:
            fid = corridor if kind == "line" else corridor + "-01"
            properties = dict(common, kind=kind, id=fid, name=label,
                              precision=precision + " Source does not report positional accuracy.")
            if kind == "station":
                properties["order"] = 1
            result.append({"type": "Feature", "id": fid, "properties": properties, "geometry": shape})
    return result
