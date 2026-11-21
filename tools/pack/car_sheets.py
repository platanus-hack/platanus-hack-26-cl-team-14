"""Pack the miniature car fleet from INE 2024, Municipalidad de Pudahuel (2023, CC BY)
and the regional Chileautos used-listing sample. See data/mvp/car_{mix,colours}.md
and the Vehicle types and colours section of data/mvp/README.md for source links.
"""
import csv
import json
import struct
import zlib
from collections import defaultdict
from pathlib import Path

from vehicle_sheets import PALETTE

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "data/mvp"
ASSETS = ROOT / "viz/assets"
# One cell stores 22 horizontal slices, back to front along +X. Two cells fit across the atlas.
WIDTH, DEPTH, SLICES, COLUMNS = 54, 24, 22, 2
# Length, half-width, height, rear and front of cabin (pixels at 12 cm).
PROFILES = {
    "automovil": (37, 8, 13, 7, 27),
    "sedan": (40, 8, 13, 8, 29),
    "hatchback": (34, 8, 14, 3, 24),
    "station_wagon": (41, 8, 14, 3, 30),
    "suv": (40, 9, 16, 3, 29),
    "todo_terreno": (38, 9, 17, 3, 25),
    "jeep": (32, 8, 16, 2, 21),
    "cabriolet": (36, 8, 11, 7, 24),
    "camioneta": (45, 9, 16, 17, 34),
    "furgon": (43, 9, 19, 1, 36),
    "minibus_particular": (52, 10, 21, 1, 46),
}
PASSENGER_CLASS = "automovil_station_wagon_todoterreno"
UTILITY = ("camioneta", "furgon", "minibus_particular")
# Representative sRGB swatches for the known source families. The sources do not report RGB.
SWATCHES = {
    "white": (236,235,226), "grey": (111,119,121), "silver": (184,194,197),
    "black": (47,50,54), "red": (184,54,52), "blue": (54,91,145),
    "brown": (126,88,67), "green": (66,106,83), "beige": (197,179,146),
    "orange": (214,125,55), "yellow": (222,183,60), "gold": (187,151,85),
    "light_blue": (129,176,199), "pink": (211,143,158), "purple": (124,91,145),
}


def pixel(body, x, y, z):
    length, width, height, rear, front = PROFILES[body]
    x -= (WIDTH - length) // 2
    if y >= height or x < 0 or x >= length:
        return " "
    boxy = body in ("furgon", "minibus_particular", "jeep", "todo_terreno")
    wheel = min((x - axle)**2 + (y - 3)**2 for axle in (6, length - 8))
    if z >= width - 1.5 and z <= width + 0.5 and wheel <= 14:
        return "hub" if wheel < 4 and z >= width - 0.5 else "tyre"
    if z >= width - 1.5 and wheel < 20:
        return " "
    if y < 3:
        return "ink" if 3 <= x < length - 3 and z < width - 2 else " "
    belt = 9 if body in UTILITY or body in ("suv", "jeep", "todo_terreno") else 8
    if y < belt:
        tip = min(x, length - x - 1)
        if z > width - 0.5 - max(0, 2 - tip):
            return " "
        if tip <= 1 and z > width - 4 and y in (5, 6):
            return "lamp" if x >= length / 2 else "red"
        if tip == 0 and y <= 4:
            return "metal"
        if body == "camioneta" and 2 < x < rear-1 and z < width-1.5 and y >= 6:
            return " " if y > 6 else "tyre" if x % 3 else "metal"
        return "shadow" if y <= 4 else "highlight" if y == belt-1 and z > width-2 else "paint"
    # Narrow the cabin toward the roof. The pickup leaves the rear bed open.
    level = (y - belt) / max(1, height - belt - 1)
    tail = rear + int(level * (1 if boxy or rear <= 3 else 3))
    nose = front - int(level * (2 if boxy else 5))
    side = width - 1 - int(level * (1 if boxy else 2))
    if not tail <= x <= nose or z > side + 0.5:
        return " "
    if body == "cabriolet":
        if x >= nose-1:
            return "glass" if y < height - 1 else "metal"
        if y == belt and z <= side-1 and (x-rear) % 6 < 3:
            return "tyre"
        return " "
    if y == height - 1:
        return "highlight" if z >= side - 0.5 else "paint"
    if x <= tail + 1 or x >= nose - 1:
        return "glass" if y <= height - 3 else "sky"
    if z >= side - 1.5 and not (body == "furgon" and x < front-11):
        return "shadow" if (nose-x) % 8 == 0 else "sky" if y == height - 2 else "glass"
    return "paint"


