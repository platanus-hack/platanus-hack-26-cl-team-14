"""Build the outdoor part of a stadium arrival from connected OSM ways."""

import heapq
import itertools
import json
import math
import subprocess
import tempfile
from collections import defaultdict
from pathlib import Path
from xml.etree import ElementTree

from shapely import STRtree, union_all
from shapely.geometry import LineString, Point, Polygon, shape

GATE = 12471507127
FOOT = {"footway", "pedestrian", "steps", "path"}
STREETS = {
    "living_street",
    "service",
    "residential",
    "unclassified",
    "tertiary",
    "secondary",
    "primary",
}
WINDOWS = {
    "monumental": "-70.620,-33.521,-70.592,-33.494",
    "mut": "-70.614,-33.429,-70.588,-33.406",
}


def metric(p):
    return ((p[0] + 70.65) * 92915, (p[1] + 33.45) * 110905)


def degrees(p):
    return [round(p[0] / 92915 - 70.65, 7), round(p[1] / 110905 - 33.45, 7)]


def tags(element):
    return {t.get("k"): t.get("v") for t in element.findall("tag")}


class GateNetwork:
    def __init__(self, root, gate, excluded, closed):
        self.nodes = {}
        self.platforms = defaultdict(list)
        forbidden = set()
        for node in root.findall("node"):
            nid = int(node.get("id"))
            self.nodes[nid] = metric((float(node.get("lon")), float(node.get("lat"))))
            t = tags(node)
            if t.get("foot", t.get("access")) in {
                "no",
                "private",
                "customers",
            } or t.get("barrier") in {"wall", "fence", "retaining_wall"}:
                forbidden.add(nid)
            if t.get("ref") and (
                t.get("highway") == "bus_stop"
                or t.get("public_transport") == "platform"
            ):
                self.platforms[t["ref"]].append(nid)
        self.gate = Point(metric(gate))
        self.excluded = excluded
        self.lines, self.edges = [], []
        adjacency = defaultdict(list)
        for way in root.findall("way"):
            wid, t = int(way.get("id")), tags(way)
            hw = t.get("highway")
            if (
                wid in closed
                or hw not in FOOT | STREETS
                and not (hw == "cycleway" and t.get("foot") in {"yes", "designated"})
            ):
                continue
            if (
                t.get("foot", t.get("access")) in {"no", "private", "customers"}
                or t.get("area") == "yes"
                or t.get("indoor") == "yes"
                or t.get("tunnel", "no") != "no"
            ):
                continue
            if hw not in FOOT and (
                t.get("bridge", "no") != "no"
                or any(
                    t.get(k) == "separate"
                    for k in ("sidewalk", "sidewalk:left", "sidewalk:right")
                )
                or t.get("sidewalk") in {"no", "none"}
            ):
                continue
            ids = [int(n.get("ref")) for n in way.findall("nd")]
            for a, b in itertools.pairwise(ids):
                if (
                    a == b
                    or a in forbidden
                    or b in forbidden
                    or a not in self.nodes
                    or b not in self.nodes
                ):
                    continue
                segment = LineString([self.nodes[a], self.nodes[b]])
                if segment.length == 0 or excluded.intersects(segment):
                    continue
                i = len(self.edges)
                self.lines.append(segment)
                self.edges.append((a, b, wid, hw))
                adjacency[a].append((b, i))
                adjacency[b].append((a, i))
        self.tree = STRtree(self.lines)
        self.last = min(
            (
                i
                for i in self.tree.query(self.gate.buffer(0.9), predicate="intersects")
                if self.lines[i].distance(self.gate) < 0.01
                or self.edges[i][3] == "footway"
            ),
            key=lambda i: (self.lines[i].distance(self.gate), i),
        )
        end = self.lines[self.last]
        self.terminal = end.interpolate(end.project(self.gate))
        self.dist, self.via = {}, {}
        queue = []
        for n in self.edges[self.last][:2]:
            self.dist[n] = self.terminal.distance(Point(self.nodes[n]))
            self.via[n] = (None, self.last)
            heapq.heappush(queue, (self.dist[n], n))
        while queue:
            dist, n = heapq.heappop(queue)
            if dist != self.dist[n]:
                continue
            for other, i in adjacency[n]:
                cost = dist + self.lines[i].length
                if cost < self.dist.get(other, math.inf):
                    self.dist[other] = cost
                    self.via[other] = (n, i)
                    heapq.heappush(queue, (cost, other))

    def walk(self, nid):
        if nid not in self.nodes:
            return None
        point = Point(self.nodes[nid])
        candidates = []
        # The OSM platform/door and the centerline must coincide within 1 m.
        # For an off-center point accept only pedestrian ways, never a motor lane.
        for i in self.tree.query(point.buffer(1), predicate="intersects"):
            a, b, _, hw = self.edges[i]
            line = self.lines[i]
            gap = line.distance(point)
            if gap > 0.01 and hw not in FOOT:
                continue
            q = line.interpolate(line.project(point))
            if gap and self.excluded.intersects(LineString([point, q])):
                continue
            if i == self.last:
                candidates.append((gap, q.distance(self.terminal), i, -1, q))
            for n in (a, b):
                if n in self.dist:
                    candidates.append(
                        (gap, self.dist[n] + q.distance(Point(self.nodes[n])), i, n, q)
                    )
        if not candidates:
            return None
        gap, _, i, n, q = min(candidates, key=lambda c: c[:4])
        points, ways = [point.coords[0], q.coords[0]], [self.edges[i][2]]
        while n is not None and n != -1:
            points.append(self.nodes[n])
            n, j = self.via[n]
            ways.append(self.edges[j][2])
        points.extend([self.terminal.coords[0], self.gate.coords[0]])
        coords = [value for value, _ in itertools.groupby(degrees(p) for p in points)]
        geom = LineString([metric(p) for p in coords])
        if self.excluded.intersects(geom):
            return None
        return {
            "osm_start_node": nid,
            "start_centerline_offset_m": round(gap, 2),
            "gate_centerline_offset_m": round(self.terminal.distance(self.gate), 2),
            "osm_ways": [wid for wid, _ in itertools.groupby(ways)],
            "geometry": {"type": "LineString", "coordinates": coords},
            "distance_m": round(geom.length, 1),
            "seconds": math.ceil(geom.length / (4.8 / 3.6)),
        }


