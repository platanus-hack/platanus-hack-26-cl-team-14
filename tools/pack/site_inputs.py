"""Comuna inputs for the site model of context/research/tobalaba-scenario-contract.md.

The contract needs three values that it has no number for: the occupied share of
dwellings, persons for each occupied dwelling, and known commuters for each resident.
This builder measures all three from the Census 2024, by comuna of the RM.

Stage 1 (`pass`, about 5 minutes): one full pass over the person microdata. It counts
persons, occupied dwellings (a change of `id_vivienda`; the rows of one dwelling are
next to each other) and commuters (`p44_lug_trab` 2 or 3 with a known comuna), by
comuna and by `tipo_operativo`. It writes the tallies to
data/raw/censo2024-personas/person_tallies.csv so stage 2 can run again in seconds.

Stage 2 (default): joins the tallies with the INE block table, which publishes
`n_vp` (private dwellings), `n_vp_ocupada` and `n_vp_desocupada` by block, and with
the apartment columns. Writes data/mvp/site_inputs.csv.

Stage `metro` (2 seconds, independent): the Metro share of the DTPM boardings at the
stops within 1 km of the site. The contract needs a Metro share inside public transport
and the Census cannot give one, because its code 2 joins metro, bus, train, taxi and
colectivo. This is the nearest measured local proxy, and it is April 2026, not EOD 2012.

Usage: python3 -I tools/pack/site_inputs.py pass    # stage 1, then stage 2
       python3 -I tools/pack/site_inputs.py         # stage 2 only
       python3 -I tools/pack/site_inputs.py metro   # the Metro share near the site
"""
import collections, csv, io, math, pathlib, re, subprocess, sys, zipfile

PERSONS = pathlib.Path("data/raw/censo2024-personas/personas_censo2024.zip")
TALLIES = pathlib.Path("data/raw/censo2024-personas/person_tallies.csv")
BLOCKS = pathlib.Path("data/raw/verify-people-work/census-blocks.zip")
OUT = pathlib.Path("data/mvp/site_inputs.csv")
MISSING = {"", "-99", "998", "999"}
I_VIV, I_COM, I_UMBRAL, I_TIPO, I_TRAB, I_DEST = 0, 5, 6, 8, 51, 52


def run_pass():
    rows = collections.defaultdict(collections.Counter)
    proc = subprocess.Popen(["unzip", "-p", str(PERSONS), "personas_censo2024.csv"],
                            stdout=subprocess.PIPE)
    stream = io.TextIOWrapper(proc.stdout, encoding="utf-8", errors="replace")
    next(stream)
    prev_viv = None
    for line in stream:
        f = line.rstrip("\n").split(";")
        comuna = f[I_COM]
        viv = f[I_VIV]
        if not comuna.startswith("13"):
            prev_viv = viv
            continue
        key = (f"{int(comuna):05d}", f[I_TIPO], f[I_UMBRAL])
        c = rows[key]
        c["persons"] += 1
        if viv != prev_viv:
            c["occupied_dwellings"] += 1
            prev_viv = viv
        if f[I_TRAB] in ("2", "3"):
            dest = f[I_DEST]
            if dest not in MISSING and not dest.zfill(5).endswith("000"):
                c["commuters_all"] += 1
                if dest.zfill(5).startswith("13"):
                    c["commuters_rm"] += 1
    cols = ["persons", "occupied_dwellings", "commuters_all", "commuters_rm"]
    with TALLIES.open("w", newline="", encoding="utf-8") as fh:
        w = csv.writer(fh)
        w.writerow(["cut", "tipo_operativo", "comuna_bajo_umbral"] + cols)
        for key in sorted(rows):
            w.writerow(list(key) + [rows[key][c] for c in cols])
    print(TALLIES, len(rows), "rows")


def block_totals():
    """Comuna sums of the INE block table. No value is suppressed in these columns."""
    z = zipfile.ZipFile(BLOCKS)
    fh = io.TextIOWrapper(z.open("Base_manzana_entidad_CPV24.csv"), encoding="utf-8-sig")
    keys = ["n_per", "n_vp", "n_vp_ocupada", "n_vp_desocupada", "n_tipo_viv_depto", "n_hog"]
    totals = collections.defaultdict(collections.Counter)
    flats = collections.defaultdict(collections.Counter)
    names = {}
    for row in csv.DictReader(fh, delimiter=";"):
        if row["COD_REGION"] != "13":
            continue
        cut = f'{int(row["CUT"]):05d}'
        names[cut] = row["COMUNA"]
        for k in keys:
            totals[cut][k] += int(row[k])
        # Blocks where almost every private dwelling is an apartment: the nearest
        # measured proxy for a new tower.
        vp = int(row["n_vp"])
        if vp >= 10 and int(row["n_tipo_viv_depto"]) >= 0.9 * vp:
            flats[cut]["persons"] += int(row["n_per"])
            flats[cut]["occupied"] += int(row["n_vp_ocupada"])
            flats[cut]["blocks"] += 1
    return totals, flats, names


