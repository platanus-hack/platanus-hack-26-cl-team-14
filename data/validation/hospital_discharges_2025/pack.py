"""Aggregate reported public hospital discharges by destination comuna and month."""

import csv
import sys
from collections import defaultdict
from pathlib import Path

YEAR = "2025"
PACK = Path(__file__).resolve().parent


def main(rem_path, registry_path, output_path):
    with (PACK / "../../mvp/communes.csv").open(newline="") as communes:
        model_cuts = {row["cut"] for row in csv.DictReader(communes)}

    with open(registry_path, encoding="utf-8-sig", newline="") as registry:
        cuts_by_site = {}
        for row in csv.DictReader(registry, delimiter=";"):
            site = row["EstablecimientoCodigo"].strip()
            cut = row["ComunaCodigo"].strip()
            if site and cut:
                previous = cuts_by_site.setdefault(site, cut)
                if previous != cut:
                    raise ValueError(f"Conflicting comuna codes for DEIS site {site}")

    sites = defaultdict(set)
    counts = defaultdict(int)
    missing_counts = 0
    unmatched = set()
    areas = set()
    seen = set()
    with open(rem_path, encoding="utf-8-sig", newline="") as rem:
        for row in csv.DictReader(rem, delimiter=";"):
            if row["PERIODO"].strip() != YEAR or row["TIPO_PERTENENCIA"].strip() != "1":
                continue
            site = row["CODIGO_ESTABLECIMIENTO"].strip()
            cut = cuts_by_site.get(site)
            if cut is None:
                unmatched.add(site)
            if cut not in model_cuts:
                continue

            month = int(row["MES"])
            area = int(row["COD_AREA_FUNCIONAL"])
            site_key = site, month, area
            if site_key in seen:
                raise ValueError(f"Repeated establishment, month and area: {site_key}")
            seen.add(site_key)

            raw_count = row["NUMERO_EGRESOS"].strip()
            if not raw_count:
                missing_counts += 1
                continue
            count = int(raw_count)
            if not 1 <= month <= 12 or count < 0:
                raise ValueError(f"Invalid month or discharge count at {site_key}")
            key = cut, month
            counts[key] += count
            sites[key].add(site)
            areas.add(area)

    if not counts:
        raise ValueError(f"No reported discharges for model comunas in {YEAR}")
    with open(output_path, "w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output, lineterminator="\n")
        writer.writerow(("cut", "month", "discharges", "reporting_establishments"))
        for cut, month in sorted(counts):
            key = cut, month
            writer.writerow((cut, f"{YEAR}-{month:02d}", counts[key], len(sites[key])))

    print(f"{len(counts)} comuna-months, {sum(counts.values())} discharges")
    print(f"{missing_counts} blank counts in model comunas")
    print(f"Functional area codes in output: {sorted(areas)}")
    print(f"Unmatched public establishment codes in Chile: {sorted(unmatched)}")


if __name__ == "__main__":
    main(*sys.argv[1:])
