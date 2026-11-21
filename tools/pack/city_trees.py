"""Pack mapped tree positions on the same coordinate grid as the city buildings."""
import gzip
import io
import struct
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

WEST, SOUTH, EAST, NORTH = -7085000, -3365000, -7045000, -3330000
SCALE = 100000
packed = bytearray()
candidates = height_tags = tree_rows = 0
xml = subprocess.check_output(["osmium", "tags-filter", sys.argv[1], "-R",
                               "n/natural=tree", "w/natural=tree_row", "-f", "osm"])
for _, item in ET.iterparse(io.BytesIO(xml)):
    if item.tag == "node":
        candidates += 1
        lon, lat = float(item.attrib["lon"]), float(item.attrib["lat"])
        if WEST/SCALE <= lon <= EAST/SCALE and SOUTH/SCALE <= lat <= NORTH/SCALE:
            x, y = round(lon*SCALE) - WEST, round(lat*SCALE) - SOUTH
            packed.extend(struct.pack("<HH", x, y))
            height_tags += item.find("tag[@k='height']") is not None
        item.clear()
    elif item.tag == "way":
        tree_rows += 1
        item.clear()

output = Path("web/data/city_trees.bin.gz")
output.write_bytes(gzip.compress(packed, mtime=0))
print(f"{len(packed)//4:,} of {candidates:,} tree nodes inside the box; {height_tags:,} have a height tag")
print(f"{tree_rows:,} tree-row ways excluded; {output.stat().st_size:,} gzip bytes")
