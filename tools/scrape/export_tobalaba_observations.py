"""Export one of three reviewed Metro station crosswalks and observations."""
import argparse
import csv
import io
import json
import zipfile
from collections import defaultdict
from decimal import Decimal
from pathlib import Path
from export_april_boardings import file_record

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("gtfs", type=Path)
parser.add_argument("boardings_csv", type=Path)
parser.add_argument("out", type=Path)
parser.add_argument("--station", choices=["TOBALABA", "LOS LEONES", "EL GOLF"], default="TOBALABA")
args = parser.parse_args()
station_id = {"TOBALABA": "TB-TOB", "LOS LEONES": "LE-LEN", "EL GOLF": "GO"}[args.station]

with zipfile.ZipFile(args.gtfs) as archive:
    stops = {r["stop_id"]: r for r in csv.DictReader(
        io.TextIOWrapper(archive.open("stops.txt"), encoding="utf-8-sig"))}
station = stops[station_id]
platforms = [r for r in stops.values() if r["parent_station"] == station["stop_id"]
             and r["location_type"] == "0"]
observations = []
totals = defaultdict(Decimal)
with args.boardings_csv.open(newline="", encoding="utf-8") as source:
    for r in csv.DictReader(source):
        if r["mode"] == "Metro" and r["internal_stop"] == args.station:
            observations.append({"source_row": int(r["source_row"]), "day_type": r["day_type"],
                                 "half_hour": r["half_hour"], "boardings": float(r["boardings"])})
            totals[r["day_type"]] += Decimal(r["boardings"])

result = {
    "kind": "observed_boardings", "timezone": "America/Santiago",
    "period": {"LABORAL": "average of 2026-04-13 through 2026-04-17",
               "SABADO": "2026-04-11", "DOMINGO": "2026-04-12"},
    "source": {"boarding_workbook": "https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2026.04_v3.xlsx",
               "gtfs_url": "https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip",
               "gtfs": file_record(args.gtfs), "exported_boardings": file_record(args.boardings_csv)},
    "crosswalk": {"mode": "Metro", "internal_stop": args.station, "user_stop": "-",
                  "rule": f"Reviewed exact station name after case conversion: {args.station}; unique GTFS station location_type=1, stop_id={station_id}. Listed platforms share this parent. No line/platform split in DTPM observations.",
                  "gtfs_station_record": station, "gtfs_platform_records": platforms},
    "station": {"id": station["stop_id"], "name": station["stop_name"],
                "latitude": float(station["stop_lat"]), "longitude": float(station["stop_lon"])},
    "limits": ["Validation-derived boardings, excluding estimated fare evasion; not unique riders.",
               "Station total, not a platform, entrance, MUT visitor count, or forecast.",
               "Absent half-hours are missing observations, not zero demand.",
               "Location is the GTFS parent station; no allocation to lines, platforms or entrances."],
    "day_totals": {k: float(v) for k, v in sorted(totals.items())},
    "observations": observations,
    "half_hour_bins": {day: [next((r["boardings"] for r in observations
                                  if r["day_type"] == day and r["half_hour"] == f"{k // 2:02d}:{k % 2 * 30:02d}"), None)
                              for k in range(48)] for day in sorted(totals)},
}
args.out.parent.mkdir(parents=True, exist_ok=True)
args.out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"rows": len(observations), "totals": result["day_totals"], "output": str(args.out)}))
