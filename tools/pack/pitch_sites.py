"""Detailed parts of the two pitch sites: the MUT (base and four towers) and Estadio Monumental (shell, stands, pitch).

Geometry and tags come from the main OSM API (Overpass answered 502). A part without an OSM height uses, in order:
building:levels x 3.5 m, a cited press figure, or the median 2023 Google Open Buildings 2.5D height of the
4 m pixels that lie fully inside it (presence >= 0.7), the same raster and filter as satellite_roof_heights.py.
Needs rasterio, shapely, pyproj, numpy and certifi (see context/research/santiago-roof-heights.md).
"""
import json
import math
import ssl
from pathlib import Path
from urllib.request import Request, urlopen

import certifi
import numpy as np
import rasterio
import shapely
from pyproj import Transformer
from rasterio.windows import Window

from satellite_roof_heights import MANIFEST, PIXEL

CTX = ssl.create_default_context(cafile=certifi.where())
OSM = "https://www.openstreetmap.org"
SITES = {
    "mut": {
        "out": Path("web/data/mut_parts.geojson"),
        "ids": {"relation": [18760408], "way": [966338859, 966338861, 966338860, 1201729148]},
        "kind": lambda t: "podium" if t.get("name") == "Mercado Urbano Tobalaba" else "tower",
    },
    "estadio_monumental": {
        "out": Path("web/data/estadio_monumental_parts.geojson"),
        # Shell, the nine named stands, the leisure=stadium outline and the main pitch inside it.
        "ids": {"relation": [8641055], "way": [297163434, 52100765, 1152713168, 1152713452, 1152713567, 1152714227,
                                               1152714337, 1152714756, 1152714966, 1152715018, 1152715166]},
        "kind": lambda t: {"stadium": "stadium_shell", "grandstand": "stand"}.get(t.get("building"))
        or {"stadium": "grounds", "pitch": "pitch"}[t["leisure"]],
    },
}
# Torre 4 (MUT El Bosque) has no OSM height or levels; La Tercera (2025) reports four towers of 19 floors.
PRESS = {1201729148: (19 * 3.5, "press_levels", "https://www.latercera.com/pulso/noticia/los-cuatro-fantasticos-de-las-oficinas-en-el-mercado-chileno/")}


def fetch(ids):
    """Each object with its nodes (and member ways) from the main OSM API."""
    out = []
    for t, numbers in ids.items():
        for n in numbers:
            req = Request(f"https://api.openstreetmap.org/api/0.6/{t}/{n}/full.json", headers={"User-Agent": "team14-pitch-sites"})
            elements = json.load(urlopen(req, context=CTX, timeout=60))["elements"]
            nodes = {e["id"]: [e["lon"], e["lat"]] for e in elements if e["type"] == "node"}
            ways = {e["id"]: [nodes[i] for i in e["nodes"]] for e in elements if e["type"] == "way"}
            obj = next(e for e in elements if e["type"] == t and e["id"] == n)
            if t == "way":
                rings = [ways[n]]
            else:  # each site relation has one closed outer way
                rings = [ways[m["ref"]] for m in obj["members"] if m["role"] == "outer"][:1] + \
                        [ways[m["ref"]] for m in obj["members"] if m["role"] == "inner"]
            out.append((t, n, obj["tags"], {"type": "Polygon", "coordinates": rings}))
    return out


