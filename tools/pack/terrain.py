"""Crop the Copernicus GLO-90 Santiago tile to the city view at its native spacing."""
import json
import struct
import zlib
from itertools import accumulate
from pathlib import Path


TILE = "Copernicus_DSM_COG_30_S34_00_W071_00_DEM"
SOURCE = Path("data/raw/copernicus-dem") / f"{TILE}.tif"
OUTPUT = Path("web/data")
WEST, SOUTH, EAST, NORTH = -70.85, -33.65, -70.45, -33.30
SAMPLES_PER_DEGREE = 1200

# The source GeoTIFF gives sample (0,0) at (-71,-33), RasterPixelIsPoint.
left = round((WEST + 71) * SAMPLES_PER_DEGREE)
top = round((-33 - NORTH) * SAMPLES_PER_DEGREE)
width = round((EAST - WEST) * SAMPLES_PER_DEGREE) + 1
height = round((NORTH - SOUTH) * SAMPLES_PER_DEGREE) + 1

# This GeoTIFF stores the full-resolution band in one deflated 2048 x 2048 tile.
tiff = SOURCE.read_bytes()
directory = struct.unpack_from("<I", tiff, 4)[0]
count = struct.unpack_from("<H", tiff, directory)[0]
entries = struct.iter_unpack("<HHII", tiff[directory+2:directory+2+12*count])
tags = {tag: value for tag, _, _, value in entries}
tile_width, offset, length = tags[322], tags[324], tags[325]
tile = zlib.decompress(tiff[offset:offset+length])
samples = []
for row in range(top, top + height):
    start = row * 4 * tile_width
    # TIFF floating-point predictor 3 stores byte differences in four MSB-first planes.
    planes = bytes(total % 256 for total in accumulate(tile[start:start+4*tile_width]))
    for col in range(left, left + width):
        sample = planes[col::tile_width]
        samples.append(round(struct.unpack(">f", sample)[0]))

(OUTPUT / "terrain.i16").write_bytes(struct.pack(f"<{len(samples)}h", *samples))
metadata = {
    "file": "terrain.i16", "width": width, "height": height,
    "encoding": "int16-le", "units": "metres", "vertical_datum": "EGM2008",
    "horizontal_crs": "EPSG:4326", "bbox": [WEST, SOUTH, EAST, NORTH],
    "origin_lon_lat": [WEST, NORTH], "step_lon_lat": [1/SAMPLES_PER_DEGREE, -1/SAMPLES_PER_DEGREE],
    "bounds_reference": "outermost sample centres", "row_order": "north to south", "column_order": "west to east",
    "minimum_m": min(samples), "maximum_m": max(samples), "nodata": None,
    "surface": "DSM including buildings and vegetation",
    "source": f"https://copernicus-dem-90m.s3.amazonaws.com/{TILE}/{TILE}.tif",
    "license": "https://dataspace.copernicus.eu/sites/default/files/media/files/2025-06/copernicus_contributing_mission_data_access_v2_cop_dem_licenses.pdf",
    "attribution": "produced using Copernicus WorldDEM-90 © DLR e.V. 2010-2014 and © Airbus Defence and Space GmbH 2014-2018 provided under COPERNICUS by the European Union and ESA; all rights reserved",
}
(OUTPUT / "terrain.json").write_text(json.dumps(metadata, ensure_ascii=False, indent=2) + "\n", encoding="utf8")
print(f"{width} × {height}, {min(samples)} to {max(samples)} m")
