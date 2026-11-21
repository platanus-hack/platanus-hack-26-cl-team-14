# Opening a station in a scenario

The station model estimates how weekday public-transport commuters redistribute their Metro entries when a physical station opens. It uses home and job points from the synthetic population. Distances are straight-line distances. Every open station within 1,000 m of a point receives a share proportional to 1 / max(distance in metres, 50). If a point has no station in range, the existing one-bus access rule applies. The timetable supplies feeder links to existing stations. The model has no timetable for the proposed services.

Existing station classes, scales, public mode shares and half-hour profiles stay fixed at their baseline fit. A new station uses the work profile if its assigned walk job mass divided by its assigned walk home mass exceeds the original training median. As in the baseline model, zero home mass gives an infinite ratio. The scenario changes entry station choice. Job choice, mode switching, train paths, capacity and time savings require other evidence. Entry values for future stations are modeled counterfactuals. April observations can check the current network only.

## Engine contract

- Use `stations::access::planned::PlannedStations::load(data)` with the catalog from PR #184. `select(&[Selector::Line("L7")])` returns all matching source features, a map from each design station ID to its physical station ID, and unique new access points. Other selectors accept a corridor or a design station ID. Union all active design records before calculating station demand.
- Each existing interchange uses its current GTFS parent and coordinates. A shared future interchange has one stable physical ID and point across selectors. Opening L7 selects 19 design stations, of which three use existing parents. The projection therefore adds 16 entry stations. The separate proximity audit used all 19 project points. [GTFS station hierarchy](https://gtfs.org/documentation/schedule/reference/#stopstxt)
- For a user-supplied point, construct an `AddedStation { id, name, longitude, latitude }`. Supply one record per open physical ID. Resolve repeat requests, closures and current-parent aliases before projecting. Opening a line preserves closures of existing interchanges.
- Call `access.project_with_additions(homes, jobs, &open_existing, &additions)` on the scenario's public-weighted commuter masses. The result lists current parents in baseline order, followed by the additions in input order. New parents have 48 zero baseline bins. Use `event_access_share_with_additions(longitude, latitude, &open_existing, &additions)` for event access under the same scenario. New stations have unverified capacity.
- The output describes modeled weekday entries. It contains no evidence for a travel-time saving, a new transfer, or a traveler changing mode.

## Run on the test laptop

Use a full checkout of the candidate. Point both runs at the same data directory from the pinned input revision. Run the frozen checker from the full candidate checkout.

```sh
git fetch origin refs/pull/184/head
run=$(mktemp -d)
git worktree add --detach "$run/inputs" ee8045e1bda9ebd001dec7a27db4c342b6b30b36
pack="$run/inputs/data/mvp"
cargo run --locked --release --manifest-path engine/Cargo.toml --bin population -- "$pack" "$run/base.json" l9
python3 -I tools/pack/check_score.py "$run/base.json"
cargo run --locked --release --manifest-path engine/Cargo.toml --bin population -- "$pack" "$run/open.json" l9 line:L7
python3 - "$run/base.json" "$run/open.json" <<'PY'
import json
import math
import sys

base, opened = (json.load(open(path)) for path in sys.argv[1:])
added = opened.keys() - base.keys()
print("Existing stations:", len(base), "New physical stations:", len(added))
print("Valid half-hour series:", all(len(v) == 48 and all(math.isfinite(x) and x >= 0 for x in v) for v in opened.values()))
print("Entries at new stations:", sum(sum(opened[key]) for key in sorted(added)))
print("Change at existing stations:", sum(sum(opened[key]) - sum(base[key]) for key in base))
PY
```

Expected current-network result: 126 stations, 31 held out, 1,123 scored cells, half-hour WAPE 0.4179 and absolute error 238,349.700047. The L7 run should add 16 physical stations with finite nonnegative half-hour series. Actual projected entries require a remote run. The frozen score on the current network and the historical backcheck of station openings answer different questions. The threshold for a change to the baseline is 1,000 fewer absolute boarding errors.
