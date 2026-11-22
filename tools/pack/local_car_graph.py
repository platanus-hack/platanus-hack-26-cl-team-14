"""Pack directed public car streets for local display detours around placed buildings."""
import gzip
import json
import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

from editable_roads import car_allowed, clipped_parts, source_record, travel_direction
from roads import VIEW, distance, read_roads

OUTPUT = Path("web/data/local_car_graph.json.gz")
HIGHWAYS = ("motorway", "motorway_link", "trunk", "trunk_link", "primary", "primary_link",
            "secondary", "secondary_link", "tertiary", "tertiary_link", "residential",
            "unclassified", "living_street", "service")


def build(pbf):
    with TemporaryDirectory() as tmp:
        city = Path(tmp) / "city.osm.pbf"
        raw = Path(tmp) / "streets.osm"
        subprocess.run(["osmium", "extract", "--bbox", ",".join(map(str, VIEW)),
                        "--strategy", "complete_ways", "-o", city, pbf], check=True)
        subprocess.run(["osmium", "tags-filter", city, "w/highway=" + ",".join(HIGHWAYS),
                        "-o", raw], check=True)
        points, source = read_roads(raw)

    nodes, ways = {}, {}
    for number, tags, members in source:
        oneway, _ = travel_direction(tags)
        directions = []
        if oneway in {"no", "yes"} and car_allowed(tags, "forward"):
            directions.append(False)
        if oneway in {"no", "-1"} and car_allowed(tags, "backward"):
            directions.append(True)
        if not directions:
            continue
        edges = []
        for part in clipped_parts(number, members, points):
            for _, left, right, a, b in part:
                nodes[left], nodes[right] = a, b
                length = round(distance(a, b), 3)
                for reverse in directions:
                    start, end = (right, left) if reverse else (left, right)
                    edges.append({"from": start, "to": end, "length_m": length})
        if edges:
            tunnel = tags.get("tunnel", "no") != "no"
            bridge = tags.get("bridge", "no") != "no"
            ways[str(number)] = {"structure": "tunnel" if tunnel else "bridge" if bridge else "ground",
                                 "edges": edges}

    pack = {"attribution": "© OpenStreetMap contributors", "license": "ODbL-1.0",
            "source": source_record(pbf), "bounds": VIEW, "nodes": nodes, "ways": ways}
    with OUTPUT.open("wb") as dest, gzip.GzipFile(filename="", fileobj=dest, mode="wb", mtime=0) as compressed:
        compressed.write((json.dumps(pack, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode())
    print(f"{len(ways)} ways, {len(nodes)} nodes, {sum(len(w['edges']) for w in ways.values())} directed edges, "
          f"{OUTPUT.stat().st_size} gzip bytes")


if __name__ == "__main__":
    build(Path(sys.argv[1]))
