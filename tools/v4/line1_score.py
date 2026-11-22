"""V4 score: Line 1 interruption, Monday 18 Nov 2024, bins 27-28 (13:30, 14:00).
Usage: python3 tools/v4/line1_score.py [path/to/scenario-binary]"""
import json, subprocess, sys

BIN = sys.argv[1] if len(sys.argv) > 1 else "engine/target/release/scenario"
CLOSED = ["SA", "MM", "PV", "GO", "AL"]
OBS = {"BA-BQ": [536, 878], "EM": [-734, -826]}  # Monday minus Thursday, DTPM etapas
REF = 3368  # Thursday stages in the four cells
EVENT = [{"kind": "station", "target": {"type": "station", "id": s}, "op": "set", "value": 0, "half_hours": [27, 29]} for s in CLOSED]

def run(changes):
    out = subprocess.run([BIN, "data/mvp", "-"], input=json.dumps(changes), capture_output=True, text=True, check=True).stdout
    return {s["id"]: s["scenario_half_hour"] for s in json.loads(out)["stations"]}

base, event = run([]), run(EVENT)
err = 0
for sid, obs in OBS.items():
    for k, t in enumerate([27, 28]):
        pred = event[sid][t] - base[sid][t]
        err += abs(pred - obs[k])
        print(f"{sid:6} bin {t}  base {base[sid][t]:8.1f}  event {event[sid][t]:8.1f}  pred {pred:+8.1f}  obs {obs[k]:+5d}")
for sid in CLOSED:
    print(f"{sid:6} closed  bins 27-28 event {event[sid][27]:.1f}, {event[sid][28]:.1f}")
print(f"score {err / REF:.4f}  (no change 0.8830)")
