"""Export all April Metro observations with exact names and six reviewed aliases."""
import argparse
import csv
import io
import json
import unicodedata
import zipfile
from collections import defaultdict
from decimal import Decimal
from pathlib import Path
from export_april_boardings import file_record

DAYS = {
    "LABORAL": "average of 2026-04-13 through 2026-04-17",
    "SABADO": "2026-04-11",
    "DOMINGO": "2026-04-12",
}

ALIASES = {
    "CAL Y CANTO": ("CA-PCA", "Puente prefix omitted; one L2/L3 parent"),
    "PARQUE OHIGGINS": ("PQ", "Apostrophe and spacing differ"),
    "PDTE PEDRO AGUIRRE CERDA": ("PAC", "Presidente abbreviated; platform names also use Pdte."),
    "PLAZA MAIPU": ("PM", "de omitted; accent differs"),
    "RONDIZONNI": ("RO", "Explicit source spelling discrepancy: GTFS Rondizzoni"),
    "UNION LATINO AMERICANA": ("LA", "Latinoamericana spacing and accent differ"),
}
ALIAS_REVIEW = "https://discord.com/channels/1573514576400752640/1573579088684253235/1573591251574587474"


def normalized(name):
    return " ".join("".join(c for c in unicodedata.normalize("NFKD", name.upper())
                            if not unicodedata.combining(c)).split())


def export(gtfs, boardings, out):
    parents = defaultdict(list)
    parent_ids = {}
    with zipfile.ZipFile(gtfs) as archive:
        for row in csv.DictReader(io.TextIOWrapper(archive.open("stops.txt"), encoding="utf-8-sig")):
            if row["location_type"] == "1":
                parents[normalized(row["stop_name"])].append(row)
                parent_ids[row["stop_id"]] = row
    records = {}
    totals = defaultdict(Decimal)
    status_totals = defaultdict(Decimal)
    rows_read = 0
    with boardings.open(newline="", encoding="utf-8") as source:
        for row in csv.DictReader(source):
            if row["mode"] != "Metro":
                continue
            rows_read += 1
            name, day = row["internal_stop"], row["day_type"]
            if name not in records:
                candidates = parents[normalized(name)]
                station = candidates[0] if len(candidates) == 1 else None
                status = "unique_normalized_name" if station else "unresolved"
                alias_reason = None
                if name in ALIASES:
                    station_id, alias_reason = ALIASES[name]
                    station = parent_ids[station_id]
                    status = "reviewed_alias"
                records[name] = {
                    "source_name": name, "source_comunas": [],
                    "match_status": status, "alias_reason": alias_reason,
                    "station": {"id": station["stop_id"], "name": station["stop_name"],
                                "longitude": float(station["stop_lon"]),
                                "latitude": float(station["stop_lat"])} if station else None,
                    "candidate_parent_ids": [s["stop_id"] for s in candidates],
                    "half_hour_bins": {d: [None] * 48 for d in DAYS},
                    "source_rows": {d: [None] * 48 for d in DAYS},
                }
            item = records[name]
            if row["comuna"] not in item["source_comunas"]:
                item["source_comunas"].append(row["comuna"])
            h, m = map(int, row["half_hour"].split(":"))
            slot = h * 2 + m // 30
            if item["source_rows"][day][slot] is not None:
                raise ValueError(f"Duplicate source station/day/bin: {name}, {day}, {slot}")
            amount = Decimal(row["boardings"])
            item["half_hour_bins"][day][slot] = float(amount)
            item["source_rows"][day][slot] = int(row["source_row"])
            totals[day] += amount
            status_totals[(item["match_status"], day)] += amount
    stations = [records[n] for n in sorted(records)]
    result = {
        "kind": "observed_metro_boardings", "timezone": "America/Santiago", "period": DAYS,
        "unit": "validation-derived boardings; excludes estimated fare evasion",
        "matching": "Unique location_type=1 name after uppercase, Unicode accent removal and whitespace collapse, or one of six explicit peer-reviewed aliases. No fuzzy matching.",
        "alias_review": ALIAS_REVIEW,
        "bin_definition": "Index k is local k*30 minutes; null means no source observation, not zero.",
        "score_ready": False,
        "score_limits": [
            "No held-out split or scoring formula is frozen in this artifact.",
            "Targets used for input profiles, fitting or observed-plus-delta reconstruction are not independent validation.",
            "TB-TOB, LE-LEN and GO have already been used in the demo; do not call them held-out.",
            "Only matched stations can enter a spatial score; unresolved names remain preserved here.",
            "Station totals do not identify a line, platform, entrance, unique rider or train load.",
        ],
        "stations": stations,
    }
    out.mkdir(parents=True, exist_ok=True)
    target = out / "metro_observations.json"
    target.write_text(json.dumps(result, ensure_ascii=False, separators=(",", ":")) + "\n", encoding="utf-8")
    audit = {
        "sources": {"gtfs": file_record(gtfs), "exported_boardings": file_record(boardings)},
        "upstream_command": "python3 tools/scrape/export_april_boardings.py GTFS_ZIP BOARDING_XLSX OUTPUT_DIR",
        "source_rows": rows_read, "source_station_names": len(stations),
        "matched_station_names": sum(s["station"] is not None for s in stations),
        "unique_parent_ids": len({s["station"]["id"] for s in stations if s["station"]}),
        "matching_counts": {status: sum(s["match_status"] == status for s in stations)
                            for status in ["unique_normalized_name", "reviewed_alias", "unresolved"]},
        "alias_review": ALIAS_REVIEW,
        "unresolved_names": [s["source_name"] for s in stations if s["station"] is None],
        "totals": {day: str(totals[day]) for day in DAYS},
        "totals_by_match_status": [{"match_status": status, "day_type": day, "boardings": str(value)}
                                   for (status, day), value in sorted(status_totals.items())],
        "observed_bins": rows_read, "missing_bins": len(stations) * len(DAYS) * 48 - rows_read,
        "output": file_record(target),
    }
    (out / "metro_observations_manifest.json").write_text(json.dumps(audit, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(audit, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("gtfs", type=Path)
    parser.add_argument("boardings_csv", type=Path)
    parser.add_argument("out", type=Path)
    args = parser.parse_args()
    export(args.gtfs, args.boardings_csv, args.out)
