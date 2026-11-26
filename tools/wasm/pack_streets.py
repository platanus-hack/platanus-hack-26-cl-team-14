"""Pack the viewer's road surfaces without changing their coordinates."""
import gzip
import json
import struct
from pathlib import Path

SOURCE = Path("web/data/road_surface.geojson.gz")
OUTPUT = Path("web/wasm/road-surface.bin.gz")
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


temporary = OUTPUT.with_suffix(".tmp")
with gzip.open(SOURCE, "rb") as source, temporary.open("wb") as file, gzip.GzipFile(
    filename="", fileobj=file, mode="wb", compresslevel=6, mtime=0
) as pack:
    for feature in features(source):
        props = feature["properties"]
        if props["structure"] == "tunnel":
            continue
        rings = feature["geometry"]["coordinates"]
        out = bytearray()
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
        pack.write(out)
temporary.replace(OUTPUT)