def rows(name):
    with (DATA / (name + ".csv")).open() as stream:
        return list(csv.DictReader(stream))


def weighted(items):
    total = sum(weight for _, weight in items)
    cumulative = 0.
    result = []
    for index, weight in items:
        cumulative += weight
        result.append([index, cumulative / total])
    result[-1][1] = 1.0
    return result


def distributions():
    bodies = list(PROFILES)
    colours = list(SWATCHES)
    municipal = defaultdict(list)
    for row in rows("car_bodies"):
        municipal[row["cut"]].append((row["class"], float(row["share"])))
    types = defaultdict(list)
    for row in rows("car_mix"):
        kind, share, cut = row["class"], float(row["share"]), row["cut"]
        # Engine car mode uses INE E-H only. Normalize these four shares within each issuing CUT.
        if kind == PASSENGER_CLASS:
            for body, fraction in municipal.get(cut, [("automovil", 1.)]):
                types[cut].append((bodies.index(body), share * fraction))
        elif kind in UTILITY:
            types[cut].append((bodies.index(kind), share))
    paints = defaultdict(list)
    for row in rows("car_colour_proxy") + rows("car_colours"):
        # Source tables retain unknown; a visible paint is drawn from the known families only.
        if row["colour"] != "unknown":
            paints[row["cut"]].append((colours.index(row["colour"]), float(row["share"])))
    return [{cut: weighted(items) for cut, items in table.items()} for table in (types, paints)]


def png(path, width, height, pixels):
    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    scan = b"".join(b"\0" + pixels[y*width*4:(y+1)*width*4] for y in range(height))
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) +
                     chunk(b"IDAT", zlib.compress(scan, 9)) + chunk(b"IEND", b""))


def main():
    styles = len(PROFILES) * len(SWATCHES)
    width = WIDTH * SLICES * COLUMNS
    height = DEPTH * ((styles + COLUMNS - 1) // COLUMNS)
    pixels = bytearray(width * height * 4)
    for b, body in enumerate(PROFILES):
        bands = [[pixel(body, x, y, abs(z - (DEPTH-1)/2))
                  for y in range(SLICES) for x in range(WIDTH)] for z in range(DEPTH)]
        for c, rgb in enumerate(SWATCHES.values()):
            palette = {key: bytes(value) for key, value in PALETTE.items()}
            for key, shade in (("paint", rgb), ("highlight", [int(v + (255-v)*0.23) for v in rgb]),
                               ("shadow", [int(v*0.73) for v in rgb])):
                palette[key] = bytes([*shade, 255])
            row, column = divmod(b * len(SWATCHES) + c, COLUMNS)
            for z, band in enumerate(bands):
                start = ((row*DEPTH + z)*width + column*WIDTH*SLICES)*4
                pixels[start:start+len(band)*4] = b"".join(palette[key] for key in band)
    png(ASSETS / "car_fleet.png", width, height, pixels)
    atlas = [styles, len(SWATCHES), len(PROFILES) - len(UTILITY), COLUMNS, SLICES]
    (ASSETS / "car_fleet.json").write_text(json.dumps([atlas, distributions()], separators=(",", ":")) + "\n")


if __name__ == "__main__":
    main()