def satellite(features):
    """Median Google 2023 height of interior 4 m pixels, for each feature that still has no height."""
    todo = [f for f in features if "height_m" not in f["properties"] and f["properties"]["kind"] in ("stand", "stadium_shell", "tower")]
    if not todo:
        return
    utm = Transformer.from_crs(4326, 32719, always_xy=True)
    polys = shapely.transform([shapely.geometry.shape(f["geometry"]) for f in todo], utm.transform, interleaved=False)
    manifest = json.load(urlopen(MANIFEST, context=CTX))
    prefix = manifest["uriPrefix"].replace("gs://", "https://storage.googleapis.com/")
    samples = [[] for _ in todo]
    with rasterio.Env(GDAL_DISABLE_READDIR_ON_OPEN="EMPTY_DIR", CURL_CA_BUNDLE=certifi.where(),
                      GDAL_INGESTED_BYTES_AT_OPEN="131072", GDAL_HTTP_MULTIRANGE="SERIAL", GDAL_HTTP_TIMEOUT="60"):
        for source in manifest["tilesets"][0]["sources"]:
            a = source["affineTransform"]
            left, top = a["translateX"], a["translateY"]
            width, depth = source["dimensions"]["width"] * a["scaleX"], -source["dimensions"]["height"] * a["scaleY"]
            for i, p in enumerate(polys):
                x0, y0, x1, y1 = p.bounds
                if not (left <= x0 and x1 < left + width and top - depth < y0 and y1 <= top):
                    continue
                c0, c1 = int((x0 - left) / PIXEL), int((x1 - left) / PIXEL) + 1
                r0, r1 = int((top - y1) / PIXEL), int((top - y0) / PIXEL) + 1
                with rasterio.open("GTIFF_DIR:13:/vsicurl/" + prefix + source["uris"][0]) as tile:
                    h, score = tile.read([2, 3], window=Window.from_slices((r0, r1), (c0, c1)))
                rows, cols = np.mgrid[r0:r1, c0:c1]
                centers = shapely.points(left + PIXEL * (cols.ravel() + .5), top - PIXEL * (rows.ravel() + .5))
                # Same rule as the city lookup: the whole pixel lies inside the outline.
                inside = shapely.contains(p, centers) & (shapely.distance(p.boundary, centers) >= PIXEL / math.sqrt(2))
                keep = inside & (score.ravel() >= .7) & (h.ravel() >= 2) & (h.ravel() <= 100)
                samples[i] += h.ravel()[keep].tolist()
    for f, s in zip(todo, samples):
        if s:
            f["properties"].update(height_m=round(float(np.median(s)), 1), height_source="google_open_buildings_2023_median",
                                   height_pixels=len(s))


for site, spec in SITES.items():
    features = []
    for osm_type, osm_id, t, geometry in fetch(spec["ids"]):
        p = {"parent_id": site, "kind": spec["kind"](t), "name": t.get("name"), "osm_type": osm_type, "osm_id": osm_id,
             "geometry_source": f"{OSM}/{osm_type}/{osm_id}"}
        if "height" in t:
            p.update(height_m=float(t["height"]), height_source="osm_height_tag")
        elif "building:levels" in t:
            p.update(height_m=float(t["building:levels"]) * 3.5, height_source="osm_levels_x_3.5")
        elif osm_id in PRESS:
            h, src, url = PRESS[osm_id]
            p.update(height_m=h, height_source=src, height_source_url=url)
        features.append({"type": "Feature", "id": f"{p['kind']}_{osm_id}", "properties": p, "geometry": geometry})
    satellite(features)
    # A stand with no pixel that passes all filters (whole pixel inside, presence, 2-100 m) takes the shell median.
    shell = next((f["properties"] for f in features if f["properties"]["kind"] == "stadium_shell"), None)
    for f in features:
        if f["properties"]["kind"] == "stand" and "height_m" not in f["properties"]:
            f["properties"].update(height_m=shell["height_m"], height_source="stadium_shell_median")
    features.sort(key=lambda f: f["id"])
    spec["out"].write_text(json.dumps({"type": "FeatureCollection", "attribution": "© OpenStreetMap contributors; Google Open Buildings 2.5D Temporal",
                                       "license": "ODbL-1.0", "features": features}, ensure_ascii=False, separators=(",", ":")))
    print(spec["out"], len(features), "parts", spec["out"].stat().st_size, "bytes")
    for f in features:
        q = f["properties"]
        print(" ", q["kind"], q["name"], q.get("height_m"), q.get("height_source"), q.get("height_pixels", ""))
