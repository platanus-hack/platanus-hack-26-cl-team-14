"""Copy the exact building polygons near the browser's initial camera."""
import gzip
import struct
from pathlib import Path

with gzip.open("web/data/city_building_polygons.bin.gz", "rb") as source:
    header = source.read(28)
    _, cols, rows, west, south, _, _, step = struct.unpack("<4sHH4iI", header)
    tile_count = cols * rows
    offsets = struct.unpack(f"<{tile_count + 1}I", source.read(4 * (tile_count + 1)))
    pack = bytearray(header)
    pack.extend(bytes(4 * (tile_count + 1)))
    for tile in range(tile_count):
        struct.pack_into("<I", pack, 28 + 4 * tile, len(pack))
        col, row = tile % cols, tile // cols
        lon = (west + (col + 0.5) * step) / 100_000
        lat = (south + (row + 0.5) * step) / 100_000
        # Rig::default starts at 1650 m. Near tiles can be up to 2688 m away.
        x = (lon + 70.6505) * 92_915
        z = (lat + 33.443) * 110_905
        if x * x + z * z <= 3_000 ** 2:
            source.seek(offsets[tile])
            pack.extend(source.read(offsets[tile + 1] - offsets[tile]))
    struct.pack_into("<I", pack, 28 + 4 * tile_count, len(pack))
Path("web/wasm/building-polygons-startup.bin.gz").write_bytes(
    gzip.compress(pack, compresslevel=6, mtime=0)
)
