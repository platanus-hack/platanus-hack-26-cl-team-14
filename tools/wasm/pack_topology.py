"""Pack the nodes and directed edges used by the car intersection viewer."""
import gzip
import json
from pathlib import Path

SOURCE = Path("web/data/local_car_graph.json.gz")
OUTPUT = Path("web/wasm/car-topology.bin.gz")
CHUNK = 1 << 20


def members(section):
    """Read the members of one object in the graph export."""
    marker = f'"{section}":{{'
    decoder = json.JSONDecoder()
    data = ""
    with gzip.open(SOURCE, "rt", encoding="utf-8") as source:
        for chunk in iter(lambda: source.read(CHUNK), ""):
            data = data[-len(marker):] + chunk
            start = data.find(marker)
            if start >= 0:
                break
        else:
            raise ValueError(f"Missing graph section {section}")
        at = start + len(marker)
        while True:
            if len(data) - at < CHUNK // 2:
                data = data[at:] + source.read(CHUNK)
                at = 0
            if data[at] == "}":
                break
            while True:
                try:
                    key, after_key = decoder.raw_decode(data, at)
                    value, at = decoder.raw_decode(data, after_key + 1)
                    break
                except json.JSONDecodeError:
                    chunk = source.read(CHUNK)
                    if not chunk:
                        raise
                    data += chunk
            yield key, value
            if at == len(data):
                data = source.read(CHUNK)
                at = 0
            if data[at] == "}":
                break
            at += 1


def varint(out, value):
    while value >= 128:
        out.append((value & 127) | 128)
        value >>= 7
    out.append(value)


def zigzag(value):
    return 2 * value if value >= 0 else -2 * value - 1


def coordinate(value):
    # OSM coordinates have at most seven decimal places.
    units = round(value * 10_000_000)
    if units / 10_000_000 != value:
        raise ValueError(f"Unexpected OSM node coordinate {value}")
    return units


nodes = {}
positions = bytearray()
previous = (0, 0, 0)
for key, (lon, lat) in members("nodes"):
    if not key.isascii() or not key.isdecimal():  # Skip clipped endpoints as the viewer does.
        continue
    nodes[key] = len(nodes)
    current = (int(key), coordinate(lon), coordinate(lat))
    for value, base in zip(current, previous):
        varint(positions, zigzag(value - base))
    previous = current

temporary = OUTPUT.with_suffix(".tmp")
previous = 0
with temporary.open("wb") as file, gzip.GzipFile(
    filename="", fileobj=file, mode="wb", compresslevel=6, mtime=0
) as pack:
    count = bytearray()
    varint(count, len(nodes))
    pack.write(count)
    pack.write(positions)
    for _, way in members("ways"):
        ground = way["structure"] == "ground"
        out = bytearray()
        for edge in way["edges"]:
            start, end = nodes.get(edge["from"]), nodes.get(edge["to"])
            if start is None or end is None:
                continue
            varint(out, zigzag(start - previous))
            varint(out, (zigzag(end - start) << 1) | ground)
            previous = end
        pack.write(out)
temporary.replace(OUTPUT)
