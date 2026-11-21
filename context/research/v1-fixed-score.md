# V1 Metro baseline check

Use the frozen April 2026 Metro score to check baseline station predictions. The project uses the [V4 real-event evaluation](../verticals.md#v4--evals) as its fixed score for new layers. Run that evaluation when V4 publishes its command and data. The April Metro score can identify baseline changes while V4 develops the evaluation.

## Observed target

The DTPM workbook reports average weekday boardings for April 13–17, 2026. The scorer fixes 31 held-out stations, 1,123 observed station half-hours, and 570,369.4 boardings. Counts come from validations and exclude fare evasion. They measure station entries. See the [DTPM index](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) and [source audit](metro-observed-targets.md).

The [scorer](../../tools/pack/check_score.py) divides the sum of absolute errors by the fixed observed total. Lower half-hour WAPE is better. It also reports daily WAPE, which sums the same cells by station before taking absolute errors. Missing or invalid predictions in an observed cell make the run ineligible. Keep all held-out boardings and their derived profiles out of fitting. Repeated comparisons use the holdout for development.

## Run the replay engine baseline

Run from the repository root. Use the same revision and data for both settings.

```sh
set -o pipefail
score_dir=$(mktemp -d /tmp/v1-score.XXXXXX)
cargo build --locked --release --manifest-path engine/Cargo.toml --bin scenario
for weights in l9 april-transport; do
  printf '[]\n' | SANTIAGO_STATION_JOB_WEIGHTS="$weights" engine/target/release/scenario data/mvp - |
    jq '[.stations[] | {key: .id, value: .base_half_hour}] | from_entries' > "$score_dir/$weights.json"
  python3 -I tools/pack/check_score.py "$score_dir/$weights.json"
done
```

At commit `f623d40bc40495a1824030861f2fd2445777e5a0`, both zero-edit runs covered all 1,123 cells.

| Replay baseline | Half-hour WAPE | Daily WAPE |
| --- | ---: | ---: |
| L9 | 0.4179 | 0.2968 |
| April transport | 0.4115 | 0.2964 |

The default replay engine uses L9. To compare a proposed layer with its predecessor, keep the same weighting option on both runs unless the layer changes that option. Record the full commit, option, and both scores. The scenario output rounds station counts to one decimal. The [population CLI](../../engine/README.md#april-weights-for-station-access) supplies the unrounded comparison: L9 0.41788655 and April weights 0.41148381 for half-hour WAPE. The checked-in [L4b score artifact](../../data/mvp/station_score_run.json) records 0.44593458 from the earlier station binary.

## What to report for the three replays

- **MUT → Monumental.** Compare matched event or site counts when available. State the time window and whether the source measures entries or exits. See the [match evidence](monumental-pre-match-alightings.md).
- **Mapocho navigable.** Show modeled boat demand under the stated service properties. The [Mapocho brief](mapocho-navegable.md) measures existing travel in the corridor. It supplies context for demand and does not observe boat uptake.
- **Close the full Metro.** Compare with the October 2019 observation at the service and time coverage that V1 verifies. Use April 2026 for the baseline score. Zero boardings at closed stations verify the modeled closure. To measure redistribution, compare observed trips or counts for the modes that absorbed demand.
