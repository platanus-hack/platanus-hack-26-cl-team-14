"""Place April 15 work arrivals at their final public transport stop."""
import csv
import io
import json
import xml.etree.ElementTree as ET
import zipfile
from collections import Counter, defaultdict
from pathlib import Path

raw = Path("data/raw")
folder = Path("data/validation/trip_purpose_2026_04")
bus_ids = defaultdict(set)
rail_comunas = defaultdict(set)
modes = {}
with zipfile.ZipFile(raw / "dtpm/subidas_2026_04.xlsx") as workbook:
    strings = ["".join(item.itertext()) for item in ET.fromstring(workbook.read("xl/sharedStrings.xml"))]
    with workbook.open("xl/worksheets/sheet3.xml") as sheet:
        for _, row in ET.iterparse(sheet):
            if row.tag.endswith("}row"):
                if row.get("r") != "1":
                    mode, stop, public_id = (strings[int(cell[0].text)] for cell in row[1:4])
                    modes[stop] = mode
                    if mode == "BUS":
                        bus_ids[stop].add(public_id)
                    elif mode == "Tren":
                        rail_comunas[stop].add(strings[int(row[4][0].text)])
                row.clear()

with zipfile.ZipFile(raw / "dtpm-gtfs-march/GTFS_20260321_v3.zip") as feed:
    stops = {s["stop_id"]: (float(s["stop_lon"]), float(s["stop_lat"]))
             for s in csv.DictReader(io.TextIOWrapper(feed.open("stops.txt"), encoding="utf-8-sig")) if s["stop_lon"] and s["stop_lat"]}

metro = {r["station_name"]: r["station_id"] for r in csv.DictReader((folder / "metro_arrivals.csv").open())}
polygons = []
for feature in json.loads(Path("data/mvp/communes.geojson").read_text())["features"]:
    for outer, in feature["geometry"]["coordinates"]:
        xs, ys = zip(*outer)
        polygons.append((min(xs), max(xs), min(ys), max(ys), outer, feature["properties"]["cut"]))


def comuna(x, y):
    for west, east, south, north, ring, cut in polygons:
        if west <= x <= east and south <= y <= north:
            inside = False
            for (a, b), (c, d) in zip(ring, ring[1:]):
                if (b > y) != (d > y) and x < (c-a)*(y-b)/(d-b)+a:
                    inside = not inside
            if inside:
                return cut
    return "UNPLACED"


work = Counter()
for row in csv.DictReader((raw / "dtpm-viajes-2026-04/by_destination.csv").open()):
    if row["proposito"] == "TRABAJO":
        work[row["destination_stop"]] += float(row["expanded_trips"])

comuna_codes = {r["name"]: r["cut"] for r in csv.DictReader(Path("data/mvp/communes.csv").open())}
totals = defaultdict(Counter)
counts = Counter()
for code, trips in work.items():
    mode = modes.get(code, "unknown")
    cut = "UNPLACED"
    if code in metro:
        mode = "Metro"
        cut = comuna(*stops[metro[code]])
    elif len(bus_ids.get(code, ())) == 1:
        stop_id = next(iter(bus_ids[code]))
        if stop_id in stops:
            cut = comuna(*stops[stop_id])
    elif len(rail_comunas.get(code, ())) == 1:
        cut = comuna_codes.get(next(iter(rail_comunas[code])), "UNPLACED")
    totals[cut][mode] += trips
    counts[cut, mode] += 1

with (folder / "work_arrivals_by_comuna.csv").open("w", newline="") as output:
    writer = csv.writer(output)
    writer.writerow(["cut", "destination_mode", "destination_codes", "expanded_work_trips"])
    for cut, mode in sorted(counts):
        writer.writerow([cut, mode, counts[cut, mode], round(totals[cut][mode], 4)])
