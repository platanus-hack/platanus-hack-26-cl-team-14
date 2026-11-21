"""Line 7 from Metro's updated design in SEA Annex 14, January 2021."""
import io
import subprocess
import xml.etree.ElementTree as ET
from pathlib import Path
from zipfile import ZipFile

SOURCE = "https://seia.sea.gob.cl/archivos/2021/01/12/52a_Anexo_14_KMZ_y_Planos_Actualizados_del_Proyecto__Parte_1_.zip"
SCHEDULE = "https://dprmetropolitana.dpr.gob.cl/2026/03/19/delegado-presidencial-participo-en-importante-encuentro-de-tuneles-de-la-linea-7-del-metro/"
RAW = Path(__file__).resolve().parents[2] / "data/raw/metro-futuro"
KML = "{http://www.opengis.net/kml/2.2}"

# Public design labels in order from Renca to Vitacura.
STATIONS = [
    ("Brasil", "Estación Brasil"),
    ("José Miguel Infante", "Estación José Miguel Infante"),
    ("Salvador Gutiérrez", "Estación Salvador Gutierrez"),
    ("Huelén", "Estación Huelen"),
    ("Neptuno", "Estación Neptuno"),
    ("Radal", "Estación Radal"),
    ("Walker Martínez", "Estación Walker Martinez"),
    ("Matucana", "Estación Matucana"),
    ("Ricardo Cumming", "Estación Ricardo Cumming"),
    ("Puente Cal y Canto", "Estación Cal y Canto Principal"),
    ("Baquedano", "Acceso Existente Baquedano"),
    ("Pedro de Valdivia", "Estación Pedro de Valdivia Principal"),
    ("Isidora Goyenechea", "Estación Isidora Goyenechea Principal"),
    ("Vitacura", "Estación Vitacura"),
    ("Américo Vespucio", "Estación Americo Vespucio Principal"),
    ("Parque Araucano", "Estación Parque Araucano"),
    ("Gerónimo de Alderete", "Estación Geronimo Alderete"),
    ("Padre Hurtado", "Estación Padre Hurtado"),
    ("Estoril", "Estación Estoril Norte"),
]


def coordinates(element):
    return [[round(float(v), 6) for v in point.split(",")[:2]]
            for point in element.findtext(KML + "coordinates").split()]


def features():
    RAW.mkdir(exist_ok=True, parents=True)
    archive = RAW / SOURCE.rsplit("/", 1)[1]
    if not archive.exists():
        subprocess.run(["curl", "-fLsS", SOURCE + "?download=1", "-o", str(archive)], check=True)
    with ZipFile(archive) as outer:
        kmz = outer.read("1. KMZ Actualizado Proyecto L7.kmz")
    with ZipFile(io.BytesIO(kmz)) as inner, inner.open("doc.kml") as kml:
        root = ET.parse(kml)
    folders = {folder.findtext(KML + "name"): folder for folder in root.iter(KML + "Folder")}
    labels = {p.findtext(KML + "name"): coordinates(p.find(KML + "Point"))[0]
              for p in folders["Etiquetas Fase de Operación"].findall(KML + "Placemark")}
    common = dict(line="L7", corridor="L7", status="under_construction", source=SOURCE,
                  opening_year=2028, opening_year_source=SCHEDULE)
    result = []
    for order, (name, label) in enumerate(STATIONS, 1):
        props = dict(common, kind="station", id=f"L7-{order:02}", order=order, name=name,
                     source_label=label, geometry_source=SOURCE, precision="SEA operation-phase map label. Positional accuracy unspecified.",
                     name_status="design label", source_layer="Etiquetas Fase de Operación")
        result.append(dict(type="Feature", properties=props, geometry=dict(type="Point", coordinates=labels[label])))

    # Annex 14 includes a depot spur, overlapping CAD lines and terminal details.
    # Keep one centreline per passenger-corridor segment; preserve the source breaks.
    segments = list(folders["Eje Trazado L7"].iter(KML + "LineString"))
    omit = {13, 17, 20, 23, 28, 30, 35, 38, 39, 42, 44, 46, 47}
    paths = [coordinates(segment) for i, segment in enumerate(segments) if i not in omit]
    result.append(dict(type="Feature", geometry=dict(type="MultiLineString", coordinates=paths),
                       properties=dict(common, kind="line", name="Línea 7", id="L7",
                                       geometry_source=SOURCE, source_layer="Eje Trazado L7; includes terminal tails",
                                       precision="Engineering GIS. Positional accuracy unspecified.",
                                       projected_terminal_minutes=37)))
    return result
