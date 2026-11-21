"""Paint the miniature vehicles. Each horizontal atlas tile is one transparent top view."""
from pathlib import Path
import struct
import sys
import zlib

# X runs from tail to nose, +Z down the image. Each image plane sits 0.12 m above the last.
PALETTE = {
    " ": (0, 0, 0, 0),
    "ink": (24, 33, 45, 255), "tyre": (41, 48, 57, 255),
    "hub": (172, 186, 192, 255), "metal": (137, 154, 160, 255),
    "glass": (43, 62, 83, 255), "sky": (91, 126, 151, 255),
    "shine": (142, 173, 185, 255), "cream": (244, 238, 220, 255),
    "gold": (235, 160, 53, 255), "ochre": (158, 91, 27, 255),
    "sun": (255, 200, 98, 255), "lamp": (255, 225, 152, 255),
    "red": (216, 51, 49, 255), "wine": (143, 39, 45, 255),
    "silver": (213, 222, 222, 255), "green": (55, 143, 102, 255),
    "coat": (49, 128, 158, 255), "skin": (223, 172, 131, 255),
}

# Tail, nose and half-width, in pixels, for each successive horizontal slice of the coupe.
CAR_SHAPE = [
    (6,35,4),(5,36,5),(3,38,7),(1,40,8),(1,41,8),(0,41,9),
    (1,41,9),(1,40,9),(2,39,8),(6,32,8),(8,30,7),(9,29,7),
    (10,28,7),(11,27,6),(12,26,6),
]


def car(x, y, z):
    a, b, w = CAR_SHAPE[y]
    end = min(x - a, b - x)
    tyre = min((x - axle)**2 + (y - 3.25)**2 for axle in (8, 32))
    if 6.5 <= z <= 9.5 and tyre <= 15:
        if z >= 8.5 and tyre < 4.9:
            return "ink" if tyre < 0.6 else "hub"
        return "ink" if tyre > 10 else "tyre"
    if z >= 6.5 and tyre < 23 and y <= 7:
        return " "
    if y == 9 and 1 <= x <= 5 and z <= 7.5:
        return "ochre" if x < 3 else "gold"
    if end < 0 or z > w + 0.5 - max(0, 2 - end):
        return " "
    if y <= 2:
        return "tyre" if y == 2 else "ink"
    if y == 3 and (end < 3 or z > w-1):
        return "hub" if z > 4.5 else "ochre"
    if x >= 36 and y in (4,5,6) and z >= 4.5:
        return "lamp" if y == 5 else "ink"
    if x <= 3 and y in (4,5) and z >= 4.5:
        return "red"
    if x >= 39 and y in (4,5) and z <= 3.5:
        return "ink"
    if y >= 9:
        if y < 14 and end <= 2 and z < w-0.5:
            return "sky" if y >= 12 and x % 6 < 2 else "glass"
        if y < 14 and z >= w-1.5 and a+1 <= x <= b-1:
            return "ochre" if x in (16,17) else "sky" if y == 13 else "glass"
        return "cream" if z <= 1.5 else "sun" if z >= w-0.5 else "gold"
    if z >= w-0.5 and 12 <= x <= 29:
        if x in (14,15) and y == 6:
            return "hub"
        if y == 4 or x in (12,27) and y <= 6:
            return "ochre"
    if z <= 1.5 and y >= 6:
        return "cream"
    if y >= 7 and x in (6,7,34,35) and 4.5 <= z <= 6.5:
        return "ochre"
    return "gold"


