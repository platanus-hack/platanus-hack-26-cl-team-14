"""Match 2023 Google Open Buildings height estimates to interior pixels of OSM roofs."""
import csv
import gzip
import io
import json
import math
import sys
import ssl
from pathlib import Path
from urllib.request import urlopen

import certifi
import numpy as np
import rasterio
import shapely
from pyproj import Transformer
from rasterio.windows import Window
from shapely.geometry import shape

MANIFEST = "https://storage.googleapis.com/open-buildings-temporal-data/v1/manifests/97_EPSG_32719_2023_06_30.json"
OUTPUT = Path("data/mvp/satellite_roof_heights.csv.gz")
PIXEL = 4  # metres, native 0.5 m grid at overview factor 8


def source_roofs(path):
    keys, polygons, tagged = [], [], []
    with path.open() as src:
        for line in src:
            feature = json.loads(line)
            p = feature["properties"]
            keys.append((p["osm_id"], p["part"]))
            polygons.append(shape(feature["geometry"]))
            tagged.append(p["height_m"] or 0)
    project = Transformer.from_crs(4326, 32719, always_xy=True)
    polygons = shapely.transform(polygons, project.transform, interleaved=False)
    centers = shapely.point_on_surface(polygons)
    return keys, polygons, np.array(tagged), shapely.get_x(centers), shapely.get_y(centers)


def match(source, prefix, polygons, x, y, heights):
    affine = source["affineTransform"]
    left, top = affine["translateX"], affine["translateY"]
    width = source["dimensions"]["width"] * affine["scaleX"]
    depth = -source["dimensions"]["height"] * affine["scaleY"]
    rows = np.flatnonzero((x >= left) & (x < left + width) & (y <= top) & (y > top - depth) & (heights == 0))
    if not len(rows):
        return
    col = ((x[rows] - left) / PIXEL).astype(int)
    row = ((top - y[rows]) / PIXEL).astype(int)
    center = shapely.points(left + PIXEL * (col + 0.5), top - PIXEL * (row + 0.5))
    # The entire square fits if its centre lies inside and a half diagonal clears every ring.
    roofs = polygons[rows]
    inside = shapely.contains(roofs, center) & (shapely.distance(shapely.boundary(roofs), center) >= PIXEL / math.sqrt(2))
    rows, col, row = rows[inside], col[inside], row[inside]
    if not len(rows):
        return
    url = prefix + source["uris"][0]
    # GDAL opens just the factor-eight TIFF directory. The manifest supplies its transform.
    with rasterio.open("GTIFF_DIR:13:/vsicurl/" + url) as tile:
        if tile.width != math.ceil(width / PIXEL) or tile.height != math.ceil(depth / PIXEL):
            raise ValueError(f"Unexpected overview dimensions for {url}")
        window = Window.from_slices((int(row.min()), int(row.max()) + 1), (int(col.min()), int(col.max()) + 1))
        values, presence = tile.read([2, 3], window=window)
        a, b = row - row.min(), col - col.min()
        h, score = values[a, b], presence[a, b]
        accepted = (score >= 0.7) & (h >= 2) & (h <= 100)
        heights[rows[accepted]] = np.rint(10 * h[accepted]).astype(np.uint16)
    print(source["uris"][0], int(accepted.sum()), flush=True)


def main():
    with urlopen(MANIFEST, context=ssl.create_default_context(cafile=certifi.where())) as src:
        manifest = json.load(src)
    keys, polygons, tagged, x, y = source_roofs(Path(sys.argv[1]))
    heights = np.zeros(len(keys), dtype=np.uint16)
    prefix = manifest["uriPrefix"].replace("gs://", "https://storage.googleapis.com/")
    bounds = (x.min(), y.min(), x.max(), y.max())
    with rasterio.Env(GDAL_DISABLE_READDIR_ON_OPEN="EMPTY_DIR", CURL_CA_BUNDLE=certifi.where(),
                      GDAL_INGESTED_BYTES_AT_OPEN="131072", GDAL_HTTP_MULTIRANGE="SERIAL",
                      GDAL_HTTP_TIMEOUT="60"):
        for source in manifest["tilesets"][0]["sources"]:
            a = source["affineTransform"]
            w = source["dimensions"]["width"]
            h = source["dimensions"]["height"]
            if (a["translateX"] < bounds[2] and a["translateX"] + w * a["scaleX"] > bounds[0]
                    and a["translateY"] > bounds[1] and a["translateY"] + h * a["scaleY"] < bounds[3]):
                match(source, prefix, polygons, x, y, heights)

    comparison = (heights > 0) & (tagged > 0)
    error = abs(heights[comparison] / 10 - tagged[comparison])
    print(f"{len(error)} matched OSM height tags; median absolute difference {np.median(error):.2f} m")
    with OUTPUT.open("wb") as out, gzip.GzipFile(filename="", fileobj=out, mode="wb", mtime=0) as gz:
        with io.TextIOWrapper(gz, newline="") as text:
            writer = csv.writer(text, lineterminator="\n")
            writer.writerow(["osm_id", "part", "height_dm"])
            writer.writerows((*keys[i], int(heights[i])) for i in np.flatnonzero(heights))
    print(f"{np.count_nonzero(heights)} estimates, {OUTPUT.stat().st_size} bytes")


if __name__ == "__main__":
    main()
