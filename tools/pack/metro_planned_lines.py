"""Read the official Metro project plans for lines 8 and 9. See metro_planned.py."""

import hashlib
import math
import re
import subprocess
from io import BytesIO
from pathlib import Path
from xml.etree import ElementTree as ET
from zipfile import ZipFile

RAW = Path(__file__).resolve().parents[2] / "data/raw/metro_planned"
SEA = "https://seia.sea.gob.cl/archivos/"
PLAN_8 = SEA + "2026/02/24/Anexo_01-03_Actualizacion_del_KMZ_del_Proyecto.zip?download=1"
PLAN_9 = SEA + "2025/03/03/Anexo_22_Kmz_del_Proyecto.zip?download=1"
PLAN_9_SOUTH = SEA + "2026/06/30/Anexo_41_Act_Planos_y_KMZ_del_Proyecto.zip?download=1"
YEARS_8 = "https://www.monumentos.gob.cl/sites/default/files/2025-06/20240911_acta_sesion_ext_env_def_f.pdf"
YEARS_9 = "https://www.dtpm.cl/index.php/component/content/category/113-servicios"
ALT_9 = "https://dprmetropolitana.dpr.gob.cl/2026/03/19/delegado-presidencial-participo-en-importante-encuentro-de-tuneles-de-la-linea-7-del-metro/"
STATUS_8 = "https://www.sea.gob.cl/noticias/mayo-registra-record-historico-en-monto-de-proyectos-aprobados-ambientalmente-usd13962"
STATUS_9_SOUTH = "https://dprmetropolitana.dpr.gob.cl/2026/11/19/se-aprueba-extension-de-linea-9-del-metro-trayecto-la-pintana-puente-alto/"
MINVU_POINTS = "https://geoide.minvu.cl/server2/rest/services/INFORMACION_EXTERNA/Metro/FeatureServer/0"

# Names in the order of the main station footprints E01 through E14.
NAMES_8 = [
    "Terminal Sur", "Diego Portales", "Trinidad", "Rojas Magallanes", "Walker Martínez",
    "Macul", "Las Torres", "Punta de Rieles", "Rodrigo de Araya", "Grecia",
    "Chile España", "Diagonal Oriente", "Eliodoro Yáñez", "Los Leones",
]
NAMES_9_SOUTH = ["La Primavera", "Eyzaguirre", "Juanita", "Ejército", "Plaza de Puente Alto"]


def plan(url, filename, sha256):
    RAW.mkdir(parents=True, exist_ok=True)
    path = RAW / filename
    if not path.exists():
        subprocess.run(["curl", "-fsSL", "--max-filesize", "200000000", url, "-o", str(path)], check=True)
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != sha256:
        raise ValueError(f"Metro source changed: {url}")
    with ZipFile(BytesIO(data)) as outer:
        kmz = next(n for n in outer.namelist() if n.endswith(".kmz"))
        with ZipFile(BytesIO(outer.read(kmz))) as inner:
            return ET.fromstring(inner.read("doc.kml"))


def marks(root, folder):
    return root.findall(f".//{{*}}Folder[{{*}}name='{folder}']/{{*}}Placemark")


def coordinates(node):
    return [[round(float(v), 6) for v in item.split(",")[:2]] for item in node.text.split()]


def point_or_center(mark):
    node = mark.find(".//{*}Point/{*}coordinates")
    if node is not None:
        return coordinates(node)[0], "Metro station label in the design KMZ; positional accuracy unspecified."
    ring = coordinates(mark.find(".//{*}outerBoundaryIs//{*}coordinates"))
    bounds = [[min(axis), max(axis)] for axis in zip(*ring)]
    point = [round((low + high) / 2, 6) for low, high in bounds]
    radius = math.ceil(math.hypot((bounds[0][1] - point[0]) * 93000, (bounds[1][1] - point[1]) * 111200))
    return point, f"Center of the station footprint bounds; corners within {radius} m. Survey accuracy unspecified."


def trim_start(path, point):
    """Trim the terminal tail at the point's projection onto the tunnel."""
    def projection(i):
        a, b = path[i], path[i + 1]
        dx, dy = (b[0] - a[0]) * 0.834, b[1] - a[1]
        px, py = (point[0] - a[0]) * 0.834, point[1] - a[1]
        length2 = dx * dx + dy * dy
        t = max(0, min(1, (px * dx + py * dy) / length2)) if length2 else 0
        return (px - t * dx) ** 2 + (py - t * dy) ** 2, i, t

    _, i, t = min(map(projection, range(len(path) - 1)))
    a, b = path[i], path[i + 1]
    start = [round(low + t * (high - low), 6) for low, high in zip(a, b)]
    return [start] + path[i + 1:]


def feature(kind, geometry, **properties):
    return {"type": "Feature", "properties": {"kind": kind, **properties}, "geometry": geometry}


