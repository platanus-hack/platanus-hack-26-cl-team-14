"""Count mapped use evidence on the native city's rendered OSM footprints."""
import argparse
import csv
import json
from collections import Counter
from pathlib import Path

from pyproj import Transformer
from shapely import STRtree, make_valid
from shapely.geometry import Point, shape
from shapely.ops import transform


CLASSES = "housing office commerce education health industry public other".split()
EVIDENCE = ["footprint_function", "building_type", "interior_point"]
BUILDING = {
    "housing": "apartments apartment residential house detached semidetached_house terrace dormitory bungalow cabin farm static_caravan",
    "office": "office offices",
    "commerce": "retail commercial supermarket kiosk market hotel hostel guest_house",
    "education": "school kindergarten university college school_building",
    "health": "hospital clinic healthcare doctors",
    "industry": "industrial factory warehouse manufacturing logistics",
    "public": "public civic government townhall fire_station police library courthouse community_centre",
    "other": "church cathedral chapel mosque synagogue temple religious monastery shrine stadium sports_centre sports_hall grandstand sports_pavilion train_station train_station_building transportation station bus_station parking garage garages service farm_auxiliary barn stable cowshed greenhouse hangar water_tower toilets",
}
AMENITY = {
    "education": "school kindergarten university college childcare music_school language_school driving_school training",
    "health": "hospital clinic doctors dentist pharmacy veterinary nursing_home social_facility",
    "commerce": "restaurant cafe fast_food food_court bar pub biergarten bank bureau_de_change marketplace fuel car_wash car_rental cinema casino gambling ice_cream",
    "public": "townhall courthouse police fire_station post_office library community_centre prison public_building ranger_station",
    "other": "place_of_worship bus_station ferry_terminal shelter monastery theatre arts_centre events_venue conference_centre planetarium nightclub crematorium funeral_hall",
}
# Require an indoor-capable venue when the evidence comes from a leisure or tourism tag.
LEISURE = set("fitness_centre sports_centre sports_hall stadium bowling_alley dance ice_rink amusement_arcade escape_game adult_gaming_centre trampoline_park".split())
TOURISM = set("hotel hostel motel guest_house apartment chalet alpine_hut wilderness_hut museum gallery aquarium".split())
ABSENT = {"", "no", "none", "unknown", "vacant", "empty", "disused", "abandoned", "construction", "closed"}
CAMERAS = {
    "Tobalaba": (-70.600974, -33.417357),
    "Puente Alto": (-70.57536, -33.60958),
    "Quilicura": (-70.72939, -33.36569),
}


def lookup(groups):
    return {value: category for category, values in groups.items() for value in values.split()}


BUILDING_CLASS = lookup(BUILDING)
AMENITY_CLASS = lookup(AMENITY)


def values(tags, key):
    return {v.strip().lower() for v in tags.get(key, "").split(";")} - ABSENT


def single(classes):
    if len(classes) == 1:
        return next(iter(classes))
    return "other" if classes else None


def function(tags, footprint=False):
    classes = set()
    if footprint:
        classes.update(BUILDING_CLASS[v] for v in values(tags, "building:use") if v in BUILDING_CLASS)
    classes.update(AMENITY_CLASS[v] for v in values(tags, "amenity") if v in AMENITY_CLASS)
    if values(tags, "healthcare"):
        classes.add("health")
    if values(tags, "shop"):
        classes.add("commerce")
    for office in values(tags, "office"):
        classes.add("public" if office in {"government", "diplomatic"} else "office")
    if values(tags, "leisure") & LEISURE:
        classes.add("other")
    for tourism in values(tags, "tourism") & TOURISM:
        classes.add("other" if tourism in {"museum", "gallery", "aquarium"} else "commerce")
    return single(classes)


def features(path):
    with path.open() as source:
        for line in source:
            yield json.loads(line.lstrip("\x1e"))


def building_classes(source):
    classes = {}
    for feature in features(source):
        tags = feature["properties"]
        category = function(tags, footprint=True)
        if category:
            classes[feature["id"]] = (category, "footprint_function")
        else:
            category = single({BUILDING_CLASS[v] for v in values(tags, "building") if v in BUILDING_CLASS})
            if category:
                classes[feature["id"]] = (category, "building_type")
    return classes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source_polygons", type=Path, help="OSM polygons.geojsonseq with original tags and area IDs")
    parser.add_argument("fill_dir", type=Path, help="directory with real_rendered.geojsonseq and real_blocks.csv")
    parser.add_argument("source_points", type=Path, help="tagged OSM nodes from the same urban PBF, in GeoJSONSeq")
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    with (args.fill_dir / "real_blocks.csv").open() as source:
        cuts = {(r["osm_id"], int(r["part"])): r["cut"] or "unassigned" for r in csv.DictReader(source)}
    classes = building_classes(args.source_polygons)
    point_classes, points = [], []
    for feature in features(args.source_points):
        category = function(feature["properties"])
        if category:
            points.append(shape(feature["geometry"]))
            point_classes.append(category)
    point_tree = STRtree(points)

    to_utm = Transformer.from_crs(4326, 32719, always_xy=True).transform
    to_wgs = Transformer.from_crs(32719, 4326, always_xy=True).transform
    circles = {name: transform(to_wgs, Point(*to_utm(*xy)).buffer(500)) for name, xy in CAMERAS.items()}
    with Path("data/mvp/communes.csv").open() as source:
        rows = {("comuna", r["cut"]): Counter() for r in csv.DictReader(source)}
    rows["all", "view"] = Counter()
    rows["comuna", "unassigned"] = Counter()
    rows.update({("circle", name): Counter() for name in circles})
    for feature in features(args.fill_dir / "real_rendered.geojsonseq"):
        props = feature["properties"]
        osm_id = props["osm_id"]
        polygon = shape(feature["geometry"])
        if not polygon.is_valid:
            polygon = make_valid(polygon)
        category, evidence = classes.get(osm_id, (None, None))
        if not category:
            candidates = point_tree.query(polygon, predicate="contains")
            category = single({point_classes[i] for i in candidates})
            evidence = "interior_point"
        cut = cuts[osm_id, props["part"]]
        groups = [("all", "view"), ("comuna", cut)]
        groups.extend(("circle", name) for name, circle in circles.items() if circle.intersects(polygon))
        for group in groups:
            counts = rows[group]
            counts["buildings"] += 1
            if category:
                counts["specific"] += 1
                counts[category] += 1
                counts[evidence] += 1

    with args.output.open("w", newline="") as out:
        writer = csv.writer(out)
        writer.writerow(["scope", "area", "buildings", "specific", "specific_share", *CLASSES, *EVIDENCE])
        for (scope, area), counts in sorted(rows.items()):
            total, specific = counts["buildings"], counts["specific"]
            share = f"{specific / total:.8f}" if total else ""
            writer.writerow([scope, area, total, specific, share, *(counts[k] for k in CLASSES + EVIDENCE)])


if __name__ == "__main__":
    main()
