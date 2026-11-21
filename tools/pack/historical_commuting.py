"""Prepare RM intercommunal Census counts; no model, scoring or normalization.

Run: python3 -I tools/pack/historical_commuting.py
"""
import csv
import hashlib
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PACK = ROOT / "data/validation/census_2002"
CURRENT = ROOT / "data/mvp"


def rows(path):
    with path.open(newline="", encoding="utf-8") as source:
        return list(csv.DictReader(source))


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


cuts = sorted(row["cut"] for row in rows(CURRENT / "communes.csv"))
domain = [(origin, destination) for origin in cuts for destination in cuts if origin != destination]
classification = {row["destination_raw"]: row["classification"] for row in rows(PACK / "destination-classification.csv")}
raw_manifest = json.loads((PACK / "manifest.json").read_text())
coverage = {}
outputs = {}
for year, source, origin_key, destination_key in [
    ("2002", PACK / "raw_od.csv", "origin", "destination_raw"),
    ("2024", CURRENT / "work_od.csv", "origin_cut", "destination_cut"),
]:
    counts = Counter()
    categories = Counter()
    for row in rows(source):
        origin, destination, workers = row[origin_key], row[destination_key], int(row["workers"])
        counts[origin, destination] += workers
        category = classification[destination] if year == "2002" else (
            "internal_rm52" if destination in cuts else "known_external_domestic"
        )
        categories[category] += workers
    diagonal = sum(counts[cut, cut] for cut in cuts)
    included = sum(counts[pair] for pair in domain)
    output = PACK / f"intercommunal_{year}.csv"
    with output.open("w", newline="", encoding="utf-8") as target:
        writer = csv.writer(target)
        writer.writerow(["origin_cut", "destination_cut", "workers"])
        writer.writerows((origin, destination, counts[origin, destination]) for origin, destination in domain)
    outputs[str(output.relative_to(ROOT))] = sha256(output)
    coverage[year] = {
        "source_rows": len(counts),
        "source_workers": sum(counts.values()),
        "source_workers_by_destination_class": dict(sorted(categories.items())),
        "internal_rm_workers_including_diagonal": categories["internal_rm52"],
        "excluded_diagonal_workers": diagonal,
        "included_intercommunal_workers": included,
        "excluded_outside_internal_rm_workers": sum(counts.values()) - categories["internal_rm52"],
        "excluded_total_workers": sum(counts.values()) - included,
        "dense_output_cells": len(domain),
        "positive_output_cells": sum(counts[pair] > 0 for pair in domain),
    }

input_paths = [PACK / "raw_od.csv", PACK / "manifest.json", PACK / "destination-classification.csv",
               PACK / "old_current_cut_crosswalk.csv", CURRENT / "communes.csv", CURRENT / "work_od.csv"]
manifest = {
    "checked_date": raw_manifest["source"]["checked_date"],
    "source_retrieved_at_utc": None,
    "retrieval_time_note": "Exact source retrieval UTC was not recorded.",
    "generator": {"path": "tools/pack/historical_commuting.py", "sha256": sha256(Path(__file__))},
    "inputs_sha256": {str(path.relative_to(ROOT)): sha256(path) for path in input_paths},
    "outputs_sha256": outputs,
    "domain": {"cuts": cuts, "origin_count": len(cuts), "destination_count": len(cuts),
               "same_home_work_commune_excluded": True, "cells_per_year": len(domain)},
    "coverage": coverage,
    "source_frames": {
        "2002": "All enumeration geography; habitual RM home VIVEHAB; employed SLABORAL 1/2/4; raw COMTRAES classified by official workplace-answer aggregate.",
        "2024": "Canonical RM-resident work_od.csv; work-location codes 2/3 and known domestic destinations; home workers, multiple/foreign/unknown destinations already excluded upstream.",
    },
    "cohort_caveats": [
        "2002 cannot separate home workers from other same-commune workers. The diagonal is excluded in both years; full-matrix cohort equivalence is not established.",
        "Absent 2024 pairs are assigned zero only under the existing complete known-workplace count-aggregation assumption; they are not missing survey observations.",
        "The 52 RM codes are stable in the audited primary lists; this does not establish cadastral boundary equivalence.",
        "Crosswalk is derived from primary lists, not an official published concordance. Four external historical parents require child aggregation and cannot be allocated to child units.",
        "Foreign categories are classified from responses, not numeric prefixes; country names remain unmapped. All raw categories remain unchanged in raw_od.csv.",
        "Counts represent employed people at usual workplaces, not daily trips. INE assumes employed students reported work in 2002.",
        "Domestic external, foreign, unknown and diagonal workers are excluded, not redistributed. No global normalization is applied.",
    ],
    "classification_provenance": raw_manifest["classification"],
    "benchmark_implemented": False,
    "approved_evaluation_target": False,
}
(PACK / "comparison_manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
print(json.dumps(coverage, indent=2))