def features():
    l8 = plan(PLAN_8, "v16.bin", "c579f9d3ebfd2a3b5c5122a3dcbdafd171255794df75cc1bd02466a3ca2a9efe")
    l9 = plan(PLAN_9, "m9.bin", "1aa15fa8b4ca5efaf82f4e04326afbd375fe4db2b2e8f5bcdf01770039ef8c39")
    south = plan(PLAN_9_SOUTH, "v3.bin", "cf6b783471c7dddaaee85814cf8da61004dbbfb24b416138554cc3cb5e4e3ec8")
    stations = {8: [], 9: []}
    for mark in marks(l8, "Estaciones"):
        label = mark.findtext("{*}name")
        if "Acceso" in label or "Auxiliar" in label:
            continue
        number = int(label[1:3])
        stations[8].append((number, NAMES_8[number - 1], label, mark, PLAN_8))
    # Stage 2 runs from Cal y Canto to the north of Bío Bío. Stage 1 ends at Gabriela.
    ordered = sorted(marks(l9, "Estaciones"), key=lambda m: -point_or_center(m)[0][1])
    for number, mark in enumerate(ordered, 1):
        label = mark.findtext("{*}name")
        name = re.sub(r"^(Estación |E\d+\.? )", "", label).removesuffix(" - PC-14")
        stations[9].append((number, name, label, mark, PLAN_9))
    for mark in marks(south, "Estaciones"):
        label = mark.findtext("{*}name")
        number = int(label.split()[1])
        stations[9].append((number, NAMES_9_SOUTH[number - 15], label, mark, PLAN_9_SOUTH))

    output = []
    endpoints = {}
    for line, rows in stations.items():
        coords = []
        for order, name, label, mark, source in sorted(rows):
            point, precision = point_or_center(mark)
            coords.append(point)
            if line == 8:
                phase = 1 if order <= 11 else 2
                year = 2032 if phase == 1 else 2033
            else:
                phase = 2 if order <= 4 else (1 if order <= 14 else 3)
                year = {1: 2030, 2: 2032, 3: 2033}[phase]
            props = dict(id=f"l{line}-{order:02}", line=f"L{line}", order=order, name=name,
                         name_type="project label", source_label=label, source=source, geometry_source=source,
                         precision=precision, phase=phase, opening_year=year,
                         opening_year_source=YEARS_8 if line == 8 else YEARS_9,
                         status="under_construction" if line == 9 and phase == 1 else "approved")
            if line == 8:
                props["status_source"] = STATUS_8
            if line == 9 and phase == 2:
                props.update(alternative_opening_year=2033, alternative_opening_year_source=ALT_9)
            if line == 9 and phase == 3:
                props.update(status_source=STATUS_9_SOUTH, name_source=MINVU_POINTS if order < 19 else STATUS_9_SOUTH)
            output.append(feature("station", {"type": "Point", "coordinates": point}, **props))
        endpoints[line] = coords

    # The longest L8 axis is the main tunnel. Remove both terminal tails.
    axes = marks(l8, "Eje Línea 8")[0].findall(".//{*}LineString/{*}coordinates")
    path8 = max(map(coordinates, axes), key=len)[::-1]
    path8 = trim_start(path8, endpoints[8][0])
    path8 = trim_start(path8[::-1], endpoints[8][-1])[::-1]
    # The final five L9 centerline records contain the full main alignment and the separate depot links.
    axes9 = marks(l9, "Eje Túnel")[-5:]
    north, middle, southern = [coordinates(axes9[i].find(".//{*}LineString/{*}coordinates")) for i in (0, 1, 3)]
    north.reverse()
    southern.reverse()
    # The extension KMZ publishes a tunnel footprint. Its outward side runs toward Puente Alto.
    outline = coordinates(marks(south, "Trazado")[0].find(".//{*}outerBoundaryIs//{*}coordinates"))
    edge = outline[:len(outline) // 2 + 1]
    parts9 = [trim_start(north, endpoints[9][0]), middle, southern,
              trim_start(edge[::-1], endpoints[9][-1])[::-1]]
    for line, geom, sources, context in [
        (8, {"type": "LineString", "coordinates": path8}, [PLAN_8], "Metro main tunnel axis cropped at the positions nearest to the mapped terminal stations."),
        (9, {"type": "MultiLineString", "coordinates": parts9}, [PLAN_9, PLAN_9_SOUTH],
         "Three Metro centerline sections and the outward edge of the Stage 3 tunnel footprint. Terminal station points are projected onto the path. The source sections have small gaps or overlaps."),
    ]:
        output.append(feature("line", geom, id=f"l{line}", line=f"L{line}", name=f"Línea {line}",
                              status="approved" if line == 8 else "partly_under_construction",
                              source=sources[0], geometry_source=sources, precision=context,
                              opening_year=None, note="Stages open separately. See station years."))
    return output
