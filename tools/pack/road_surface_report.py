"""Compare displayed streets with the candidate road polygons at fixed camera locations."""
import csv
import gzip
import json
import shutil
import subprocess
import sys
from pathlib import Path

from road_surface import RAW, OUTPUT

def read_json(path):
    with (gzip.open(path, "rt") if path.suffix==".gz" else path.open()) as f:
        return json.load(f)


def movement_paths():
    local = read_json(Path("web/data/local_mover_routes.json.gz"))
    for mode, index in sorted({(g["mode"],g["route"]) for g in local["groups"]}):
        yield mode, "local_osrm", index, local["paths"][index]
    for route in read_json(Path("web/data/transit_paths.json")):
        if route["mode"]=="bus":
            yield "bus", "gtfs", route["route"], route["path"]


# Report inputs are local snapshots of geometry actually read by the two views.
with (RAW / "baseline.csv").open("w") as out:
    csv_out = csv.writer(out)
    csv_out.writerow(("width_m","geometry"))
    for path in (Path("data/mvp/roads.geojson"), Path("web/data/residential_streets.geojson.gz")):
        for f in read_json(path)["features"]:
            kind = f["properties"]["kind"]
            width = {"motorway":13,"trunk":10,"primary":10,"secondary":8}.get(kind,5.6)
            csv_out.writerow((width,json.dumps(f["geometry"],separators=(",",":"))))
with (RAW / "routes.csv").open("w") as out:
    csv_out = csv.writer(out)
    csv_out.writerow(("mode","source","route_id","geometry"))
    for mode,source,key,coords in movement_paths():
        csv_out.writerow((mode,source,key,json.dumps({"type":"LineString","coordinates":coords},separators=(",",":"))))
with gzip.open(OUTPUT,"rb") as src, (RAW / "published.geojson").open("wb") as out:
    shutil.copyfileobj(src,out)
# Camera teams can rerun this report with another CSV. A radius is a map-plane sample, in metres.
with (RAW / "cameras.csv").open("w") as out:
    rows = csv.writer(out)
    rows.writerow(("camera","lon","lat","radius_m"))
    if len(sys.argv)>1:
        with Path(sys.argv[1]).open() as src:
            for camera in csv.DictReader(src):
                rows.writerow([camera[k] for k in ("camera","lon","lat","radius_m")])
with Path(__file__).with_suffix(".sql").open() as queries:
    subprocess.run(["duckdb","-no-agent","-csv","-bail",str(RAW / "surface.duckdb")],stdin=queries,check=True)
