"""Attach OSM track classifications to published GTFS station-to-station passenger paths."""
import csv
import gzip
import io
import json
import math
import subprocess
import sys
from collections import Counter
from pathlib import Path
from xml.etree import ElementTree as ET
from zipfile import ZipFile

pbf, gtfs = sys.argv[1:]
paths = [r for r in json.loads(Path("web/data/transit_paths.json").read_text()) if r["mode"] == "metro"]
lines = {r["route"] for r in paths}
# Use one full all-stops relation per line and direction. Express variants reuse the same track.
relation_ids = [444961, 444963, 444964, 444976, 444982, 2193874, 2943868, 3636603,
                6657231, 6657239, 6657245, 6657266, 7732975, 9633217]
xml = subprocess.check_output(["osmium", "getid", "-r", "-f", "osm", pbf, *[f"r{i}" for i in relation_ids]])
osm = {(e.tag, int(e.get("id"))): e for e in ET.fromstring(xml)}

def tags(element):
    return {tag.get("k"): tag.get("v") for tag in element.findall("tag")}

def coord(element):
    return [float(element.get("lon")), float(element.get("lat"))]

def distance(a, b):
    dx = math.radians(b[0] - a[0])
    dy = math.radians(b[1] - a[1])
    h = math.sin(dy / 2)**2 + math.cos(math.radians(a[1])) * math.cos(math.radians(b[1])) * math.sin(dx / 2)**2
    return 12742017.6 * math.asin(math.sqrt(h))

def position(point, path):
    # Fractional vertex index of the closest point along the published path.
    scale = math.cos(math.radians(point[1]))
    best = (float("inf"), 0)
    for i, (a, b) in enumerate(zip(path, path[1:])):
        vx, vy = (b[0]-a[0])*scale, b[1]-a[1]
        px, py = (point[0]-a[0])*scale, point[1]-a[1]
        f = max(0, min(1, (px*vx+py*vy)/(vx*vx+vy*vy))) if vx or vy else 0
        best = min(best, ((px-f*vx)**2+(py-f*vy)**2, i+f))
    return best[1]

def interpolate(path, p):
    i = min(int(p), len(path)-2)
    return [round(a+(b-a)*(p-i), 7) for a,b in zip(path[i],path[i+1])]

def feature(shape, coordinates, **properties):
    return {"type":"Feature", "geometry":{"type":shape, "coordinates":coordinates}, "properties":properties}

def evidence(t):
    if t.get("tunnel") == "yes":
        return "tunnel", "tunnel=yes"
    if t.get("bridge") in {"yes","viaduct"}:
        return "viaduct", "bridge="+t["bridge"]
    if t.get("tunnel") == "no":
        return "surface", "tunnel=no; no bridge tag"
    return "unknown", "no tunnel or bridge tag"

with ZipFile(gtfs) as feed:
    stops = {r["stop_id"]: r for r in csv.DictReader(io.TextIOWrapper(feed.open("stops.txt"), encoding="utf-8-sig"))}
def station_coord(station):
    row = stops[station]
    return [float(row["stop_lon"]),float(row["stop_lat"])]

