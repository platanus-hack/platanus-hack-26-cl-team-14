"""Write data/mvp/metro_training.json: all Metro parents with coordinates, LABORAL bins only for training stations.

Usage: python3 -I tools/pack/metro_training.py
Held-out stations: the 31 frozen GTFS parent IDs of calm-condor's scorer (tools/pack/check_score.py).
They keep their coordinates but get "laboral": null, so their counts never reach the Rust binary.
"""
import hashlib, json

SRC = "data/mvp/metro_observations.json"
SHA256 = "ecd8131fac2bfcf401bb5700dda7a7a0a31534990b8b962134d31449651ef5a3"  # PR #35 bytes
HOLDOUT = {"AL", "LF", "AG", "CER", "CCH", "DS", "EI", "ECO", "FCV", "RGR", "HSR", "LC-LCI", "LME", "LCR", "LV",
           "LOR", "MQ", "MEY", "NUN-NUO", "PB", "PE", "PPA", "PDG", "QN", "RO", "SJE", "AN-NA", "SRO", "TRI", "US", "VFR"}
raw = open(SRC, "rb").read()
assert hashlib.sha256(raw).hexdigest() == SHA256, "metro_observations.json changed; the frozen split may move"
st = json.loads(raw)["stations"]
assert {s["station"]["id"] for s in st} >= HOLDOUT
out = [{"name": s["source_name"], "id": s["station"]["id"], "longitude": s["station"]["longitude"],
        "latitude": s["station"]["latitude"], "holdout": s["station"]["id"] in HOLDOUT,
        "laboral": None if s["station"]["id"] in HOLDOUT else s["half_hour_bins"]["LABORAL"]} for s in st]
json.dump(out, open("data/mvp/metro_training.json", "w"), separators=(",", ":"))
print(len(out), "stations,", sum(o["holdout"] for o in out), "held out")
