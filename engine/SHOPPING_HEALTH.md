# Shopping and health trips on a normal weekday

This experiment compares two all-stations-open predictions. The reference uses April public-transport job weights. The candidate replaces part of each half-hour's entries with Metro legs from shopping and health tours. Call the separate `shopping_health_stations` binary to enable the candidate.

The model applies 2012 EOD sector rates to Census block populations. It draws the destination comuna from the observed distribution for all trips of that purpose and assumes it applies to direct home–activity–home tours. Each eligible OpenStreetMap object within the selected comuna has equal weight. Two objects can represent the same place. Missing sectors, destinations and objects reduce coverage. The model draws departure times, return times and both modes together from a regional tour profile. Overlap with individual work and study schedules is unknown. Errands are excluded because the survey records lack a usable destination activity for them.

Only Metro legs with distinct, reachable stations at both ends contribute entries. Each station must be within 1,000 m of its endpoint. The fit uses daily outbound exits at 95 training stations and the April OTROS arrival counts multiplied by the survey coverage share (0.013748779). One nonnegative coefficient scales the tours. Each half-hour keeps the reference total across training stations. The final score uses observed boardings at the 31 held-out stations. [Registered rule](https://discord.com/channels/1573514576400752640/1573688249287708682/1573723023813648525)

## Run on Juan’s machine

Use a full checkout of the candidate and run from the repository root. The separate input worktree pins the survey tables. Save the aggregate coverage and both score outputs.

```sh
git fetch origin data/eod-other-trips
run=$(mktemp -d)
git worktree add --detach "$run/inputs" d4fd18bf4e976cb935c17895daa2ef35d4ef8179
pack="$run/inputs/data/mvp"
arrivals="$run/inputs/data/validation/trip_purpose_2026_04/metro_arrivals.csv"
cargo run --locked --release --manifest-path engine/Cargo.toml --bin population -- "$pack" "$run/april.json" april-transport
cargo run --locked --release --manifest-path engine/Cargo.toml --bin shopping_health_stations -- "$pack" "$arrivals" "$run/april.json" "$run/candidate.json" > "$run/coverage.json"
python3 -I - "$run/april.json" "$run/candidate.json" <<'PY'
import math
import runpy
import sys

errors = []
for path in sys.argv[1:]:
    sys.argv = ["tools/pack/check_score.py", path]
    result = runpy.run_path(sys.argv[0])
    errors.append(result["num"])
    print(path, "absolute error", result["num"], "WAPE", result["num"] / result["den"])
baseline, candidate = errors
if not math.isclose(baseline, 234697.7735058091, rel_tol=0, abs_tol=0.000001):
    sys.exit("Reference differs from the registered baseline. Report both values.")
gain = baseline - candidate
print("Fewer boarding errors:", gain, "Gate:", "PASS" if gain >= 1000 else "FAIL")
PY
```

The reference WAPE is 0.41148380945017204 across 1,123 observed half-hours at 31 stations. Acceptance requires at least 1,000 fewer absolute boarding errors. Report the outcome and coverage on the first run.
