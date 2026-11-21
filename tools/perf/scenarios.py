"""Run against a local release server: python3 tools/perf/scenarios.py PORT NEW_DIR [BASELINE_DIR]."""
import argparse
import hashlib
import json
import statistics
import time
from pathlib import Path
from urllib.request import Request, urlopen


def change(kind, target, op, value):
    return {"kind": kind, "target": target, "op": op, "value": value}


mut = change("jobs", {"type": "block", "id": "13114131009033"}, "set", 0)
quilicura = change("jobs", {"type": "point", "longitude": -70.735771, "latitude": -33.362463}, "add", 20000)
maipu = change("residents", {"type": "point", "longitude": -70.769317, "latitude": -33.510979}, "add", 5000)
minutes = change("minutes", {"type": "comuna", "id": "13125"}, "add", -10)
jobs = change("jobs", {"type": "comuna", "id": "13119"}, "multiply", 1.2)
stations = {s: change("station", {"type": "station", "id": s}, "set", 0) for s in ("FER", "LCR", "PQU")}
cases = {
    "empty": [], "mut": [mut], "quilicura": [quilicura], "maipu": [maipu],
    "minutes": [minutes], "multiplier": [jobs],
    **{s: [ch] for s, ch in stations.items()}, "three-stations": list(stations.values()),
    "combined": [mut, quilicura, maipu, stations["PQU"]],
    "event-maipu": [{**change("event", maipu["target"], "set", 1500), "assumed_end_half_hour": 40}],
}


def run(port, directory, baseline):
    directory.mkdir(parents=True)
    expected = json.loads((baseline / "results.json").read_text()) if baseline else {}
    results = {}
    for name, changes in cases.items():
        body = json.dumps({"changes": changes}).encode()
        request = Request(f"http://127.0.0.1:{port}/api/scenario", body, {"Content-Type": "application/json"})
        durations, wall, sizes = [], [], []
        for i in range(6):
            start = time.perf_counter()
            with urlopen(request) as response:
                raw = response.read()
            duration = (time.perf_counter() - start) * 1000
            data = json.loads(raw)
            if name == "event-maipu" and len(data["events"]) != 1:
                raise SystemExit("The event response must contain the modeled event")
            if len(data["stations"]) != 126 or not isinstance(data["trips_delta"], list):
                raise SystemExit(f"{name}: incomplete station or trip response")
            for station in data["stations"]:
                if len(station["base_half_hour"]) != 48 or len(station["scenario_half_hour"]) != 48:
                    raise SystemExit(f"{name}: incomplete half-hour series for {station['id']}")
            elapsed = data.pop("elapsed_ms")
            del data["revision"]
            canonical = json.dumps(data, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
            digest = hashlib.sha256(canonical).hexdigest()
            if i == 0:
                (directory / f"{name}.json").write_bytes(raw)
                reference = expected[name]["sha256"] if baseline else digest
            if digest != reference or data["not_modeled"] or len(data["applied"]) != len(changes):
                (directory / f"{name}-failed.json").write_bytes(raw)
                raise SystemExit(f"{name} run {i}: expected {reference}, got {digest}; inspect the saved response")
            if i:
                durations.append(elapsed)
                wall.append(duration)
                sizes.append(len(raw))
        results[name] = dict(sha256=digest, engine_ms=statistics.median(durations),
                             http_ms=round(statistics.median(wall), 2), bytes=statistics.median(sizes))
        print(name, *results[name].values(), flush=True)
    (directory / "results.json").write_text(json.dumps(results, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("port", type=int)
    parser.add_argument("directory", type=Path)
    parser.add_argument("baseline", type=Path, nargs="?")
    args = parser.parse_args()
    run(args.port, args.directory, args.baseline)
