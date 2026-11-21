"""Build tiled road surfaces from unsimplified Santiago OSM ways."""
import csv
import gzip
import json
import re
import subprocess
import sys
from pathlib import Path

from city_footprints import numeric

RAW = Path("data/raw/road-surface")
OUTPUT = Path("web/data/road_surface.geojson.gz")
# Width of the whole mapped way (one carriageway for divided roads), in metres.
WIDTHS = {
    "motorway": 7, "trunk": 7, "primary": 7, "secondary": 6.4, "tertiary": 6.4,
    "motorway_link": 3.6, "trunk_link": 3.6, "primary_link": 3.5,
    "secondary_link": 3.2, "tertiary_link": 3.2, "residential": 5.6, "living_street": 4,
    "unclassified": 5.6, "service": 3.5, "road": 5.6, "track": 3.0, "busway": 3.5,
    "pedestrian": 4, "footway": 1.8, "path": 1.8, "steps": 1.5, "bridleway": 2.5,
    "cycleway": 2.5,
}


def source_way(tags):
    highway = tags.get("highway")
    if highway not in WIDTHS or tags.get("area") == "yes" or tags.get("indoor") == "yes":
        return
    bridge = tags.get("bridge", "no") != "no"
    tunnel = tags.get("tunnel", "no") != "no"
    structure = "tunnel" if tunnel else "bridge" if bridge else "ground"
    default_layer = -1 if tunnel else 1 if bridge else 0
    text = tags.get("layer", "")
    layer = int(text) if re.fullmatch(r"[+-]?\d+", text) else default_layer
    if highway == "footway" and tags.get("footway") == "sidewalk":
        use = "sidewalk"
    elif highway == "cycleway":
        use = "cycle"
    elif highway in ("footway", "steps", "path", "pedestrian", "bridleway"):
        use = "foot"
    else:
        use = "road"
    width = numeric(tags.get("width", ""), units=True)
    if width:
        origin = "width"
    elif (lanes := tags.get("lanes", "")).isdigit() and int(lanes) > 0:
        width, origin = 3.2 * int(lanes), "lanes"
    else:
        width, origin = WIDTHS[highway], "class_default"
    return highway, structure, layer, use, width, origin


def main(pbf):
    RAW.mkdir(parents=True, exist_ok=True)
    extract = RAW / "ways.pbf"
    filtered = "w/highway=" + ",".join(WIDTHS)
    subprocess.run(["osmium", "tags-filter", pbf, filtered, "-O", "-o", extract], check=True)
    # Export unrounded OSM vertices. Buffer before any coordinate rounding.
    command = ["osmium", "export", "-u", "type_id", "--geometry-types", "linestring",
               "-f", "geojsonseq", extract]
    with subprocess.Popen(command, stdout=subprocess.PIPE, text=True) as process, (RAW / "ways.csv").open("w") as out:
        writer = csv.writer(out)
        writer.writerow(("osm_id","highway","structure","layer","road_use","width_m","width_source","geometry"))
        for line in process.stdout:
            feature = json.loads(line.lstrip("\x1e"))
            fields = source_way(feature["properties"])
            if fields:
                writer.writerow((feature["id"], *fields, json.dumps(feature["geometry"], separators=(",",":"))))
        if process.wait():
            raise RuntimeError("osmium export failed")
    with Path(__file__).with_suffix(".sql").open() as queries:
        subprocess.run(["duckdb", "-no-agent", "-bail", str(RAW / "surface.duckdb")], stdin=queries, check=True)
    header = {"type":"FeatureCollection", "attribution":"© OpenStreetMap contributors",
              "license":"ODbL-1.0", "osm_timestamp":"2026-11-20T21:21:21Z"}
    count = 0
    with (RAW / "features.tsv").open("rb") as features, OUTPUT.open("wb") as dest, gzip.GzipFile(
            filename="", fileobj=dest, mode="wb", mtime=0, compresslevel=6) as pack:
        pack.write(json.dumps(header, ensure_ascii=False, separators=(",",":")).encode()[:-1] + b',"features":[')
        for row in features:
            if count:
                pack.write(b",")
            pack.write(row.rstrip(b"\n"))
            count += 1
        pack.write(b"]}\n")
        raw_size = pack.tell()
    print(f"{count:,} polygons, {raw_size:,} inflated bytes, {OUTPUT.stat().st_size:,} gzip bytes")


if __name__ == "__main__":
    main(sys.argv[1])
