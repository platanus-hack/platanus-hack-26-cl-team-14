"""Extract the stadium shell, pitch, athletics track and four lighting towers."""
import json
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory

from roads import write_json

PBF = sys.argv[1] if len(sys.argv) > 1 else "data/raw/osm-buildings-city/urban.osm.pbf"
OUT = Path("web/data/estadio_nacional_parts.geojson")
MOP = "https://arquitectura.mop.gob.cl/uploads/sites/6/2024/11/estadio-nacional_bajo.pdf#page=6"
PARTS = [
    ("stadium_shell", "r8647176", "Coliseo Central"),
    ("pitch", "w52100528", "Cancha de fútbol"),
    ("track", "r8647177", "Pista de atletismo"),
    ("floodlight", "n3718842223", "Torre de iluminación sureste"),
    ("floodlight", "n3718842224", "Torre de iluminación noroeste"),
    ("floodlight", "n3718862025", "Torre de iluminación suroeste"),
    ("floodlight", "n3718862029", "Torre de iluminación noreste"),
]

with TemporaryDirectory() as temp:
    selected = Path(temp) / "stadium.osm.pbf"
    subprocess.run(["osmium", "getid", "-r", PBF, *(osm for _, osm, _ in PARTS), "-o", selected], check=True)
    raw = subprocess.check_output(["osmium", "export", "-u", "type_id", "-f", "geojsonseq", selected])
features = (json.loads(line.lstrip(b"\x1e")) for line in raw.split(b"\n") if line)
objects = {feature["id"]: feature for feature in features}

parts = []
for kind, osm, name in PARTS:
    osm_type = {"r": "relation", "w": "way", "n": "node"}[osm[0]]
    number = int(osm[1:])
    if osm_type == "node":
        obj = objects[osm]
        geometry = obj["geometry"]
    else:
        obj = objects[f"a{2*number + (osm_type == 'relation')}"]
        geometry = {"type": "Polygon", "coordinates": obj["geometry"]["coordinates"][0]}
    props = {"parent_id": "estadio_nacional", "kind": kind, "name": name, "osm_type": osm_type,
             "osm_id": number, "geometry_source": f"https://www.openstreetmap.org/{osm_type}/{number}"}
    if kind == "stadium_shell":
        props.update(height_m=float(obj["properties"]["height"]), height_source="osm_height_tag")
    if kind == "floodlight":
        props.update(height_m=54, height_source="mop_1962_works", height_source_url=MOP)
    parts.append({"type": "Feature", "id": f"{kind}_{number}", "properties": props, "geometry": geometry})

write_json(OUT, {"type": "FeatureCollection", "attribution": "© OpenStreetMap contributors",
                 "license": "ODbL-1.0", "osm_timestamp": "2026-11-20T21:21:21Z", "features": parts})
print(f"{len(parts)} parts, {OUT.stat().st_size:,} bytes")
