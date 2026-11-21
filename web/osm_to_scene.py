"""OSM extract -> web/data/buildings.geojson and web/data/ground.geojson.
Source: https://api.openstreetmap.org/api/0.6/map?bbox=-70.6105,-33.4255,-70.5915,-33.4105
Run from the repo root: python3 web/osm_to_scene.py
Height: tag `height`, else `building:levels` x 3.2 m, else 9 m (assumption, flagged h_src="default").
"""
import json, re
import xml.etree.ElementTree as ET

root = ET.parse("data/raw/osm_buildings/tobalaba_map.osm").getroot()
nodes = {n.get("id"): [round(float(n.get("lon")), 6), round(float(n.get("lat")), 6)] for n in root.iter("node")}
ways = {w.get("id"): w for w in root.iter("way")}
tags = lambda e: {t.get("k"): t.get("v") for t in e.iter("tag")}
coords = lambda w: [nodes[nd.get("ref")] for nd in w.iter("nd") if nd.get("ref") in nodes]

buildings, ground = [], []


def building(t, ring, osm_id):
    if len(ring) < 4:
        return
    if "height" in t:
        h, src = float(re.match(r"[\d.]+", t["height"]).group()), "height"
    elif "building:levels" in t:
        h, src = float(re.match(r"[\d.]+", t["building:levels"]).group()) * 3.2, "levels"
    else:
        h, src = 9, "default"
    buildings.append({"type": "Feature", "geometry": {"type": "Polygon", "coordinates": [ring]},
                      "properties": {"id": osm_id, "h": round(h, 1), "h_src": src, "name": t.get("name", "")}})


for wid, w in ways.items():
    t = tags(w)
    if "building" in t or "building:part" in t:
        building(t, coords(w), "w" + wid)
    elif t.get("highway") in ("motorway", "trunk", "primary", "secondary", "tertiary", "residential", "motorway_link", "trunk_link", "primary_link"):
        ground.append({"type": "Feature", "geometry": {"type": "LineString", "coordinates": coords(w)},
                       "properties": {"kind": "road", "major": t["highway"] in ("motorway", "trunk", "primary")}})
    elif t.get("waterway") in ("river", "canal"):
        ground.append({"type": "Feature", "geometry": {"type": "LineString", "coordinates": coords(w)}, "properties": {"kind": "river"}})
    elif t.get("leisure") in ("park", "garden", "golf_course") or t.get("landuse") == "grass":
        ground.append({"type": "Feature", "geometry": {"type": "Polygon", "coordinates": [coords(w)]}, "properties": {"kind": "park"}})

for r in root.iter("relation"):  # multipolygon buildings: outer rings only
    t = tags(r)
    if "building" in t:
        for m in r.iter("member"):
            if m.get("role") == "outer" and m.get("ref") in ways:
                building(t, coords(ways[m.get("ref")]), "r" + r.get("id"))

json.dump({"type": "FeatureCollection", "features": buildings}, open("web/data/buildings.geojson", "w"), separators=(",", ":"))
json.dump({"type": "FeatureCollection", "features": ground}, open("web/data/ground.geojson", "w"), separators=(",", ":"))
from collections import Counter
print(len(buildings), "buildings", Counter(b["properties"]["h_src"] for b in buildings), len(ground), "ground")
for b in buildings:
    p = b["properties"]
    if re.search(r"MUT|Mercado Urbano|Costanera|Titanium", p["name"]):
        print(p, b["geometry"]["coordinates"][0][0])
