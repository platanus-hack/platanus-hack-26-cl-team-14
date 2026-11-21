"""Build data/mvp/zones.csv: Census 2024 home zones (manzanas and rural entities) of the RM.

Usage: python3 -I tools/pack/zones.py
Reads data/raw/verify-people-work/census-blocks.zip and cartography-rm.gpkg.
"""
import csv, io, json, sqlite3, struct, zipfile

RAW = "data/raw/verify-people-work/"
ENV = [0, 32, 48, 48, 64]  # GeoPackage envelope bytes by flag code


def rings(b, o):
    """Read one WKB geometry at offset o; return (rings per polygon, new offset)."""
    e = "<" if b[o] == 1 else ">"
    t = struct.unpack_from(e + "I", b, o + 1)[0] % 1000
    o += 5
    if t == 6:  # MultiPolygon
        n = struct.unpack_from(e + "I", b, o)[0]
        o += 4
        polys = []
        for _ in range(n):
            p, o = rings(b, o)
            polys += p
        return polys, o
    nr = struct.unpack_from(e + "I", b, o)[0]  # Polygon
    o += 4
    poly = []
    for _ in range(nr):
        npt = struct.unpack_from(e + "I", b, o)[0]
        o += 4
        poly.append(struct.unpack_from(e + "%dd" % (2 * npt), b, o))
        o += 16 * npt
    return [poly], o


def centroid(blob):
    """Area-weighted centroid; holes subtract."""
    polys, _ = rings(blob, 8 + ENV[(blob[3] >> 1) & 7])
    A = X = Y = 0.0
    for poly in polys:
        for k, r in enumerate(poly):
            a = x = y = 0.0
            for i in range(0, len(r) - 2, 2):
                x0, y0, x1, y1 = r[i], r[i + 1], r[i + 2], r[i + 3]
                c = x0 * y1 - x1 * y0
                a += c
                x += (x0 + x1) * c
                y += (y0 + y1) * c
            s = (1 if k == 0 else -1) * (1 if a > 0 else -1)  # outer adds, holes subtract
            A += s * a / 2
            X += s * x / 6
            Y += s * y / 6
    return X / A, Y / A


db = sqlite3.connect(RAW + "cartography-rm.gpkg")
pts = {}
for layer in ("Manzanas_CPV24", "Entidades_CPV24"):
    for mz, shape in db.execute(f"select MANZENT, SHAPE from {layer}"):
        pts[str(int(mz))] = centroid(shape)
n = lambda v: 0 if v == "*" else int(float(v))  # '*' = suppressed by INE
src = zipfile.ZipFile(RAW + "census-blocks.zip").open("Base_manzana_entidad_CPV24.csv")
rows = [r for r in csv.DictReader(io.TextIOWrapper(src, "utf-8-sig"), delimiter=";") if r["COD_REGION"] == "13"]

# Comuna containers (CONTENEDOR_COMUNAL=1) have no polygon: put them at the
# person-weighted mean of the block centroids of their comuna (lands where people live).
acc = {}
for r in rows:
    if r["CONTENEDOR_COMUNAL"] == "0":
        lon, lat = pts[str(int(r["MANZENT"]))]
        w = n(r["n_per"])
        a = acc.setdefault(r["CUT"], [0, 0.0, 0.0])
        a[0] += w; a[1] += w * lon; a[2] += w * lat

out = csv.writer(open("data/mvp/zones.csv", "w", newline=""))
out.writerow(["manzana_id", "comuna_cut", "lon", "lat", "persons", "dwellings", "workers", "students"])
for r in rows:
    mz = str(int(r["MANZENT"]))
    if r["CONTENEDOR_COMUNAL"] == "0":
        lon, lat = pts[mz]
    else:
        w, x, y = acc[r["CUT"]]
        lon, lat = x / w, y / w
    out.writerow([mz, r["CUT"], f"{lon:.6f}", f"{lat:.6f}", n(r["n_per"]), n(r["n_vp"]),
                  n(r["n_ocupado"]), n(r["n_asistencia_basica"]) + n(r["n_asistencia_media"])])

# communes.csv: sums of the same block rows; point = polygon centroid of the comuna.
com = {}
for r in rows:
    c = com.setdefault(r["CUT"], [r["COMUNA"], 0, 0, 0])
    c[1] += n(r["n_per"]); c[2] += n(r["n_hog"]); c[3] += n(r["n_vp"])
out = csv.writer(open("data/mvp/communes.csv", "w", newline=""))
out.writerow(["cut", "name", "population", "households", "dwellings", "longitude", "latitude"])
feats = []
for cut, shape in db.execute("select CUT, SHAPE from Comunal_CPV24 order by CUT"):
    name, p, h, d = com[str(cut)]
    lon, lat = centroid(shape)
    out.writerow([cut, name, p, h, d, f"{lon:.6f}", f"{lat:.6f}"])
    polys, _ = rings(shape, 8 + ENV[(shape[3] >> 1) & 7])
    geo = []
    for poly in polys:
        g = []
        for r in poly:
            ring = []
            for i in range(0, len(r), 2):
                pt = [round(r[i], 4), round(r[i + 1], 4)]  # 4 decimals = about 10 m
                if not ring or pt != ring[-1]:
                    ring.append(pt)
            g.append(ring)
        geo.append(g)
    feats.append({"type": "Feature", "properties": {"cut": cut, "name": name, "population": p},
                  "geometry": {"type": "MultiPolygon", "coordinates": geo}})
json.dump({"type": "FeatureCollection", "features": feats}, open("data/mvp/communes.geojson", "w"), separators=(",", ":"))
