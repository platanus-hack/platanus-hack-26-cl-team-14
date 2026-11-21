"""Pack the INE block of every near/far mesh feature in the same tile order."""
import csv
import gzip
import json
import struct
import sys
from array import array
from itertools import accumulate
from pathlib import Path

from building_mesh import COLS, ROWS, STEP, little_endian, oriented_box, quantize

RAW = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("data/raw/building-fill")
OUT = Path("web/data/city_building_block_ids.bin.gz")
MISSING = 65535


def assignments():
    with (RAW / "real_blocks.csv").open() as f:
        real = {(r["osm_id"], int(r["part"])): (r["block_id"], r["cut"]) if r["block_id"] else None
                for r in csv.DictReader(f)}
    with (RAW / "real_rendered.geojsonseq").open() as f:
        for line in f:
            feature = json.loads(line)
            p = feature["properties"]
            yield feature["geometry"]["coordinates"][0], real[(p["osm_id"], p["part"])]
    with (RAW / "generated.geojsonseq").open() as f:
        for line in f:
            feature = json.loads(line)
            p = feature["properties"]
            yield feature["geometry"]["coordinates"][0], (p["block_id"], p["cut"])


tiles = [array("H") for _ in range(COLS * ROWS)]
blocks = {}
for shell, assignment in assignments():
    x, y, *_ = oriented_box(quantize(shell))
    col = min(COLS - 1, max(0, x // STEP))
    row = min(ROWS - 1, max(0, y // STEP))
    key = blocks.setdefault(assignment, len(blocks)) if assignment else MISSING
    tiles[row * COLS + col].append(key)

offsets = array("I", accumulate(map(len, tiles), initial=0))
with OUT.open("wb") as raw, gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0, compresslevel=6) as out:
    out.write(struct.pack("<4sHHII", b"SGBI", COLS, ROWS, len(blocks), offsets[-1]))
    out.write(little_endian(offsets))
    for block_id, cut in blocks:
        out.write(struct.pack("<QH", int(block_id), int(cut)))
    for tile in tiles:
        out.write(little_endian(tile))
print(f"{offsets[-1]:,} buildings, {len(blocks):,} block IDs, {OUT.stat().st_size:,} gzip bytes")
