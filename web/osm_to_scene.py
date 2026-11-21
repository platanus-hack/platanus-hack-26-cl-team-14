"""OSM extract -> web/data/ground.geojson (roads, river, canal, parks). Buildings come from data/mvp/buildings.geojson.
Source: https://api.openstreetmap.org/api/0.6/map?bbox=-70.6105,-33.4255,-70.5915,-33.4105
Run from the repo root: python3 web/osm_to_scene.py
"""
import json
import xml.etree.ElementTree as ET

root = ET.parse("data/raw/osm_buildings/tobalaba_map.osm").getroot()
nodes = {n.get("id"): [round(float(n.get("lon")), 6), round(float(n.get("lat")), 6)] for n in root.iter("node")}
ways = {w.get("id"): w for w in root.iter("way")}
tags = lambda e: {t.get("k"): t.get("v") for t in e.iter("tag")}
coords = lambda w: [nodes[nd.get("ref")] for nd in w.iter("nd") if nd.get("ref") in nodes]

ground = []

for wid, w in ways.items():
    t = tags(w)
    if t.get("highway") in ("motorway", "trunk", "primary", "secondary", "tertiary", "residential", "motorway_link", "trunk_link", "primary_link"):
        ground.append({"type": "Feature", "geometry": {"type": "LineString", "coordinates": coords(w)},
                       "properties": {"kind": "road", "major": t["highway"] in ("motorway", "trunk", "primary")}})
    elif t.get("waterway") in ("river", "canal"):
        ground.append({"type": "Feature", "geometry": {"type": "LineString", "coordinates": coords(w)}, "properties": {"kind": t["waterway"]}})
    elif t.get("leisure") in ("park", "garden", "golf_course") or t.get("landuse") == "grass":
        ground.append({"type": "Feature", "geometry": {"type": "Polygon", "coordinates": [coords(w)]}, "properties": {"kind": "park"}})

json.dump({"type": "FeatureCollection", "features": ground}, open("web/data/ground.geojson", "w"), separators=(",", ":"))
print(len(ground), "ground features")
