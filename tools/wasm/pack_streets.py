"""Pack the viewer's road surfaces without changing their coordinates."""
import gzip
import json
import struct
from itertools import groupby
from pathlib import Path

SOURCE = Path("web/data/road_surface.geojson.gz")
OUTPUT = Path("web/wasm/road-surface.bin.gz")
DEFERRED = Path("web/wasm/road-surface-deferred.bin")
FEATURE = b'{"type":"Feature"'
ROAD_USES = {"road": 0, "cycle": 1, "foot": 2, "sidewalk": 3}


def coordinate(text):
    # Match serde_json's default conversion from a decimal significand.
    whole, fraction = text.split(".")
    return float(int(whole + fraction)) / 10 ** len(fraction)


def features(source):
    # The road exporter starts each feature with the same fixed prefix.
    first = source.read(1 << 20)
    pending = first.split(FEATURE, 1)[1]
    while True:
        parts = pending.split(b"," + FEATURE)
        pending = parts.pop()
        for part in parts:
            yield json.loads(FEATURE + part, parse_float=coordinate)
        chunk = source.read(1 << 20)
        if not chunk:
            break
        pending += chunk
    yield json.loads(FEATURE + pending.removesuffix(b"]}\n"), parse_float=coordinate)


def varint(out, value):
    while value >= 128:
        out.append((value & 127) | 128)
        value >>= 7
    out.append(value)


def zigzag(value):
    return 2 * value if value >= 0 else -2 * value - 1


def bits(value):
    return struct.unpack("<Q", struct.pack("<d", value))[0]


def encode_tile(group):
    out = bytearray()
    for feature in group:
        props = feature["properties"]
        if props["structure"] == "tunnel":
            continue
        rings = feature["geometry"]["coordinates"]
        for value in (props["tile"], zigzag(props["layer"]), ROAD_USES.get(props["use"], 0), len(rings)):
            varint(out, value)
        for ring in rings:
            varint(out, len(ring) - 1)
            previous = [0, 0]
            for point in ring[:-1]:
                for axis, value in enumerate(point):
                    millionths = round(value * 1_000_000)
                    rounded = millionths / 1_000_000
                    if value != rounded:
                        out.append(0)
                        varint(out, zigzag(bits(value) - bits(rounded)))
                    varint(out, 1 + zigzag(millionths - previous[axis]))
                    previous[axis] = millionths
    return out


def startup_tile(tile):
    # The road pack uses the building grid. A 5.7 km center radius covers the
    # opening frustum and its shadows with at least one tile of margin.
    col, row = tile % 20, tile // 20
    x = (-70.85 + (col + 0.5) * 0.02 + 70.6505) * 92_915
    z = (-33.65 + (row + 0.5) * 0.02 + 33.443) * 110_905
    return x * x + z * z <= 5_700 ** 2


startup_tmp = OUTPUT.with_suffix(".tmp")
deferred_tmp = DEFERRED.with_suffix(".tmp")
with gzip.open(SOURCE, "rb") as source, startup_tmp.open("wb") as file, deferred_tmp.open("wb") as later, gzip.GzipFile(
    filename="", fileobj=file, mode="wb", compresslevel=6, mtime=0
) as pack:
    for tile, group in groupby(features(source), key=lambda f: f["properties"]["tile"]):
        encoded = encode_tile(group)
        if startup_tile(tile):
            pack.write(encoded)
        elif encoded:
            compressed = gzip.compress(encoded, compresslevel=6, mtime=0)
            later.write(struct.pack("<HI", tile, len(compressed)))
            later.write(compressed)
startup_tmp.replace(OUTPUT)
deferred_tmp.replace(DEFERRED)
