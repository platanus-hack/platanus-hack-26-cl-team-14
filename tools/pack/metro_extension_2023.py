"""Metro boardings before and after the two extensions that opened in 2023.

Openings, both confirmed on primary government pages:
  Line 3 to Quilicura   Monday 2023-09-25  Ferrocarril, Lo Cruzat, Plaza de Quilicura
    https://mtt.gob.cl/extension-de-linea-3-de-metro-inicia-operacion-beneficiando-a-mas-271-mil-personas-de-quilicura/
  Line 2 to San Bernardo Monday 2023-11-27  El Bosque, Observatorio, Copa Lo Martinez, El Pino
    https://mtt.gob.cl/extension-de-linea-2-comenzo-su-operacion-y-beneficiara-a-618-mil-personas/

Source: DTPM "Subidas por Paradero y Estacion", average weekday (LABORAL) boardings
in 48 half hours, from https://www.dtpm.cl/index.php/documentos/matrices-de-viaje
  2023.04  weekdays 17 to 21 April      before both openings
  2023.08  weekdays 7 to 11 August      before both openings
  2023.11  weekdays 6 to 10 November    after Line 3, before Line 2

The November cut sits between the two openings, so it reads Line 3 six weeks after
its opening while it is still a clean "before" for Line 2, in one file and one method.

The April workbook carries an extra `segunda_parada` column, so every column index
comes from the header row and none is hardcoded. A station on two lines has one row
for each line; rows are kept by (line, station) and also summed over lines.

The files are .xlsb (Excel binary). This reads BIFF12 records with the standard
library only: variable-length record id and size, then the cell records.

Output: data/validation/metro_extension_2023/boardings.csv
        cut,days,line,station,comuna,total,b00..b47
Usage:  python3 -I tools/pack/metro_extension_2023.py
"""
import csv, pathlib, struct, zipfile

RAW = pathlib.Path("data/raw/dtpm-2023")
OUT = pathlib.Path("data/validation/metro_extension_2023")
CUTS = [("2023.04", "x-apr/Tablas de Subidas y Bajadas/2023.04 Matriz_sub_SS_MH.xlsb"),
        ("2023.08", "x-ago/Tablas de subidas y bajadas/2023.08 Matriz_sub_SS_MH.xlsb"),
        ("2023.11", "x-nov/2023.11 Matriz_sub_SS_MH.xlsb")]


def records(buf):
    i = 0
    while i < len(buf):
        b = buf[i]; i += 1
        rid = b & 0x7F
        if b & 0x80:
            rid |= (buf[i] & 0x7F) << 7; i += 1
        size = shift = 0
        while True:
            b = buf[i]; i += 1
            size |= (b & 0x7F) << shift; shift += 7
            if not b & 0x80: break
        yield rid, buf[i:i + size]; i += size


def wide(payload, offset):
    (count,) = struct.unpack_from("<I", payload, offset)
    return payload[offset + 4:offset + 4 + 2 * count].decode("utf-16-le", "replace")


def number(value):
    if value & 2:
        out = float(value >> 2 if not value & 0x80000000 else (value >> 2) - (1 << 30))
    else:
        out = struct.unpack("<d", struct.pack("<Q", (value & 0xFFFFFFFC) << 32))[0]
    return out / 100 if value & 1 else out


def rows(path):
    book = zipfile.ZipFile(path)
    shared = [wide(p, 1) for rid, p in records(book.read("xl/sharedStrings.bin")) if rid == 19]
    row, cells = -1, {}
    for rid, p in records(book.read("xl/worksheets/sheet2.bin")):
        if rid == 0:
            if row >= 0: yield cells
            (row,) = struct.unpack_from("<I", p, 0); cells = {}
        elif rid in (2, 4, 5, 6, 7, 9):
            (col,) = struct.unpack_from("<I", p, 0)
            if rid == 2: cells[col] = number(struct.unpack_from("<I", p, 8)[0])
            elif rid in (5, 9): cells[col] = struct.unpack_from("<d", p, 8)[0]
            elif rid == 7:
                (k,) = struct.unpack_from("<I", p, 8)
                cells[col] = shared[k]
            elif rid == 6: cells[col] = wide(p, 8)
            elif rid == 4: cells[col] = bool(p[8])
    if row >= 0: yield cells


OUT.mkdir(parents=True, exist_ok=True)
out, totals = [], {}
for cut, member in CUTS:
    head, days, kept = None, "", 0
    for cells in rows(RAW / member):
        text = [v for v in cells.values() if isinstance(v, str)]
        if head is None:
            if any(v.startswith("Fuente") for v in text):
                days = text[0].split("(")[-1].rstrip(")")
            if "TOTAL" in text:
                head = {str(v).strip(): k for k, v in cells.items() if isinstance(v, str)}
            continue
        line, station = cells.get(head["servicio Sonda"]), cells.get(head["parada SIMT"])
        if not (isinstance(line, str) and line.startswith("L") and isinstance(station, str)):
            continue
        first = head["TOTAL"] + 1
        bins = [cells.get(first + b, 0.0) for b in range(48)]
        out.append([cut, days, line, station.strip().upper(), cells.get(head["Comuna"], ""),
                    round(cells[head["TOTAL"]], 1)] + [round(v, 1) for v in bins])
        kept += 1
    totals[cut] = (kept, days, sum(r[5] for r in out if r[0] == cut))
    print(f"{cut}  {days}  rows {kept}  network boardings {totals[cut][2]:,.1f}")

with (OUT / "boardings.csv").open("w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["cut", "days", "line", "station", "comuna", "total"] + [f"b{b:02d}" for b in range(48)])
    w.writerows(out)
print(OUT / "boardings.csv", len(out), "rows")
