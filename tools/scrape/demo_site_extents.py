"""Fetch public OSM relations with AX; measure PR #409's 500 m capture box.

Run from any directory. Add --saved to reuse the committed raw responses.
Only the Python standard library and the ax executable are required.
"""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "data/validation/demo_site_extents"
CAPTURE_COMMIT = "85b9def9fbad92acf9ffe00762b2b415258b6950"
SITES = [("mut", 18760408, -70.6011321, -33.4172801),
         ("monumental", 14111047, -70.6059663, -33.5065368)]
HALF = 250.0


def ecef(lon, lat):
    """PR ecef() at height zero: OSM nodes have no ellipsoid height."""
    lon, lat = math.radians(lon), math.radians(lat)
    sl, cl = math.sin(lat), math.cos(lat)
    n = 6378137.0 / math.sqrt(1.0 - 0.0066943799901413165 * sl * sl)
    return (n * cl * math.cos(lon), n * cl * math.sin(lon),
            n * (1.0 - 0.0066943799901413165) * sl)


def local(lon, lat, center_lon, center_lat):
    """PR earth_to_local(): return east and south components in metres.

    Anchor height changes only the up component, so it cancels here.
    """
    lo, la = math.radians(center_lon), math.radians(center_lat)
    delta = [a - b for a, b in zip(ecef(lon, lat), ecef(center_lon, center_lat))]
    east = (-math.sin(lo), math.cos(lo), 0.0)
    south = (math.sin(la) * math.cos(lo), math.sin(la) * math.sin(lo), -math.cos(la))
    return tuple(sum(a * b for a, b in zip(delta, axis)) for axis in (east, south))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--saved", action="store_true", help="use saved full JSON responses")
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    rows = []
    for site, relation_id, lon, lat in SITES:
        url = f"https://api.openstreetmap.org/api/0.6/relation/{relation_id}/full.json"
        raw_path = OUT / f"{relation_id}.json"
        if not args.saved:
            result = subprocess.run(["ax", url, "--body", "-f"], check=True, capture_output=True)
            json.loads(result.stdout)
            raw_path.write_bytes(result.stdout)
        raw = raw_path.read_bytes()
        data = json.loads(raw)
        elements = {(e["type"], e["id"]): e for e in data["elements"]}
        relation = elements[("relation", relation_id)]
        outer = [m for m in relation["members"] if m["role"] == "outer"]
        inner = [m for m in relation["members"] if m["role"] == "inner"]
        node_ids = {n for m in outer for n in elements[("way", m["ref"])]["nodes"]}
        nodes = [elements[("node", n)] for n in node_ids]
        points = [local(n["lon"], n["lat"], lon, lat) for n in nodes]
        xs, zs = zip(*points)
        rows.append(dict(
            site=site, relation_id=relation_id, relation_version=relation["version"],
            relation_timestamp=relation["timestamp"],
            outer_way_count=len(outer), inner_way_count=len(inner), outer_unique_nodes=len(nodes),
            center_lon=lon, center_lat=lat, half_extent_m=HALF,
            min_lon=min(n["lon"] for n in nodes), max_lon=max(n["lon"] for n in nodes),
            min_lat=min(n["lat"] for n in nodes), max_lat=max(n["lat"] for n in nodes),
            west_clearance_m=round(HALF + min(xs), 3), east_clearance_m=round(HALF - max(xs), 3),
            north_clearance_m=round(HALF + min(zs), 3), south_clearance_m=round(HALF - max(zs), 3),
            source_url=url, raw_file=raw_path.name, raw_sha256=hashlib.sha256(raw).hexdigest(),
            capture_commit=CAPTURE_COMMIT, attribution="© OpenStreetMap contributors",
            license="ODbL-1.0", license_url="https://www.openstreetmap.org/copyright"))
    with (OUT / "clearances.csv").open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=rows[0], lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    for row in rows:
        print(row["site"], {k: v for k, v in row.items() if "clearance" in k})


if __name__ == "__main__":
    main()