# Width and tapers change with height. Both transit vehicles have a stepped windscreen.
def transit(metro, x, y, z):
    length, width, height = (112,12,28) if metro else (96,11,27)
    a = (8,6,4,3,2,1)[y] if y < 6 else 1 + max(0, y - height + 6)//2
    b = length - 1 - ((7,5,4,3,2,1)[y] if y < 6 else max(0, y - height + 8)//2)
    w = width - (4,3,2,1)[y] if y < 4 else width - max(0,y-height+4)//2
    end = min(x-a, b-x)
    axles = (13,20,89,96) if metro else (15,75)
    wheel = min((x-k)**2+(y-4.2)**2 for k in axles)
    if width-2.5 <= z <= width+0.5 and wheel <= 24:
        if z >= width-0.5 and wheel < 8.5:
            return "tyre" if wheel < 1.2 else "hub"
        return "ink" if wheel > 17 else "tyre"
    if z >= width-2.5 and wheel < 32 and y < 10:
        return " "
    if y >= height-3:
        # Two raised cooling units on a bus, three on a Metro car.
        vents = ((15,29),(47,61),(78,89)) if metro else ((13,29),(59,75))
        for start,stop in vents:
            margin = y-height+3
            if start+margin <= x <= stop-margin and z <= 4.5-margin:
                return "tyre" if (x-start)%4 == 2 and z < 3.5 else "hub"
        return " "
    if end < 0 or z > w+0.5-max(0,(3 if metro else 2)-end):
        return " "
    if y <= 2:
        return "ink"
    if y <= 4:
        return "metal" if z >= w-0.5 else "tyre"
    if y < 10 and end <= 2:
        if z >= w-3.5 and y in (7,8):
            return "lamp" if x > length//2 else "red"
        if y == 5 and z <= 2.5:
            return "ink"
    if y >= height-5:
        if z >= w-0.5 and y == height-5:
            return "wine" if metro else "red"
        return "cream" if not metro else "silver"
    doors = (15,39,64,89) if metro else (8,41,81)
    near_door = min(abs(x-k) for k in doors)
    if z >= w-1.5 and near_door <= 3 and 5 <= y < 20:
        if near_door in (0,3):
            return "ink" if not metro else "metal"
        return "glass" if y >= 10 else "metal" if metro else "tyre"
    if end <= 3 and y >= 12:
        if not metro and y >= 20 and x > length//2 and z < 5.5:
            return "lamp" if int(z)%3 else "ink"
        return "sky" if y >= 19 and z < 4.5 else "glass"
    if 13 <= y < height-5 and z >= w-2.5:
        if x % (14 if metro else 10) in (0,1):
            return "silver" if metro else "ink"
        return "sky" if y >= 19 else "glass"
    if 14 <= y < height-5:
        return "sky" if z < 2.5 else "glass"
    if y in (10,11,12):
        return "red" if metro else "cream"
    if y in (5,6):
        return "metal" if metro else "wine"
    return "silver" if metro else "red"


def bicycle(x, y, z):
    # Tire rings, diagonal frame tubes, handlebars and rider. Side profile is +X.
    d = min((x-a)**2 + (y-3.2)**2 for a in (3.5,13.5))
    if z <= 0.5 and 5.5 <= d <= 11.0:
        return "tyre"
    if z <= 0.5 and (abs(y-3.4) < 0.6 and 3<=x<=9
                       or 6<=x<=10 and abs(y-(x-4.5)) < 1
                       or 8<=x<=14 and abs(y-(-x+16)) < 1):
        return "green"
    if 11<=x<=13 and 6<=y<=8 and z<=2.5:
        return "ink"
    if (x-8.5)**2 + (z*1.3)**2 < 5 and 7<=y<=11:
        return "coat"
    if (x-9.5)**2 + (z*1.2)**2 + (y-13)**2 < 6:
        return "skin" if y<14 else "cream"
    if 5<=y<=8 and abs(x-7.5) < 1.2 and z<=1.5:
        return "tyre"
    return " "


# Tile width, tile height, number of horizontal slices and the artwork function.
SHEETS = {
    "car": (43,24,15,car),
    "bus": (96,26,27,lambda x,y,z: transit(False,x,y,z)),
    "metro": (112,30,28,lambda x,y,z: transit(True,x,y,z)),
    "bicycle": (18,10,16,bicycle),
}

def write_sheet(path, width, depth, height, art):
    # The shaded outer ink follows each cross section. Lamps, hubs and the stripe keep their paint.
    rim = {"gold":"ochre","red":"wine","silver":"metal","glass":"ink",
           "green":"tyre","coat":"glass","sky":"glass","sun":"gold"}
    tiles = []
    for y in range(height):
        tile = [[art(x, y, abs(z - (depth-1)/2)) for x in range(width)] for z in range(depth)]
        colors = []
        for z, row in enumerate(tile):
            band = bytearray()
            for x, color in enumerate(row):
                if color != " " and (x == 0 or row[x-1] == " " or z == depth-1 or tile[z+1][x] == " "):
                    color = rim.get(color, color)
                band.extend(PALETTE[color])
            colors.append(band)
        tiles.append(colors)
    rgba = b"".join(b"\0"+b"".join(tile[z] for tile in tiles) for z in range(depth))
    def chunk(tag, data):
        return struct.pack(">I",len(data))+tag+data+struct.pack(">I",zlib.crc32(tag+data))
    ihdr = struct.pack(">IIBBBBB",width*height,depth,8,6,0,0,0)
    path.write_bytes(b"\x89PNG\r\n\x1a\n"+chunk(b"IHDR",ihdr)+chunk(b"IDAT",zlib.compress(rgba,9))+chunk(b"IEND",b""))

if __name__ == "__main__":
    target = Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parents[2]/"viz"/"assets"
    target.mkdir(parents=True, exist_ok=True)
    for name, spec in SHEETS.items():
        write_sheet(target/(name+".png"), *spec)
