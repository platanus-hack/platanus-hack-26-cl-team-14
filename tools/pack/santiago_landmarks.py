"""Export nine public Santiago landmarks from the cached November 2026 urban OSM extract."""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

from roads import clipped_segments, rounded_path, write_json

PBF = sys.argv[1] if len(sys.argv) > 1 else "data/raw/osm-buildings-city/urban.osm.pbf"
OUT = Path("web/data/santiago_landmarks.geojson")
PLACES = (
    ("gran_torre_costanera", "tower", "w1179710949", "https://www.skycostanera.cl/es"),
    ("estadio_nacional", "stadium", "w28092600", "https://www.bcn.cl/leychile/navegar?idNorma=273588"),
    ("la_moneda", "palace", "r394238", "https://ide.patrimoniocultural.gob.cl/patrimonio/909"),
    ("san_cristobal", "peak", "n285901766", "https://mapas.parquemet.gob.cl/doc/MAP_ParqueMet_es.pdf"),
    ("santa_lucia", "peak", "n314997482", "https://www.santiagoturismo.cl/es/cerro-santa-lucia/"),
    ("parque_forestal", "park", "w332070819", "https://www.santiagoturismo.cl/es/parque-forestal/"),
    ("parque_ohiggins", "park", "w14817188", "https://www.santiagoturismo.cl/es/parque-ohiggins/"),
)
ARENA = ("movistar_arena", "arena", "w32864171", "https://www.santanderarena.cl/sobre-nosotros")
# Main-stream members of OSM waterway relation 7548664 inside the view, in river order.
MAPOCHO = (313436483, 1557777157, 1146079769, 32920812, 607853671,
           23393901, 1570502197, 1570502190, 22716905, 22700378)
RIVER_SOURCE = "https://sistemahumedales.mma.gob.cl/HumedalesUrbanos/DetailsPublico/114"


def feature(place_id, name, kind, osm, source, geometry):
    osm_type = {"n": "node", "w": "way", "r": "relation"}[osm[0]]
    props = {"id": place_id, "name": name, "kind": kind, "osm_type": osm_type,
             "osm_id": int(osm[1:]), "source_url": source}
    return {"type": "Feature", "id": place_id, "geometry": geometry, "properties": props}


ids = [p[2] for p in (*PLACES, ARENA)] + [f"w{n}" for n in MAPOCHO]
with tempfile.NamedTemporaryFile(suffix=".osm.pbf") as selected:
    subprocess.run(["osmium", "getid", "-r", PBF, *ids, "-f", "pbf"], stdout=selected, check=True)
    stream = subprocess.check_output(["osmium", "export", "-u", "type_id", "-f", "geojsonseq", selected.name])
features = [json.loads(line.lstrip(b"\x1e")) for line in stream.split(b"\n") if line]
objects = {f["id"]: f for f in features}


def place(place_id, kind, osm, source):
    if kind == "peak":
        obj = objects[osm]
        shape = {"type": "Point", "coordinates": [round(c, 5) for c in obj["geometry"]["coordinates"]]}
    else:
        # Osmium uses doubled OSM IDs for areas, with the low bit set for relations.
        obj = objects[f"a{2 * int(osm[1:]) + (osm[0] == 'r')}"]
        rings = obj["geometry"]["coordinates"][0]
        tolerance = 4 if kind == "park" else 1
        shape = {"type": "Polygon", "coordinates": [rounded_path(ring, tolerance) for ring in rings]}
    tags = obj["properties"]
    item = feature(place_id, tags["name"], kind, osm, source, shape)
    for tag, field in (("height", "height_m"), ("building:levels", "levels"), ("ele", "elevation_m")):
        if tag in tags:
            item["properties"][field] = float(tags[tag])
    if kind == "arena":
        item["properties"].update(former_name=tags["old_name:2008-2026"],
                                  roof_color=tags["roof:colour"], wikidata=tags["wikidata"],
                                  height_m=50, height_source="venue_operator", height_scope="dome")
    return item

landmarks = [place(*entry) for entry in PLACES]
course = []
for n in MAPOCHO:
    line = objects[f"w{n}"]["geometry"]["coordinates"]
    for segment in clipped_segments(line):
        course.extend(segment[1:] if course else segment)
river = feature("rio_mapocho", "Río Mapocho", "river", "r7548664", RIVER_SOURCE,
                {"type": "LineString", "coordinates": rounded_path(course, 10)})
river["properties"]["osm_way_ids"] = MAPOCHO
landmarks.extend((river, place(*ARENA)))
write_json(OUT, {"type": "FeatureCollection", "attribution": "© OpenStreetMap contributors",
                 "license": "ODbL-1.0", "osm_timestamp": "2026-11-20T21:21:21Z", "features": landmarks})
print(f"{len(landmarks)} landmarks, {OUT.stat().st_size} bytes")
