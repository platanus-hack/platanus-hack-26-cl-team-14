"""Station score for the page -> web/data/score.json. This script computes nothing itself.

It runs the Rust station model (layer L1: points within 1,000 m of the nearest Metro parent)
and the frozen scorer, and keeps their outputs for the 31 held-out stations:
observed April 2026 LABORAL boardings and the model prediction, 48 half hours each.
Run from the repo root: python3 web/score.py   (needs cargo on PATH)
"""
import json, re, subprocess, sys

sys.argv = sys.argv[:1]  # the scorer reads sys.argv at import
PRED = "/tmp/station_predictions.json"
subprocess.run(["cargo", "run", "--release", "--manifest-path", "engine/Cargo.toml", "--bin", "stations",
                "--", "data/mvp", PRED, "1000"], check=True)
out = subprocess.run([sys.executable, "-I", "tools/pack/check_score.py", PRED], check=True, capture_output=True, text=True).stdout
wape = dict(re.findall(r"(WAPE_\w+)\s*=\s*([\d.]+)", out))

exec(re.search(r"HOLDOUT = \{.*?\n\}", open("tools/pack/check_score.py").read(), re.S).group())  # the frozen 31 IDs
pred = json.load(open(PRED))
stations = [{"id": s["station"]["id"], "name": s["station"]["name"], "lon": s["station"]["longitude"], "lat": s["station"]["latitude"],
             "observed": s["half_hour_bins"]["LABORAL"], "predicted": [round(x, 1) for x in pred[s["source_name"]]]}
            for s in json.load(open("data/mvp/metro_observations.json"))["stations"] if s["station"]["id"] in HOLDOUT.values()]
json.dump({"layer": "L1", "wape_bins": float(wape["WAPE_bins"]), "wape_daily": float(wape["WAPE_daily"]), "stations": stations},
          open("web/data/score.json", "w"))
print(len(stations), "stations", wape)
