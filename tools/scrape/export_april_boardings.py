"""Offline export of the verified April 2026 boarding workbook and March GTFS."""
import argparse
import csv
import hashlib
import io
import json
import posixpath
import zipfile
from collections import defaultdict
from decimal import Decimal
from pathlib import Path
from xml.etree import ElementTree as ET

NS = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
REL = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"
SOURCES = {
    "gtfs": "https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip",
    "boardings": "https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2026.04_v3.xlsx",
}


def sheet_rows(archive, sheet_name):
    """Read cached values by cell column; release each parsed row."""
    strings = ["".join(t.text or "" for t in item.iter(NS + "t"))
               for item in ET.fromstring(archive.read("xl/sharedStrings.xml"))]
    targets = {r.attrib["Id"]: r.attrib["Target"] for r in
               ET.fromstring(archive.read("xl/_rels/workbook.xml.rels"))}
    sheet = next(s for s in ET.fromstring(archive.read("xl/workbook.xml")).iter(NS + "sheet")
                 if s.attrib["name"] == sheet_name)
    target = targets[sheet.attrib[REL + "id"]]
    path = posixpath.normpath(posixpath.join("xl", target)).lstrip("/")
    with archive.open(path) as stream:
        for event, elem in ET.iterparse(stream, events=("start", "end")):
            if event == "start" and elem.tag == NS + "sheetData":
                data = elem
            if event == "end" and elem.tag == NS + "row":
                values = {}
                for cell in elem:
                    column = cell.attrib["r"].rstrip("0123456789")
                    value = cell.findtext(NS + "v", "")
                    if cell.attrib.get("t") == "s":
                        value = strings[int(value)]
                    values[column] = value
                yield int(elem.attrib["r"]), values
                data.remove(elem)


def file_record(path):
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"name": path.name, "bytes": path.stat().st_size, "sha256": digest}


def export(gtfs, boardings, out):
    out.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(gtfs) as archive:
        stops = {r["stop_id"]: r for r in csv.DictReader(
            io.TextIOWrapper(archive.open("stops.txt"), encoding="utf-8-sig"))}
    totals = defaultdict(lambda: [0, Decimal(0)])
    unmatched = defaultdict(lambda: [0, Decimal(0)])
    seen = set()
    duplicates = zero_rows = rows_read = 0
    columns = ["source_row", "day_type", "mode", "internal_stop", "user_stop", "comuna",
               "half_hour", "boardings", "gtfs_stop_id", "stop_lat", "stop_lon", "join_status"]
    with zipfile.ZipFile(boardings) as archive, (out / "boardings.csv").open(
            "w", newline="", encoding="utf-8") as output:
        writer = csv.writer(output)
        writer.writerow(columns)
        rows = sheet_rows(archive, "SUBIDAS_2026_04")
        next(rows)  # Source header, already verified for this workbook.
        for source_row, r in rows:
            day, mode, internal, user, comuna = (r[c] for c in "ABCDE")
            minute = round(Decimal(r["F"]) * 1440)
            clock = f"{minute // 60:02d}:{minute % 60:02d}"
            amount = Decimal(r["G"])
            key = (day, mode, internal, user, comuna, clock)
            duplicates += key in seen
            seen.add(key)
            stop = stops.get(user) if mode == "BUS" else None
            status = "matched" if stop is not None else (
                "non_bus_unresolved" if mode != "BUS" else "bus_user_code_not_in_gtfs")
            location = [stop["stop_id"], stop["stop_lat"], stop["stop_lon"]] if stop else ["", "", ""]
            writer.writerow([source_row, *key[:5], clock, r["G"], *location, status])
            totals[(day, mode, status)][0] += 1
            totals[(day, mode, status)][1] += amount
            if stop is None:
                unmatched[(day, mode, internal, user, comuna, status)][0] += 1
                unmatched[(day, mode, internal, user, comuna, status)][1] += amount
            rows_read += 1
            zero_rows += amount == 0
        summary_rows = dict(sheet_rows(archive, "RESUMEN"))
        cached = []
        for row in (3, 4, 5):
            for col in "BCD":
                day, mode = summary_rows[2][col], summary_rows[row]["A"]
                computed = sum((v[1] for k, v in totals.items() if k[:2] == (day, mode)), Decimal(0))
                published = Decimal(summary_rows[row][col])
                cached.append({"day_type": day, "mode": mode, "computed_boardings": str(computed),
                               "publisher_cached_boardings": str(published),
                               "difference": str(computed - published)})
    with (out / "unmatched_stops.csv").open("w", newline="", encoding="utf-8") as output:
        writer = csv.writer(output)
        writer.writerow(["day_type", "mode", "internal_stop", "user_stop", "comuna", "join_status",
                         "rows", "boardings"])
        for key, values in sorted(unmatched.items()):
            writer.writerow([*key, *values])
    manifest = {
        "sources": {name: {**file_record(path), "url": SOURCES[name]}
                    for name, path in (("gtfs", gtfs), ("boardings", boardings))},
        "period": {"LABORAL": "average of 2026-04-13 through 2026-04-17",
                   "SABADO": "2026-04-11", "DOMINGO": "2026-04-12"},
        "timezone": "America/Santiago", "unit": "validation-derived boardings; excludes estimated fare evasion",
        "join": "BUS Paradero Usuario = GTFS stop_id, exact; other modes unresolved",
        "gtfs_stop_count": len(stops), "rows_read": rows_read, "observed_zero_rows": zero_rows,
        "duplicate_source_keys": duplicates, "duplicate_policy": "preserve source rows; never sum duplicate keys implicitly",
        "missing_row_policy": "not zero demand", "unmatched_stop_day_keys": len(unmatched),
        "totals": [{"day_type": k[0], "mode": k[1], "join_status": k[2], "rows": v[0], "boardings": str(v[1])}
                   for k, v in sorted(totals.items())],
        "summary_comparison": cached,
        "outputs": {name: file_record(out / name) for name in ("boardings.csv", "unmatched_stops.csv")},
    }
    (out / "manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({"rows": rows_read, "duplicate_keys": duplicates, "output": str(out)}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("gtfs", type=Path)
    parser.add_argument("boardings", type=Path)
    parser.add_argument("out", type=Path)
    args = parser.parse_args()
    export(args.gtfs, args.boardings, args.out)
