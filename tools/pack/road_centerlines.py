"""Write motor-road centrelines from the source rows used to buffer the road surface."""
import csv
import gzip
import json
from pathlib import Path

from road_surface import RAW
from roads import clipped_segments

OUTPUT = Path("web/data/road_centerlines.geojson.gz")


def main():
    header = {"type": "FeatureCollection", "attribution": "© OpenStreetMap contributors",
              "license": "ODbL-1.0", "osm_timestamp": "2026-11-20T21:21:21Z"}
    way_count = part_count = 0
    with (RAW / "ways.csv").open() as source, OUTPUT.open("wb") as dest, gzip.GzipFile(
            filename="", fileobj=dest, mode="wb", mtime=0, compresslevel=6) as pack:
        pack.write(json.dumps(header, separators=(",", ":"), ensure_ascii=False).encode()[:-1] + b',"features":[')
        for row in csv.DictReader(source):
            if row["road_use"] != "road":
                continue
            width = float(row["width_m"])
            props = {"osm_way_id": int(row["osm_id"].removeprefix("w")), "highway": row["highway"],
                     "width_m": width, "left_half_width_m": width / 2, "right_half_width_m": width / 2,
                     "width_source": row["width_source"], "layer": int(row["layer"]), "structure": row["structure"]}
            coordinates = json.loads(row["geometry"])["coordinates"]
            for part, points in enumerate(clipped_segments(coordinates)):
                props["part"] = part
                geom = {"type": "LineString", "coordinates": points}
                feature = {"type": "Feature", "properties": props, "geometry": geom}
                if part_count:
                    pack.write(b",")
                pack.write(json.dumps(feature, separators=(",", ":")).encode())
                part_count += 1
                if part == 0:
                    way_count += 1
        pack.write(b"]}\n")
        raw_size = pack.tell()
    print(f"{way_count:,} motor ways, {part_count:,} parts, {raw_size:,} inflated bytes, "
          f"{OUTPUT.stat().st_size:,} gzip bytes")


if __name__ == "__main__":
    main()
