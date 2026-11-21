#!/usr/bin/env python3
"""Pull Santiago (RM) building-permit points from the Observatorio de Ciudades UC
mirror of INE georeferenced permits (2010-2018). Standard library only.

One row = one permit point with dwellings (CANTIDAD_U), floors (NUM_PISOS),
m2 (SUPERFICIE), and use (USO_DESTIN). Output: data/raw/ine_permits/rm_permits.json.

Run: python3 -I tools/scrape/ine_permits.py
"""
import json
import time
import urllib.parse
import urllib.request
from pathlib import Path

BASE = ("https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/"
        "Permisos_edificacion_2010_2018/FeatureServer/0/query")
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
            "outFields": "GLOSA_COMU,AÑO,CANTIDAD_U,USO_DESTIN,GLOSA_DEST,NUM_PISOS,SUPERFICIE",
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
    towers = [f["attributes"] for f in rows
              if (f["attributes"].get("USO_DESTIN") == "HABITACIONAL"
                  and (f["attributes"].get("NUM_PISOS") or 0) >= 15)]
    units = [t["CANTIDAD_U"] for t in towers if t.get("CANTIDAD_U")]
    assert rows, "no rows returned"
    print(f"RM permit points: {len(rows)}")
    if units:
        units.sort()
        print(f"residential towers >=15 floors: {len(towers)}, "
              f"median dwellings/permit: {units[len(units)//2]}")
