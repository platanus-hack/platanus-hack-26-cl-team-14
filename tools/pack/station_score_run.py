"""Write data/mvp/station_score_run.json: one reviewed station-model run, scored, for the score view.

Usage: python3 -I tools/pack/station_score_run.py <layer> [stations args...]
Example (L2): python3 -I tools/pack/station_score_run.py L2 1000 public
Runs the Rust `stations` binary, then the frozen scorer, then joins the observed target cells
AFTER the run (observed values never go back into Rust). Daily sums use only the observed cells.
"""
import datetime, hashlib, json, re, subprocess, sys

layer, args = sys.argv[1], sys.argv[2:]
pred_path = "/tmp/station_predictions_" + layer + ".json"
run = subprocess.run(["cargo", "run", "-q", "--release", "--manifest-path", "engine/Cargo.toml", "--bin", "stations",
                      "--", "data/mvp", pred_path, *args], capture_output=True, text=True, check=True).stdout
report = json.loads(run.splitlines()[0])
score = subprocess.run(["python3", "-I", "tools/pack/check_score.py", pred_path], capture_output=True, text=True, check=True).stdout
wape_bins, wape_daily = (float(re.search(k + r"\s*=\s*([\d.]+)", score).group(1)) for k in ("WAPE_bins", "WAPE_daily"))

sha = lambda p: hashlib.sha256(open(p, "rb").read()).hexdigest()
git = lambda *a: subprocess.run(["git", *a], capture_output=True, text=True).stdout.strip()
holdout = {s["id"] for s in json.load(open("data/mvp/metro_training.json")) if s["holdout"]}
pred = json.load(open(pred_path))
stations = []
for s in json.load(open("data/mvp/metro_observations.json"))["stations"]:
    if s["station"]["id"] not in holdout:
        continue
    o, p = s["half_hour_bins"]["LABORAL"], pred[s["source_name"]]
    cells = [t for t in range(48) if o[t] is not None]
    stations.append({"id": s["station"]["id"], "name": s["station"]["name"], "longitude": s["station"]["longitude"],
                     "latitude": s["station"]["latitude"], "observed": o, "predicted": [round(v, 2) for v in p],
                     "observed_daily": round(sum(o[t] for t in cells), 1), "predicted_daily": round(sum(p[t] for t in cells), 1)})
cells = sum(v is not None for s in stations for v in s["observed"])
num = sum(abs(s["predicted"][t] - v) for s in stations for t, v in enumerate(s["observed"]) if v is not None)
assert len(stations) == 31 and cells == 1123
assert abs(num / sum(s["observed_daily"] for s in stations) - wape_bins) < 5e-4  # rounding of stored values only

out = {"layer": layer, "stations_args": args, "model_commit": git("log", "-1", "--format=%h", "--", "engine/src"),
       "scorer_commit": git("log", "-1", "--format=%h", "--", "tools/pack/check_score.py"),
       "run_utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
       "target": "Metro boardings, April 2026, LABORAL average of 2026-04-13 to 2026-04-17 (DTPM)",
       "target_sha256": sha("data/mvp/metro_observations.json"), "predictions_sha256": sha(pred_path),
       "development_validation": True, "held_out_stations": 31, "observed_cells": cells,
       "wape_bins": wape_bins, "wape_daily": wape_daily, "scale": report["scale"],
       "assigned_home_mass": round(report["home_mass"], 1), "assigned_job_mass": round(report["job_mass"], 1),
       "stations": stations}
json.dump(out, open("data/mvp/station_score_run.json", "w"), ensure_ascii=False, separators=(",", ":"))
print(layer, wape_bins, wape_daily, cells, "cells")
