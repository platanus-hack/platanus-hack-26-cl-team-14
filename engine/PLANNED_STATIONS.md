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

Juan’s first CLI run reproduced the baseline absolute error of 238,349.700047. It projected 253,310.13 weekday entries at the 16 new L7 parents and a decrease of 87,912.36 at the existing 126, for a net increase of 165,397.77. These are outputs of the access rule. Historical validation remains open. The four stations in the L2 extension benchmark all appear in the 2026 training data. A retrospective comparison must exclude their boarding observations from any fit and label the difference between the 2024 observation year and the 2026 network and population inputs. [CLI result](https://discord.com/channels/1573514576400752640/1573688245361844345/1573729347754524714) · [L2 observations](../data/validation/metro_extension_2023/line2.md)

## Historical comparison with four Line 2 openings

Ernesto-29 ran `python3 -I tools/pack/station_opening_replay.py` at `a075a5aab7a82235ae6b801031d43a483ede87cd` on Juan’s Linux machine. The fit used 91 training stations in a network of 122 parents. It excluded the four comparison stations and their feeder links. The opening scenario then added those four station points.

| Station | Observed weekday entries in April 2024 | Modeled minus observed |
| --- | ---: | ---: |
| Copa Lo Martínez | 9,765 | +4,670.30 |
| El Bosque | 10,841 | +1,657.53 |
| Hospital El Pino | 23,039 | −5,585.82 |
| Observatorio | 6,980 | +8,430.92 |

The four stations received 59,797.93 modeled entries against 50,625 observed. The total was 18.1% high. The sum of absolute station errors was 20,344.56, or 40.2% of observed entries. Observatorio had the largest error. The comparison uses April 2024 counts with the remaining 2026 network and commuter inputs. It provides no error bound for future L7 entries. [External run](https://discord.com/channels/1573514576400752640/1573688249287708682/1573731227658232051)
