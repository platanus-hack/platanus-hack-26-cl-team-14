"""Prepare Census block assignments and synthetic footprints for the city mesh."""
from pathlib import Path
import json
import subprocess
import sys

GREEN = (
    "wr/leisure=park,garden,nature_reserve",
    "wr/landuse=recreation_ground,forest,meadow,grass,village_green",
    "wr/natural=wood,grassland,scrub,heath",
    "wr/landcover=trees,grass,scrub",
)
GROUNDS = (
    "wr/amenity=school,kindergarten,college,university,parking",
    "wr/landuse=education,vacant,brownfield,greenfield",
    "wr/place=square",
    "wr/leisure=pitch,track,sports_centre,sports_hall,stadium",
)


def extract(urban_pbf: Path, output: Path) -> Path:
    pbf = output / "grounds.osm.pbf"
    geo = output / "grounds.geojsonseq"
    subprocess.run(["osmium", "tags-filter", urban_pbf, *GREEN, *GROUNDS, "-O", "-o", pbf], check=True)
    config = output / "ground-tags.json"
    tags = ["area", "amenity", "landuse", "place", "leisure", "natural", "landcover", "indoor",
            "parking", "covered", "building", "building:part", "location", "layer"]
    config.write_text(json.dumps({"include_tags": tags}))
    subprocess.run(["osmium", "export", "-c", config, "-u", "type_id", "--geometry-types", "polygon",
                    "-f", "geojsonseq", "-O", "-o", geo, pbf], check=True)
    return geo


# Usage: python3 -I tools/pack/building_fill.py GPKG RENDERED_OSM_GEOJSONSEQ OUTPUT_DIRECTORY [URBAN_PBF]
cartography, osm, output = map(Path, sys.argv[1:4])
output.mkdir(parents=True, exist_ok=True)


def sql_path(path):
    return "'" + str(path.resolve()).replace("'", "''") + "'"


urban_pbf = Path(sys.argv[4]) if len(sys.argv) > 4 else Path("data/raw/osm-buildings-city/urban.osm.pbf")
parameters = {
    "grounds": extract(urban_pbf, output),
    "cartography": cartography, "components": osm,
    "blocks": Path("data/mvp/blocks.csv"), "jobs": Path("data/mvp/jobs.csv"),
    "real_blocks": output / "real_blocks.csv", "generated": output / "generated.geojsonseq",
    "coverage": output / "coverage.csv",
}
sql = Path(__file__).with_suffix(".sql").read_text()
for key, path in parameters.items():
    sql = sql.replace("@" + key + "@", sql_path(path))
subprocess.run(["duckdb", "-bail", "-no-agent", str(output / "fill.duckdb")], input=sql, text=True, check=True)