def gate_networks(pbf, result, centers):
    move = next(
        e
        for e in result["city_edits"]
        if e["op"] == "move" and e["id"] == "mesh:152:317"
    )
    shell = next(
        shape(f["geometry"])
        for f in json.loads(
            Path("web/data/estadio_monumental_parts.geojson").read_bytes()
        )["features"]
        if f["properties"]["kind"] == "stadium_shell"
    )
    closed = {
        int(e["id"])
        for e in result["city_edits"]
        if e["kind"] == "road" and e["op"] == "remove"
    }
    networks = {}
    with tempfile.TemporaryDirectory() as tmp:
        for venue, state in [("monumental", "before"), ("mut", "after")]:
            out = Path(tmp) / f"{venue}.osm"
            subprocess.run(
                ["osmium", "extract", "-b", WINDOWS[venue], str(pbf), "-o", str(out)],
                check=True,
            )
            root = ElementTree.parse(out).getroot()
            if venue == "monumental":
                source = root.find(f"node[@id='{GATE}']")
                original = [float(source.get("lon")), float(source.get("lat"))]
            dx, dy = [
                x - y for x, y in zip(centers[venue][0], centers["monumental"][0])
            ]
            gate = [round(original[0] + dx, 7), round(original[1] + dy, 7)]
            bowl = Polygon(
                [metric(p) for p in move[state]["geometry"]["coordinates"][0]]
            )
            drawn = Polygon(
                [metric((x + dx, y + dy)) for x, y in shell.exterior.coords]
            )
            veto = union_all([bowl, drawn])
            networks[venue] = GateNetwork(
                root, gate, veto, closed if venue == "mut" else set()
            )
    return networks


def indoor_routes(pathways):
    graph = defaultdict(list)
    for r in pathways:
        if not r["traversal_time"]:
            continue
        a, b = r["from_stop_id"], r["to_stop_id"]
        edge = (int(r["traversal_time"]), r["pathway_id"])
        graph[a].append((b, *edge))
        if r["is_bidirectional"] == "1":
            graph[b].append((a, *edge))
    return graph


def egress(graph, areas, exit_id):
    # Boarding-area children identify the scheduled platform and level.
    queue = [(0, a, a, ()) for a in areas]
    heapq.heapify(queue)
    seen = set()
    while queue:
        seconds, node, area, ids = heapq.heappop(queue)
        if node == exit_id:
            return {
                "boarding_area": area,
                "exit_id": exit_id,
                "pathway_ids": ids,
                "seconds": seconds,
            }
        if node in seen:
            continue
        seen.add(node)
        for other, time, pid in graph[node]:
            if other not in seen:
                heapq.heappush(queue, (seconds + time, other, area, (*ids, pid)))
