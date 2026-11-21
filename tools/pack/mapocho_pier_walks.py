"""Build ten representative walks from census blocks to the proposed Mapocho piers.

Run from the repository root with osmium, Shapely and the same urban.osm.pbf as
the local mover pack (data/mvp/README.md, "Local mover routes"). The engine
publishes pier totals without block-level OD, so each pier gets one nearby
populated block, for illustration. We start at its nearest connected street
node; the gap to the block centroid is recorded in metres. OSM segments use
shared source node IDs; none enter the mapped water. The final straight link
from the nearest accessible street to the hypothetical stop is proposed.

    python tools/pack/mapocho_pier_walks.py path/to/urban.osm.pbf
"""
import csv
import hashlib
import heapq
import json
import math
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path
from xml.etree import ElementTree

from shapely.geometry import LineString, shape
from shapely.ops import unary_union

ROOT = Path(__file__).resolve().parents[2]
WALKABLE = {"footway", "path", "steps", "pedestrian", "living_street", "residential",
            "service", "unclassified", "tertiary", "secondary", "primary"}
DENIED = {"private", "no", "customers", "delivery", "agricultural", "forestry"}


def metres(a, b):
    return math.hypot((a[0] - b[0]) * 92915, (a[1] - b[1]) * 110905)


def land_network(pier, pbf, temp, water):
    lon, lat = pier
    bbox = f"{lon-.0065},{lat-.0055},{lon+.0065},{lat+.0055}"
    cut = temp / "near.pbf"
    subprocess.run(["osmium", "extract", "-s", "simple", "-b", bbox, "-O", "-o", cut, pbf], check=True)
    xml = subprocess.check_output(["osmium", "tags-filter", cut,
                                   "w/highway=" + ",".join(sorted(WALKABLE)), "-f", "osm"])
    data = ElementTree.fromstring(xml)
    nodes = {n.get("id"): (float(n.get("lon")), float(n.get("lat"))) for n in data.iter("node")}
    graph = defaultdict(dict)
    for way in data.iter("way"):
        tags = {t.get("k"): t.get("v") for t in way.iter("tag")}
        if (tags.get("foot", tags.get("access")) in DENIED or tags.get("area") == "yes"
                or tags.get("tunnel", "no") != "no" or tags.get("bridge", "no") != "no"
                or tags.get("motorroad") == "yes"):
            continue
        ids = [n.get("ref") for n in way.iter("nd")]
        for a, b in zip(ids, ids[1:]):
            if a not in nodes or b not in nodes or a == b:
                continue
            if water.intersects(LineString([nodes[a], nodes[b]])):
                continue
            length = metres(nodes[a], nodes[b])
            if length:
                graph[a][b] = graph[b][a] = length
    return nodes, graph


def route(pier, blocks, nodes, graph):
    # Place the landside end of the gangway at the closest position on an
    # accessible mapped street, path or sidewalk in a populated component.
    edges = []
    lon, lat = pier
    for a, adjacent in graph.items():
        for b in adjacent:
            if a >= b:
                continue
            p, q = nodes[a], nodes[b]
            dx, dy = (q[0]-p[0])*92915, (q[1]-p[1])*110905
            t = ((lon-p[0])*92915*dx + (lat-p[1])*110905*dy) / (dx*dx+dy*dy)
            t = max(0, min(1, t))
            shore = (p[0]+(q[0]-p[0])*t, p[1]+(q[1]-p[1])*t)
            edges.append((metres(pier, shore), a, b, shore))
    for _, a, b, shore in sorted(edges):
        distances = {n: metres(nodes[n], shore) for n in (a, b)}
        parent = {a: None, b: None}
        queue = [(v, n) for n, v in distances.items()]
        heapq.heapify(queue)
        while queue:
            dist, n = heapq.heappop(queue)
            if dist != distances[n]:
                continue
            for m, length in graph[n].items():
                travel = dist + length
                if travel <= 750 and travel < distances.get(m, math.inf):
                    distances[m] = travel
                    parent[m] = n
                    heapq.heappush(queue, (travel, m))
        candidates = []
        for block, xy in blocks:
            if metres(xy, pier) > 700:
                continue
            start = min(distances, key=lambda n: metres(nodes[n], xy))
            snap = metres(nodes[start], xy)
            if 70 <= distances[start] <= 650 and snap <= 80:
                candidates.append((snap + abs(distances[start]-165)*.3, block, start, snap))
        if candidates:
            _, block, start, snap = min(candidates)
            path = []
            n = start
            while n is not None:
                path.append(nodes[n])
                n = parent[n]
            if metres(path[-1], shore) < .1:
                path[-1] = shore
            else:
                path.append(shore)
            return {"block": block, "block_snap_m": round(snap, 1),
                    "coordinates": [[round(x, 7), round(y, 7)] for x, y in [*path, pier]]}
    raise ValueError(f"No connected walk from a populated block near {pier}")


def main(pbf):
    piers = json.loads((ROOT / "context/research/mapocho/piers.json").read_text())
    river = json.loads((ROOT / "web/data/mapocho_detail.geojson").read_text())
    water = unary_union([shape(f["geometry"]) for f in river["features"] if f["properties"]["kind"] == "water"])
    with (ROOT / "data/mvp/blocks.csv").open() as source:
        blocks = [(r["block_id"], (float(r["longitude"]), float(r["latitude"])))
                  for r in csv.DictReader(source) if int(r["population"]) > 0]
    walks = {}
    with tempfile.TemporaryDirectory() as temp:
        for row in piers:
            stop = row["target"]
            pier = stop["longitude"], stop["latitude"]
            nodes, graph = land_network(pier, pbf, Path(temp), water)
            walks[stop["id"]] = route(pier, blocks, nodes, graph)
    with pbf.open("rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    pack = {"source_pbf_sha256": digest,
            "attribution": "© OpenStreetMap contributors (ODbL 1.0); INE Census 2024",
            "routes": walks}
    (ROOT / "web/data/mapocho_pier_walks.json").write_text(json.dumps(pack, separators=(",", ":")) + "\n")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
