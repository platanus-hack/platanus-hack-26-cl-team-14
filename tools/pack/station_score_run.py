"""Write data/mvp/station_score_run.json: one reviewed station-model run, scored, for the score view.

Usage: python3 -I tools/pack/station_score_run.py <stations args...>
Example (L2): python3 -I tools/pack/station_score_run.py 1000 public
The layer label comes from the arguments; unknown arguments stop the run.
Runs the Rust `stations` binary, then the frozen scorer, then joins the observed target cells
AFTER the run (observed values never go back into Rust). Daily sums use only the observed cells.
"""
import datetime, hashlib, json, re, subprocess, sys

LAYERS = {(): "L0", ("1000",): "L1", ("1000", "public"): "L2", ("1000", "public", "classes"): "L7", ("1000", "public", "classes", "split"): "L4", ("1000", "public", "classes", "pair_mode"): "L8"}
args = sys.argv[1:]
layer = LAYERS[tuple(args)]
pred_path = "/tmp/station_predictions_" + layer + ".json"
run = subprocess.run(["cargo", "run", "-q", "--release", "--manifest-path", "engine/Cargo.toml", "--bin", "stations",
                      "--", "data/mvp", pred_path, *args], capture_output=True, text=True, check=True).stdout
report = json.loads(run.splitlines()[0])
score = subprocess.run(["python3", "-I", "tools/pack/check_score.py", pred_path], capture_output=True, text=True, check=True).stdout
printed = {k: float(re.search(k + r"\s*=\s*([\d.]+)", score).group(1)) for k in ("WAPE_bins", "WAPE_daily")}

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
                     "latitude": s["station"]["latitude"], "observed": o, "predicted": p,
                     "observed_daily": sum(o[t] for t in cells), "predicted_daily": sum(p[t] for t in cells)})
cells = sum(v is not None for s in stations for v in s["observed"])
# Full-precision WAPE from the stored values; the frozen scorer printed the same numbers to 4 decimals.
den = sum(s["observed_daily"] for s in stations)
wape_bins = sum(abs(s["predicted"][t] - v) for s in stations for t, v in enumerate(s["observed"]) if v is not None) / den
wape_daily = sum(abs(s["predicted_daily"] - s["observed_daily"]) for s in stations) / den
assert len(stations) == 31 and cells == 1123
assert round(wape_bins, 4) == printed["WAPE_bins"] and round(wape_daily, 4) == printed["WAPE_daily"]

out = {"layer": layer, "stations_args": args, "model_commit": git("log", "-1", "--format=%h", "--", "engine/src"),
       "model_source_sha256": sha("engine/src/stations.rs"),
       "input_sha256": {f: sha("data/mvp/" + f) for f in ("work_od.csv", "mode_share.csv", "blocks.csv", "jobs.csv", "metro_training.json", "mode_od.csv")},
       "scorer_commit": git("log", "-1", "--format=%h", "--", "tools/pack/check_score.py"),
       "scorer_sha256": sha("tools/pack/check_score.py"),
       "run_utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
       "target": "Metro boardings, April 2026, LABORAL average of 2026-04-13 to 2026-04-17 (DTPM)",
       "target_sha256": sha("data/mvp/metro_observations.json"), "predictions_sha256": sha(pred_path),
       "development_validation": True, "held_out_stations": 31, "observed_cells": cells,
       "wape_bins": wape_bins, "wape_daily": wape_daily, "scale": report["scale"],
       "assigned_home_mass": report["home_mass"], "assigned_job_mass": report["job_mass"],
       "stations": stations}
json.dump(out, open("data/mvp/station_score_run.json", "w"), ensure_ascii=False, separators=(",", ":"))
print(layer, wape_bins, wape_daily, cells, "cells")
