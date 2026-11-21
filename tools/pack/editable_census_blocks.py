"""Export the INE census block polygons of the Metropolitan Region to WGS84."""
import argparse
import gzip
import json
from pathlib import Path

import fiona
from shapely import make_valid
from pyproj import Transformer
from shapely.geometry import mapping, shape
from shapely.ops import transform, unary_union


def polygon_parts(geom):
    if geom.geom_type == "Polygon":
        yield geom
    elif hasattr(geom, "geoms"):
        for part in geom.geoms:
            yield from polygon_parts(part)


def export(source, output):
    blocks = {}
    for layer in ("Manzanas_CPV24", "Entidades_CPV24"):
        with fiona.open(source, layer=layer) as rows:
            project = Transformer.from_crs(rows.crs_wkt, 4326, always_xy=True)
            for row in rows:
                geometry = transform(project.transform, shape(row.geometry))
                if not geometry.is_valid:
                    geometry = make_valid(geometry)
                p = row.properties
                key = (str(int(p["MANZENT"])), str(int(p["CUT"])))
                blocks.setdefault(key, []).extend(polygon_parts(geometry))
    with output.open("wb") as file, gzip.GzipFile(filename="", fileobj=file, mode="wb", mtime=0) as stream:
        stream.write(b'{"type":"FeatureCollection","features":[')
        count = 0
        for (block_id, cut), parts in sorted(blocks.items()):
            if not parts:
                continue
            geom = unary_union(parts) if len(parts) > 1 else parts[0]
            point = geom.representative_point()
            record = {
                "type": "Feature",
                "properties": {"block_id": block_id, "cut": cut, "longitude": point.x, "latitude": point.y},
                "geometry": mapping(geom),
            }
            if count:
                stream.write(b",")
            stream.write(json.dumps(record, separators=(",", ":"), allow_nan=False).encode())
            count += 1
        stream.write(b"]}\n")
    print(f"{count} blocks, {output.stat().st_size} gzip bytes")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path, nargs="?", default=Path("data/mvp/building_blocks.geojson.gz"))
    args = parser.parse_args()
    export(args.source, args.output)
