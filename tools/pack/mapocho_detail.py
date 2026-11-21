"""Export the Mapocho water surface and its road bridges for the detailed river view.

Source: OpenStreetMap through the Overpass API (ODbL). Usage: python3 -I tools/pack/mapocho_detail.py
"""
import json
import urllib.parse
import urllib.request
from pathlib import Path

OUT = Path("web/data/mapocho_detail.geojson")
API = "https://overpass-api.de/api/interpreter"
# The two water=river multipolygons of the Mapocho channel inside the urban view (west and east of Tobalaba).
RIVER_AREAS = (15533903, 12187494)
QUERY = f"""[out:json][timeout:180];
rel(id:{','.join(map(str, RIVER_AREAS))})->.r;
.r out body;
way(r.r);
out geom;
way(r.r)->.edges;
way["bridge"]["highway"](around.edges:30);
out tags geom;
"""


def fetch():
    body = urllib.parse.urlencode({"data": QUERY}).encode()
    request = urllib.request.Request(API, body, {"User-Agent": "team14-sim/1.0"})
    return json.load(urllib.request.urlopen(request, timeout=300))["elements"]


def point(p):
    return [round(p["lon"], 6), round(p["lat"], 6)]


def rings(ways):
    # Join open member ways end to end into closed rings.
    open_parts = [[point(p) for p in w] for w in ways]
    done = []
    while open_parts:
        ring = open_parts.pop()
        while ring[0] != ring[-1]:
            for i, part in enumerate(open_parts):
                if part[0] == ring[-1]:
                    ring += part[1:]
                elif part[-1] == ring[-1]:
                    ring += part[::-1][1:]
                else:
                    continue
                open_parts.pop(i)
                break
            else:
                raise ValueError("unclosed ring")
        done.append(ring)
    return done


elements = fetch()
geometry = {e["id"]: e["geometry"] for e in elements if e["type"] == "way" and "geometry" in e}
features = []
for rel in (e for e in elements if e["type"] == "relation"):
    by_role = {role: [geometry[m["ref"]] for m in rel["members"] if m["type"] == "way" and m["role"] == role]
               for role in ("outer", "inner")}
    polygons = [[outer] for outer in rings(by_role["outer"])]
    holes = rings(by_role["inner"]) if by_role["inner"] else []
    # Shortcut: every hole goes to the first outer ring. Correct for the two Mapocho relations, which have one outer ring each.
    polygons[0] += holes
    features.append({"type": "Feature",
                     "geometry": {"type": "MultiPolygon", "coordinates": polygons},
                     "properties": {"kind": "water", "name": "Río Mapocho", "osm_id": f"r{rel['id']}"}})
for way in (e for e in elements if e["type"] == "way" and "bridge" in e.get("tags", {})):
    tags = way["tags"]
    features.append({"type": "Feature",
                     "geometry": {"type": "LineString", "coordinates": [point(p) for p in way["geometry"]]},
                     "properties": {"kind": "bridge", "name": tags.get("name"), "highway": tags["highway"],
                                    "layer": int(tags.get("layer", 1)), "osm_id": f"w{way['id']}"}})
OUT.write_text(json.dumps({"type": "FeatureCollection", "features": features}, ensure_ascii=False,
                          separators=(",", ":")) + "\n")
print(f"{OUT}: {sum(f['properties']['kind'] == 'water' for f in features)} water areas, "
      f"{sum(f['properties']['kind'] == 'bridge' for f in features)} bridges, {OUT.stat().st_size} bytes")
