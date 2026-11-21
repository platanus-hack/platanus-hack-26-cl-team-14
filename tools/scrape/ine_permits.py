#!/usr/bin/env python3
"""Pull Santiago (RM) building-permit points from INE's official georeferenced
permit layer (PE_NACIONAL, 2010-2025). Standard library only.

One row = one permit point with units (CANTIDAD_UNIDAD), floors (NUM_PISOS),
m2 (SUPERFICIE), and use (USO_DESTINO). Output: data/raw/ine_permits/rm_permits.json.

Run: python3 -I tools/scrape/ine_permits.py
"""
import json
import time
import urllib.parse
import urllib.request
from pathlib import Path

# INE official org (gisine1). Layer 1 = permits; layer 2 = recepción final.
BASE = ("https://services5.arcgis.com/hUyD8u3TeZLKPe4T/arcgis/rest/services/"
        "CRF_PE_v2/FeatureServer/1/query")
OUT = Path("data/raw/ine_permits")


def q(params):
    url = BASE + "?" + urllib.parse.urlencode(params)
    with urllib.request.urlopen(url, timeout=60) as r:
        return json.load(r)


def fetch_rm():
    """Page through RM (COD_REGION=13) permits with geometry."""
    OUT.mkdir(parents=True, exist_ok=True)
    out, offset = [], 0
    while True:
        d = q({
            "where": "COD_REGION=13",
            "outFields": ("GLOSA_COMUNA,AÑO,CANTIDAD_UNIDAD,USO_DESTINO,"
                          "GLOSA_DESTINO,NUM_PISOS,SUPERFICIE"),
            "returnGeometry": "true",
            "outSR": "4326",
            "resultOffset": offset,
            "resultRecordCount": 2000,
            "f": "json",
        })
        feats = d.get("features", [])
        out.extend(feats)
        if len(feats) < 2000:
            break
        offset += 2000
        time.sleep(1)  # 1 request per second
    (OUT / "rm_permits.json").write_text(json.dumps(out, ensure_ascii=False))
    return out


if __name__ == "__main__":
    rows = fetch_rm()
    # Count tall-building units. HABITACIONAL is a clean dwelling count; MIXTO
    # units also include commercial/office, so report the two apart.
    tall = [f["attributes"] for f in rows
            if (f["attributes"].get("NUM_PISOS") or 0) >= 15]
    hab = [t["CANTIDAD_UNIDAD"] for t in tall
           if t.get("USO_DESTINO") == "HABITACIONAL" and t.get("CANTIDAD_UNIDAD")]
    mix = [t["CANTIDAD_UNIDAD"] for t in tall
           if t.get("USO_DESTINO") == "MIXTO" and t.get("CANTIDAD_UNIDAD")]
    print(f"RM permit points: {len(rows)}")
    print(f"tall (>=15 floors) HABITACIONAL: {len(hab)} permits, "
          f"{sum(hab)} dwellings")
    print(f"tall (>=15 floors) MIXTO: {len(mix)} permits, "
          f"{sum(mix)} units (incl. commercial)")
