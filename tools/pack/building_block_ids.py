"""Attach INE IDs to the exact quantized rings in the published polygon mesh."""
import csv
import gzip
import json
import struct
import sys
from array import array
from pathlib import Path

from building_mesh import little_endian, quantize

RAW = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("data/raw/building-fill")
WEB = Path("web/data")
MISSING = 65535


def features():
    with (RAW / "real_blocks.csv").open() as f:
        real = {(r["osm_id"], int(r["part"])): (r["block_id"], r["cut"]) if r["block_id"] else None
                for r in csv.DictReader(f)}
    with (RAW / "real_rendered.geojsonseq").open() as f:
        for line in f:
            feature = json.loads(line)
            p = feature["properties"]
            yield False, feature["geometry"]["coordinates"], real[(p["osm_id"], p["part"])]
    with (RAW / "generated.geojsonseq").open() as f:
        for line in f:
            feature = json.loads(line)
            p = feature["properties"]
            yield True, feature["geometry"]["coordinates"], (p["block_id"], p["cut"])


blocks, by_geometry = {}, {}
for generated, polygon, assignment in features():
    rings = []
    for ring in polygon:
        points = quantize(ring)
        if points:
            rings.append(little_endian(array("H", (c for point in points for c in point))))
    key = blocks.setdefault(assignment, len(blocks)) if assignment else MISSING
    by_geometry[generated, tuple(rings)] = key

mesh = gzip.decompress((WEB / "city_building_polygons.bin.gz").read_bytes())
cols, rows = struct.unpack_from("<HH", mesh, 4)
tiles = struct.unpack_from(f"<{cols * rows + 1}I", mesh, 28)
offsets, entries = array("I", [0]), array("H")
for start, end in zip(tiles, tiles[1:]):
    if start != end:
        count, nr, nv = struct.unpack_from("<III", mesh, start)
        pos = start + 12
        polys = struct.unpack_from(f"<{count + 1}I", mesh, pos)
        pos += 4 * (count + 1)
        rings = struct.unpack_from(f"<{nr + 1}I", mesh, pos)
        pos += 4 * (nr + 1)
        flags = pos + 4 * nv + 2 * count
        for i in range(count):
            shape = tuple(mesh[pos + 4*rings[j]:pos + 4*rings[j+1]] for j in range(polys[i], polys[i+1]))
            entries.append(by_geometry[bool(mesh[flags+i] & 4), shape])
    offsets.append(len(entries))

output = WEB / "city_building_block_ids.bin.gz"
with output.open("wb") as raw, gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0, compresslevel=6) as out:
    out.write(struct.pack("<4sHHII", b"SGBI", cols, rows, len(blocks), len(entries)))
    out.write(little_endian(offsets))
    for block_id, cut in blocks:
        out.write(struct.pack("<QH", int(block_id), int(cut)))
    out.write(little_endian(entries))
print(f"{len(entries):,} buildings, {len(blocks):,} block IDs, {output.stat().st_size:,} gzip bytes")
