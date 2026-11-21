"""Write data/mvp/metro_training.json: all Metro parents with coordinates, LABORAL bins only for training stations.

Usage: python3 -I tools/pack/metro_training.py
The held-out split is calm-condor's frozen rule (tools/pack/check_score.py): sort the normalized
names, drop TOBALABA, LOS LEONES, EL GOLF, take every 4th from index 0. Held-out stations keep
their coordinates but get "laboral": null, so their counts never reach the Rust binary.
"""
import json

USED = {"TOBALABA", "LOS LEONES", "EL GOLF"}
st = json.load(open("data/mvp/metro_observations.json"))["stations"]
holdout = set(sorted({s["source_name"] for s in st} - USED)[::4])
out = [{"name": s["source_name"], "id": s["station"]["id"], "longitude": s["station"]["longitude"],
        "latitude": s["station"]["latitude"], "holdout": s["source_name"] in holdout,
        "laboral": None if s["source_name"] in holdout else s["half_hour_bins"]["LABORAL"]} for s in st]
json.dump(out, open("data/mvp/metro_training.json", "w"), separators=(",", ":"))
print(len(out), "stations,", len(holdout), "held out")