def build():
    tallies = collections.defaultdict(collections.Counter)
    universe = collections.Counter()
    for row in csv.DictReader(TALLIES.open(encoding="utf-8")):
        for k in ("persons", "occupied_dwellings", "commuters_all", "commuters_rm"):
            universe[row["tipo_operativo"]] += int(row[k]) if k == "persons" else 0
            if row["tipo_operativo"] == PRIVATE:
                tallies[row["cut"]][k] += int(row[k])
    totals, flats, names = block_totals()
    with OUT.open("w", newline="", encoding="utf-8") as fh:
        w = csv.writer(fh)
        w.writerow(["cut", "name", "dwellings", "dwellings_occupied", "occupied_share",
                    "persons", "persons_per_occupied_dwelling",
                    "persons_per_dwelling_with_residents_present",
                    "commuters_rm", "commuters_all",
                    "commuters_rm_per_resident", "commuters_all_per_resident",
                    "apartment_share_of_dwellings",
                    "persons_per_occupied_dwelling_apartment_blocks",
                    "apartment_blocks", "apartment_block_persons",
                    "dwellings_with_residents_present", "persons_all_dwelling_kinds"])
        for cut in sorted(totals):
            t, p, fl = totals[cut], tallies[cut], flats[cut]
            persons = p["persons"]
            w.writerow([cut, names[cut], t["n_vp"], t["n_vp_ocupada"],
                        round(t["n_vp_ocupada"] / t["n_vp"], 4), persons,
                        round(persons / t["n_vp_ocupada"], 4),
                        round(persons / p["occupied_dwellings"], 4),
                        p["commuters_rm"], p["commuters_all"],
                        round(p["commuters_rm"] / persons, 4),
                        round(p["commuters_all"] / persons, 4),
                        round(t["n_tipo_viv_depto"] / t["n_vp"], 4),
                        round(fl["persons"] / fl["occupied"], 4) if fl["occupied"] else "",
                        fl["blocks"], fl["persons"], p["occupied_dwellings"], t["n_per"]])
    print(OUT, len(totals), "rows")
    print("persons by tipo_operativo:", dict(universe))


BOARDINGS = pathlib.Path("data/raw/dtpm/subidas_2026_04.xlsx")
GTFS = pathlib.Path("data/raw/verify-transport-metro/gtfs_20261114.zip")
SITE_LON, SITE_LAT = -70.600974, -33.417357  # centroid of manzana 13114131009033, the MUT
RADIUS_M = 1000
# The three reviewed DTPM-to-GTFS Metro joins (PR #23 and PR #29). The workbook has no
# user code for Metro, so an unreviewed station must not enter by name.
METRO_JOIN = {"TOBALABA": "TB-TOB", "LOS LEONES": "LE-LEN", "EL GOLF": "GO"}
CELL = re.compile(r'<c r="([A-G])\d+"([^>]*)>(?:<v>([^<]*)</v>)?')


def metro_share():
    def metres(lon, lat):
        return math.hypot((lon - SITE_LON) * math.cos(math.radians(SITE_LAT)) * 111320,
                          (lat - SITE_LAT) * 110574)

    bus, inside, outside = {}, [], []
    g = zipfile.ZipFile(GTFS)
    for r in csv.DictReader(io.TextIOWrapper(g.open("stops.txt"), encoding="utf-8-sig")):
        if not r["stop_lat"]:
            continue
        d = metres(float(r["stop_lon"]), float(r["stop_lat"]))
        if r["location_type"] == "1" and d <= 2 * RADIUS_M:
            (inside if d <= RADIUS_M else outside).append((r["stop_id"], r["stop_name"], round(d)))
        if r["location_type"] == "" and d <= RADIUS_M:
            bus[r["stop_id"]] = round(d)
    stations = {s[0]: s for s in inside}
    print("Metro parents within 1 km:", list(stations.values()))
    print("Metro parents 1 to 2 km, excluded:", outside)
    print("bus stops within 1 km:", len(bus))

    z = zipfile.ZipFile(BOARDINGS)
    shared = re.findall(r"<si>(?:<t[^>]*>)?(.*?)(?:</t>)?</si>",
                        z.read("xl/sharedStrings.xml").decode("utf-8"), re.S)
    shared = [re.sub(r"<.*?>", "", s) for s in shared]
    per_stop, matched = collections.Counter(), set()
    stream = io.TextIOWrapper(z.open("xl/worksheets/sheet3.xml"), encoding="utf-8")
    buffer = ""
    for chunk in iter(lambda: stream.read(1 << 22), ""):
        buffer += chunk
        *done, buffer = buffer.split("</row>")
        for row in done:
            cells = {}
            for col, attrs, value in CELL.findall(row):
                if value:
                    cells[col] = shared[int(value)] if 't="s"' in attrs else value
            if cells.get("A") != "LABORAL":
                continue
            boardings = float(cells.get("G", 0))
            if cells.get("B") == "Metro":
                station = METRO_JOIN.get(cells.get("C", "").upper())
                if station in stations:
                    per_stop[station] += boardings
            elif cells.get("B") == "BUS" and cells.get("D") in bus:
                per_stop[cells["D"]] += boardings
                matched.add(cells["D"])
    metro = sum(per_stop[s] for s in stations)
    total = sum(per_stop.values())
    print("bus stops with a weekday record:", len(matched), "of", len(bus))
    print("by Metro station:", {s: round(per_stop[s], 1) for s in stations})
    print(f"metro {metro:,.1f} / total {total:,.1f} = Metro share {metro / total:.4f}")
    away = metro - per_stop["LE-LEN"]
    print(f"without Los Leones (814 m, Providencia): {away:,.1f} / {away + total - metro:,.1f}"
          f" = {away / (away + total - metro):.4f}")


PRIVATE = "2"  # 1 street operative, 2 private dwelling, 3 collective dwelling
if __name__ == "__main__":
    if "metro" in sys.argv:
        metro_share()
    else:
        if "pass" in sys.argv:
            run_pass()
        build()
