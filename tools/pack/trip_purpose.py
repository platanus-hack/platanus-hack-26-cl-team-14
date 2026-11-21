"""Trips of Wednesday 2026-04-15 by destination stop, purpose and half hour (DTPM trip table).

Run from the repo root:
  python3 -I tools/pack/trip_purpose.py fetch   # one member of the ZIP, by HTTP range (294 MB, into data/raw/)
  python3 -I tools/pack/trip_purpose.py         # the aggregates

Outputs, aggregates only (no card code, no trip row):
  data/raw/dtpm-viajes-2026-04/by_destination.csv            all stops, 12 MB, not in git
  data/validation/trip_purpose_2026_04/metro_arrivals.csv    the 126 Metro stations
`proposito` is estimated by DTPM. It is not a survey answer.
"""
import collections, csv, gzip, io, json, struct, sys, unicodedata, urllib.request, zipfile

URL = "https://www.dtpm.cl/descargas/modelos_y_matrices/Viajes_2026-04.zip"
RAW = "data/raw/dtpm-viajes-2026-04/"
GZ = RAW + "2026-04-15.viajes.csv.gz"


def fetch():
    def rng(a, b):
        r = urllib.request.Request(URL, headers={"Range": "bytes=%d-%d" % (a, b), "User-Agent": "curl/8"})
        return urllib.request.urlopen(r, timeout=120).read()
    size = int(urllib.request.urlopen(urllib.request.Request(URL, method="HEAD", headers={"User-Agent": "curl/8"})).headers["Content-Length"])

    class Remote:  # the minimum that zipfile needs to read the central directory
        pos = 0
        def seekable(s): return True
        def tell(s): return s.pos
        def seek(s, o, w=0):
            s.pos = o if w == 0 else s.pos + o if w == 1 else size + o
            return s.pos
        def read(s, n=-1):
            n = size - s.pos if n < 0 else n
            if n <= 0 or s.pos >= size: return b""
            d = rng(s.pos, min(size, s.pos + n) - 1)
            s.pos += len(d)
            return d
    m = [i for i in zipfile.ZipFile(Remote()).infolist() if "2026-04-15" in i.filename][0]  # stored, not deflated
    nl, el = struct.unpack("<HH", rng(m.header_offset + 26, m.header_offset + 29))
    a, end = m.header_offset + 30 + nl + el, m.header_offset + 30 + nl + el + m.compress_size - 1
    with open(GZ, "wb") as f:
        while a <= end:
            b = min(end, a + (32 << 20) - 1)
            f.write(rng(a, b))
            a = b + 1


if sys.argv[1:] == ["fetch"]:
    fetch()
    sys.exit()

norm = lambda s: " ".join(unicodedata.normalize("NFD", s.upper()).encode("ascii", "ignore").decode().split())
metro = {norm(s["name"]): s["id"] for s in json.load(open("data/mvp/metro_training.json"))}
agg, rows = collections.Counter(), collections.Counter()
f = io.TextIOWrapper(gzip.open(GZ), encoding="utf-8", errors="replace")
ix = {c: i for i, c in enumerate(f.readline().rstrip("\n").split("|"))}
iw, idst, ip, ih = ix["factor_expansion"], ix["paradero_fin_viaje"], ix["proposito"], ix["mediahora_fin_viaje"]
for line in f:
    r = line.split("|")
    k = r[idst], r[ip], r[ih]
    agg[k] += float(r[iw])
    rows[k] += 1
with open(RAW + "by_destination.csv", "w", newline="") as o:
    w = csv.writer(o)
    w.writerow(["destination_stop", "proposito", "half_hour_end", "expanded_trips", "rows"])
    w.writerows([*k, round(agg[k], 4), rows[k]] for k in sorted(agg))
with open("data/validation/trip_purpose_2026_04/metro_arrivals.csv", "w", newline="") as o:
    w = csv.writer(o)
    w.writerow(["station_id", "station_name", "proposito", "half_hour_end", "expanded_trips", "rows"])
    w.writerows([metro[norm(k[0])], k[0], k[1], k[2], round(agg[k], 4), rows[k]] for k in sorted(agg) if norm(k[0]) in metro)
print(sum(rows.values()), "rows,", round(sum(agg.values())), "expanded trips,", len(agg), "cells")
