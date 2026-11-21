"""Compare displayed streets with the candidate road polygons at fixed camera locations."""
import csv
import gzip
import io
import json
import shutil
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from zipfile import ZipFile

from road_surface import RAW, OUTPUT

def read_json(path):
    with (gzip.open(path, "rt") if path.suffix==".gz" else path.open()) as f:
        return json.load(f)


def full_bus_paths(feed_path, buses):
    # Identify source shapes by reproducing transit_paths.py's exact packing rule.
    # Multiple shape IDs may share all their coordinates. Keep only unique full geometries.
    published = defaultdict(list)
    for bus in buses:
        published[tuple(map(tuple, bus["path"]))].append(bus["route"])
    by_shape = defaultdict(list)
    with ZipFile(feed_path) as feed, feed.open("shapes.txt") as src:
        for row in csv.DictReader(io.TextIOWrapper(src, encoding="utf-8-sig")):
            by_shape[row["shape_id"]].append((int(row["shape_pt_sequence"]),
                round(float(row["shape_pt_lon"]), 5), round(float(row["shape_pt_lat"]), 5)))
    matches = defaultdict(set)
    for rows in by_shape.values():
        shape = tuple((lon, lat) for _, lon, lat in sorted(rows))
        key = shape[:-1:3] + shape[-1:]
        if key in published:
            matches[key].add(shape)
    for key, names in published.items():
        if len(matches[key]) == 1:
            full = next(iter(matches[key]))
            for name in names:
                yield "bus", "gtfs_full_matched", name, full
        else:
            print("GTFS full shape unavailable or ambiguous:", names, len(matches[key]), file=sys.stderr)


def movement_paths():
    local = read_json(Path("web/data/local_mover_routes.json.gz"))
    for mode, index in sorted({(g["mode"],g["route"]) for g in local["groups"]}):
        yield mode, "local_osrm", index, local["paths"][index]
    buses = [r for r in read_json(Path("web/data/transit_paths.json")) if r["mode"] == "bus"]
    for route in buses:
        yield "bus", "gtfs", route["route"], route["path"]
    if len(sys.argv) > 2:
        yield from full_bus_paths(sys.argv[2], buses)


# Report inputs are local snapshots of geometry actually read by the two views.
with (RAW / "baseline.csv").open("w") as out:
    csv_out = csv.writer(out)
    csv_out.writerow(("osm_id","width_m","geometry"))
    for path in (Path("data/mvp/roads.geojson"), Path("web/data/residential_streets.geojson.gz")):
        for f in read_json(path)["features"]:
            kind = f["properties"]["kind"]
            width = {"motorway":13,"trunk":10,"primary":10,"secondary":8}.get(kind,5.6)
            props = f["properties"]
            way_id = props["id"] if "id" in props else props["osm_id"]
            csv_out.writerow((f"w{way_id}",width,json.dumps(f["geometry"],separators=(",",":"))))
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
