"""Locate simulation inputs in the EOD 2012 Santiago survey zoning."""
import csv
import sys
from collections import Counter, defaultdict
from pathlib import Path

import shapefile
from pyproj import CRS, Transformer
from shapely import make_valid
from shapely.geometry import Point, Polygon, shape
from shapely.strtree import STRtree

MVP = Path("data/mvp")
KEYS = {
    "higher_ed": ["institution", "campus", "cut", "year"],
    "activity_points": ["osm_id"],
    "blocks": ["block_id"],
    "schools": ["rbd"],
}


def survey_index(survey):
    stem = survey / "Zonificacion_EOD2012/Zonificacion_EOD2012"
    project = Transformer.from_crs("EPSG:4326", CRS.from_wkt(stem.with_suffix(".prj").read_text()),
                                   always_xy=True)
    with shapefile.Reader(shp=str(stem.with_suffix(".shp")), dbf=str(stem.with_suffix(".DBF")),
                          shx=str(stem.with_suffix(".shx")), encoding="latin-1") as reader:
        zones, cuts, polygons = [], {}, []
        for record in reader.iterShapeRecords():
            zone = int(record.record.Zona)
            zones.append(zone)
            cuts[zone] = record.record.Com
            raw = record.shape
            polygon = Polygon(raw.points) if len(raw.parts) == 1 else shape(raw.__geo_interface__)
            if not polygon.is_valid:
                polygon = make_valid(polygon, method="structure", keep_collapsed=False)
            polygons.append(polygon)

    home, origin, destination = (defaultdict(set) for _ in range(3))
    for filename, fields in [
        ("Hogares.csv", [("Zona", "Sector", home)]),
        ("viajes.csv", [("ZonaOrigen", "SectorOrigen", origin),
                        ("ZonaDestino", "SectorDestino", destination)]),
    ]:
        with (survey / filename).open(encoding="latin-1") as f:
            for row in csv.DictReader(f, delimiter=";"):
                for zone_field, sector_field, evidence in fields:
                    value = row[sector_field]
                    if value in {"1", "2", "3", "4", "5", "6", "7"} and row[zone_field]:
                        evidence[int(row[zone_field])].add(value)

    comuna_sectors = defaultdict(set)
    for zone, cut in cuts.items():
        comuna_sectors[cut].update(home[zone])

    sectors = {}
    with (MVP / "eod_sectors/zones.csv").open("w") as out:
        writer = csv.writer(out, lineterminator="\n")
        writer.writerow(["zone", "polygon_cut", "sector", "method", "home_sectors",
                         "origin_sectors", "destination_sectors"])
        for zone in sorted(zones):
            cut = cuts[zone]
            direct = home[zone] | origin[zone] | destination[zone]
            support = direct | comuna_sectors[cut]
            if len(support) == 1:
                sectors[zone] = next(iter(support))
                method = "direct" if direct else "comuna_households"
            else:
                method = "unresolved"
            writer.writerow([zone, cut, sectors.get(zone, ""), method,
                             *("|".join(sorted(source[zone])) for source in (home, origin, destination))])
    return project, STRtree(polygons), zones, sectors


def locate(row, project, tree, zones, sectors):
    if not row["longitude"] or not row["latitude"]:
        return "", "", "missing_coordinates"
    point = Point(project.transform(float(row["longitude"]), float(row["latitude"])))
    zone_ids = sorted(zones[i] for i in tree.query(point, predicate="intersects"))
    if not zone_ids:
        return "", "", "outside_zones"
    codes = {sectors.get(z) for z in zone_ids}
    zone_text = "|".join(map(str, zone_ids))
    if len(codes) != 1 or None in codes:
        return zone_text, "", "unresolved_sector"
    return zone_text, codes.pop(), "placed" if len(zone_ids) == 1 else "shared_sector"


def main(survey):
    index = survey_index(survey)
    for name, keys in KEYS.items():
        counts = Counter()
        with (MVP / f"{name}.csv").open() as source, (MVP / f"eod_sectors/{name}.csv").open("w") as output:
            writer = csv.writer(output, lineterminator="\n")
            writer.writerow([*keys, "zone", "sector", "status"])
            for row in csv.DictReader(source):
                zone, sector, status = locate(row, *index)
                writer.writerow([*(row[k] for k in keys), zone, sector, status])
                counts[status] += 1
        print(name, dict(counts))


if __name__ == "__main__":
    main(Path(sys.argv[1]))
