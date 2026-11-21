"""Build the polygon, box, and block-ID files together in a new output directory."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

urban_pbf, census_gpkg, output = (Path(value).resolve() for value in sys.argv[1:])
tools = Path(__file__).resolve().parent
repo = tools.parents[1]
# Keep every intermediate private until all three files are complete.
with tempfile.TemporaryDirectory(dir=output.parent) as tmp:
    work = Path(tmp)
    osm = work / "data/raw/osm-buildings-city"
    fill = work / "data/raw/building-fill"
    web = work / "web/data"
    osm.mkdir(parents=True)
    web.mkdir(parents=True)
    (work / "data/mvp").symlink_to(repo / "data/mvp")
    subprocess.run(["osmium", "tags-filter", urban_pbf, "wr/building", "-o", osm / "buildings.osm.pbf"], check=True)
    subprocess.run(["osmium", "export", "-u", "type_id", "--geometry-types=polygon", "-f", "geojsonseq",
                    osm / "buildings.osm.pbf", "-o", osm / "polygons.geojsonseq"], check=True)
    for script, args in (
        ("building_mesh.py", ["prepare"]),
        ("building_fill.py", [census_gpkg, fill / "real_rendered.geojsonseq", fill]),
        ("building_mesh.py", ["pack"]),
        ("building_block_ids.py", [fill]),
    ):
        subprocess.run([sys.executable, "-B", tools / script, *args], cwd=work, check=True)
    os.rename(web, output)
