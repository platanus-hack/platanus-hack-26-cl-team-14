"""Prepare Census block assignments and synthetic footprints for the city mesh."""
from pathlib import Path
import subprocess
import sys

# Usage: python3 -I tools/pack/building_fill.py GPKG RENDERED_OSM_GEOJSONSEQ OUTPUT_DIRECTORY
cartography, osm, output = map(Path, sys.argv[1:])
output.mkdir(parents=True, exist_ok=True)
def sql_path(path):
    return "'" + str(path.resolve()).replace("'", "''") + "'"


parameters = {
    "cartography": cartography, "components": osm,
    "blocks": Path("data/mvp/blocks.csv"), "jobs": Path("data/mvp/jobs.csv"),
    "real_blocks": output / "real_blocks.csv", "generated": output / "generated.geojsonseq",
    "coverage": output / "coverage.csv",
}
sql = Path(__file__).with_suffix(".sql").read_text()
for key, path in parameters.items():
    sql = sql.replace("@" + key + "@", sql_path(path))
subprocess.run(["duckdb", "-bail", "-no-agent", str(output / "fill.duckdb")], input=sql, text=True, check=True)