features = []
routes = []
unique_edges = {}
platform_objects = set()
for rid in relation_ids:
    relation = osm["relation", rid]
    line = tags(relation)["ref"]
    members = [(m.get("type"),int(m.get("ref")),m.get("role")) for m in relation.findall("member")]
    stop_nodes = [i for typ,i,role in members if typ=="node" and role=="stop"]
    ways = [i for typ,i,role in members if typ=="way" and not role]
    first = coord(osm["node",stop_nodes[0]])
    route = min((r for r in paths if r["route"]==line),
                key=lambda r: distance(first, station_coord(r["parent_station_ids"][0])))
    stations, path = route["parent_station_ids"], route["path"]
    source_ways = []
    for w in ways:
        way = osm["way",w]
        t = tags(way)
        if t.get("railway")!="subway" or "service" in t:
            raise ValueError(f"Unexpected route member {w} in relation {rid}")
        source_ways.append(([int(n.get("ref")) for n in way.findall("nd")], evidence(t), t.get("layer")))
    # The downloaded relations connect by endpoint. Trim each route to its passenger stops.
    if source_ways[0][0][0] in (source_ways[1][0][0], source_ways[1][0][-1]):
        source_ways[0][0].reverse()
    nodes = [source_ways[0][0][0]]
    edges = []
    for refs, classification, layer in source_ways:
        if refs[-1]==nodes[-1]:
            refs.reverse()
        if refs[0]!=nodes[-1]:
            raise ValueError(f"Disconnected route in relation {rid}")
        for a,b in zip(refs, refs[1:]):
            edges.append((distance(coord(osm["node",a]),coord(osm["node",b])),classification,layer))
        nodes.extend(refs[1:])
    indices = [nodes.index(n) for n in stop_nodes]
    aligned = len(indices)==len(stations) and indices==sorted(indices)
    # Match stop sequences only when every boarding location is near its GTFS parent.
    if aligned:
        aligned = all(distance(coord(osm["node",n]), station_coord(s))<350 for n,s in zip(stop_nodes,stations))
    offsets = [0]+[position(station_coord(s), path) for s in stations[1:-1]]+[len(path)-1]
    traversal = []
    for j,(a,b) in enumerate(zip(offsets,offsets[1:])):
        if b<=a:
            raise ValueError(f"Out-of-order GTFS stops in {line}")
        coords = [interpolate(path,a), *path[int(a)+1:math.ceil(b)], interpolate(path,b)]
        kinds, reasons, layers = Counter(), set(), set()
        kind_sequence = []
        if aligned:
            for edge in range(indices[j], indices[j+1]):
                length,(kind,reason),layer = edges[edge]
                kinds[kind]+=length
                if not kind_sequence or kind_sequence[-1] != kind:
                    kind_sequence.append(kind)
                reasons.add(reason)
                if layer is not None:
                    layers.add(layer)
                unique_edges[tuple(sorted(nodes[edge:edge+2]))] = (length, kind, reason)
        kind = next(iter(kinds)) if len(kinds)==1 else "mixed" if kinds else "unknown"
        segment_id = f"{line}:{route['direction']}:{j}"
        features.append(feature("LineString",coords,feature="track",line=line,kind=kind,
            segment_id=segment_id,from_parent_station_id=stations[j],to_parent_station_id=stations[j+1],
            direction=route["direction"],sequence=j,source="gtfs_shape",osm_relation_id=rid,
            origin_kind=edges[indices[j]][1][0] if aligned else "unknown",
            destination_kind=edges[indices[j+1]-1][1][0] if aligned else "unknown",
            kind_sequence=kind_sequence,osm_match=aligned,osm_kind_metres={k:round(v,1) for k,v in sorted(kinds.items())},
            tags=sorted(reasons),tagged_layers=sorted(layers)))
        traversal.append(segment_id)
    routes.append(dict(line=line,direction=route["direction"],parent_station_ids=stations,segment_ids=traversal))
    for typ,ref,role in members:
        if role!="platform" or (line,typ,ref) in platform_objects:
            continue
        platform_objects.add((line,typ,ref))
        geom = osm[typ,ref]
        points = [coord(osm["node",int(n.get("ref"))]) for n in geom.findall("nd")]
        points = points[:-1] if points[0]==points[-1] else points
        centre = [round(sum(c)/len(c),7) for c in zip(*points)]
        near = min(stations, key=lambda s: distance(centre,station_coord(s)))
        parent = near if distance(centre,station_coord(near))<350 else None
        features.append(feature("Point",centre,feature="platform",line=line,parent_station_id=parent,
                                source="osm_platform_vertex_mean",osm_type=typ,osm_id=ref))
for stop_id,row in stops.items():
    if row["location_type"]!="0" or not row["parent_station"]:
        continue
    parts = stop_id.split("_")
    if len(parts)==3 and parts[1] in lines and parts[2].startswith("V"):
        features.append(feature("Point",[round(v,7) for v in station_coord(stop_id)],feature="platform",
            line=parts[1],parent_station_id=row["parent_station"],source="gtfs_stop",stop_id=stop_id))

# Published station observations are independent of the adjoining track classification.
station_settings = {
    ("L1", "NP"): ("partly_open_below_street", "https://www.tesisenred.net/bitstream/handle/10803/80299/TRHC1de17.pdf?isAllowed=y&sequence=1"),
    ("L5", "BL"): ("underground", "https://www.ohla-group.com/wp-content/uploads/2021/07/Railway-Infrastructures-monografico.pdf"),
}
for item in features:
    props = item["properties"]
    if props["feature"] == "platform":
        setting = station_settings.get((props["line"], props["parent_station_id"]))
        if setting:
            props["station_setting"], props["station_setting_source"] = setting

osm_kinds, osm_sources = Counter(), Counter()
for metres, kind, reason in unique_edges.values():
    osm_kinds[kind] += metres
    osm_sources[reason] += metres
pack = {"type":"FeatureCollection","attribution":"© OpenStreetMap contributors; DTPM March 2026 GTFS",
        "routes":sorted(routes,key=lambda r:(r["line"],r["direction"])),
        "unique_osm_track_geometry_m":round(sum(osm_kinds.values()),1),
        "unique_osm_kind_metres":{k:round(v,1) for k,v in sorted(osm_kinds.items())},
        "unique_osm_evidence_metres":{k:round(v,1) for k,v in sorted(osm_sources.items())},
        "features":features}
data = (json.dumps(pack,ensure_ascii=False,separators=(",",":"))+"\n").encode()
out = Path("web/data/metro_tracks.geojson.gz")
with out.open("wb") as f, gzip.GzipFile(filename="", fileobj=f, mode="wb", mtime=0, compresslevel=6) as gz:
    gz.write(data)
print(len(data),out.stat().st_size,pack["unique_osm_track_geometry_m"])
