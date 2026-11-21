"""Write urban cycling infrastructure from Geofabrik OSM ways."""
from pathlib import Path

from roads import PACK, clipped_segments, distance, read_roads, rounded_path, write_json


def main():
    points, ways = read_roads(Path("data/raw/osm/cycleways.osm"))
    features = []
    metres = 0
    for way_id, tags, nodes in ways:
        if "highway" not in tags or tags.get("area") == "yes":
            continue
        kinds = {tags.get(key) for key in ("cycleway", "cycleway:left", "cycleway:right", "cycleway:both")}
        kind = "cycleway" if tags["highway"] == "cycleway" else "track" if "track" in kinds else "lane"
        for part in clipped_segments([points[node] for node in nodes]):
            path = rounded_path(part, 2)
            if len(path) < 2:
                continue
            metres += sum(distance(a, b) for a, b in zip(part, part[1:]))
            features.append({"type": "Feature", "properties": {"id": way_id, "kind": kind,
                             "name": tags.get("name", "")}, "geometry": {
                             "type": "LineString", "coordinates": path}})
    output = PACK / "cycleways.geojson"
    write_json(output, {"type": "FeatureCollection", "features": features})
    print(f"{len(features)} paths, {metres / 1000:.1f} km, {output.stat().st_size} bytes")


if __name__ == "__main__":
    main()
