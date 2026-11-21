"""Build the Santiago ground layers from the cached urban OSM extract."""
import gzip
import json
import subprocess
import sys
from collections import Counter
from pathlib import Path

from roads import VIEW

PBF = sys.argv[1] if len(sys.argv) > 1 else "data/raw/osm-buildings-city/urban.osm.pbf"
RAW = Path("data/raw/osm-city-ground")
DEST = Path("web/data")
STREETS = ("w/highway=residential,living_street",)
GREEN = ("wr/leisure=park,garden,nature_reserve", "wr/landuse=recreation_ground,forest,meadow,grass,village_green",
         "wr/natural=wood,grassland,scrub,heath", "wr/landcover=trees,grass,scrub")
GREEN_KIND = """CASE
  WHEN leisure IN ('park','garden','nature_reserve') THEN leisure
  WHEN landuse IN ('recreation_ground','meadow','grass','village_green') THEN landuse
  WHEN landuse='forest' OR "natural"='wood' OR landcover='trees' THEN 'wood'
  WHEN "natural" IN ('grassland','scrub','heath') THEN "natural"
  WHEN landcover IN ('grass','scrub') THEN landcover END"""


def coordinates(points):
    return [[round(x, 5), round(y, 5)] for x, y in points]


def build(name, filters, kind, area):
    source = RAW / (name + ".osm.pbf")
    geo = RAW / (name + ".geojsonseq")
    parts = RAW / (name + ".parts.tsv")
    subprocess.run(["osmium", "tags-filter", PBF, *filters, "-O", "-o", source], check=True)
    dimension = "polygon" if area else "linestring"
    subprocess.run(["osmium", "export", "-u", "type_id", "--geometry-types", dimension,
                    "-f", "geojsonseq", "-O", "-o", geo, source], check=True)
    sql = f"""
LOAD spatial;
SET threads=3;
CREATE MACRO bbox() AS ST_MakeEnvelope({','.join(map(str, VIEW))});
CREATE MACRO meters(g) AS ST_Transform(g, 'EPSG:4326','EPSG:32719',always_xy:=true);
CREATE MACRO degrees(g) AS ST_Transform(g, 'EPSG:32719','EPSG:4326',always_xy:=true);
COPY (
  WITH selected AS (
    SELECT id, name, {kind} AS kind, geom::GEOMETRY AS geom FROM ST_Read('{geo}')
  ), clipped AS (
    SELECT id,name,kind,ST_Intersection(ST_MakeValid(geom),bbox()) AS geom
    FROM selected WHERE kind IS NOT NULL AND ST_Intersects(geom,bbox())
  ), simplified AS (
    SELECT id,name,kind,degrees(ST_SimplifyPreserveTopology(meters(geom),{4 if area else 3})) geom FROM clipped
  ), pieces AS (
    SELECT id,name,kind,unnest(ST_Dump(ST_CollectionExtract(
      ST_ReducePrecision(ST_Intersection(geom,bbox()),0.00001),{3 if area else 2}))) AS part FROM simplified
  )
  SELECT json_object('id',id,'kind',kind,'name',name,'geometry',ST_AsGeoJSON(part.geom)::JSON)::VARCHAR
  FROM pieces WHERE NOT ST_IsEmpty(part.geom) ORDER BY id,part.path
) TO '{parts}' (FORMAT CSV, HEADER false, QUOTE '', ESCAPE '', DELIMITER '\t');
"""
    subprocess.run(["duckdb", "-no-agent", "-bail"], input=sql, text=True, check=True)
    counts = Counter()
    vertices = total = 0
    out = DEST / (name + ".geojson.gz")
    header = {"type": "FeatureCollection", "attribution": "© OpenStreetMap contributors",
              "license": "ODbL-1.0", "osm_timestamp": "2026-11-20T21:21:21Z"}
    with out.open("wb") as file, gzip.GzipFile(filename="", fileobj=file, mode="wb", mtime=0, compresslevel=6) as gz:
        gz.write((json.dumps(header, ensure_ascii=False, separators=(",", ":"))[:-1] + ',"features":[').encode())
        with parts.open() as src:
            for row in src:
                rec = json.loads(row)
                osm_id = int(rec["id"][1:])
                osm_type = "way"
                if area:
                    if osm_id % 2:
                        osm_type = "relation"
                    osm_id //= 2
                props = {"osm_id": osm_id, "osm_type": osm_type, "kind": rec["kind"]}
                if rec["name"]:
                    props["name"] = rec["name"]
                geom = rec["geometry"]
                coords = geom["coordinates"]
                geom["coordinates"] = [coordinates(ring) for ring in coords] if area else coordinates(coords)
                feat = {"type": "Feature", "properties": props, "geometry": geom}
                if total:
                    gz.write(b",")
                gz.write(json.dumps(feat, ensure_ascii=False, separators=(",", ":")).encode())
                total += 1
                counts[rec["kind"]] += 1
                vertices += sum(map(len, coords)) if area else len(coords)
        gz.write(b"]}\n")
        raw_bytes = gz.tell()
    print(f"{name}: {total:,} parts, {vertices:,} vertices, {raw_bytes:,} raw bytes, "
          f"{out.stat().st_size:,} gzip bytes. {dict(counts)}")


if __name__ == "__main__":
    RAW.mkdir(parents=True, exist_ok=True)
    build("residential_streets", STREETS, "highway", False)
    build("green_areas", GREEN, GREEN_KIND, True)
