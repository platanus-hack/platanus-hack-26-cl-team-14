# Final engine Metro checks

The final engine rerun uses commit `b611722bc503472996480f0fb55bc95829b7195d` and the `data/mvp` tree `962cb7adc50de0f3e9dc088f55b3245f9441ebbd`. These checks support the evaluation slide at 2:25. [Shared build](https://discord.com/channels/1573514576400752640/1573943617813348484/1573943809471946844)

## April 2026 weekday entries

The frozen scorer covers 31 held-out stations and 1,123 half-hour bins. DTPM reports 570,369.4 weekday entries in those bins. The model fits other stations. Repeated use of this holdout makes it a development check.

| Error measure | Sum of absolute errors | Observed entries | Error |
| --- | ---: | ---: | ---: |
| Daily totals by station | 169,282.9 | 570,369.4 | 29.68% |
| Half-hour bins by station | 238,349.7 | 570,369.4 | 41.79% |

Weighted absolute percentage error divides the sum of absolute errors by the observed total. The daily measure first sums the scored bins per station. This check compares baseline levels.

## November 2023 Line 2 extension

The replay removes four stations from the station catalog and feeder links. It then adds them as planned points. The fit uses 91 other stations. The rest of the network and commuter inputs use the 2026 pack. The observations are DTPM weekday averages from 15–19 April 2024. [Source and comparison limits](../../data/validation/metro_extension_2023/line2.md)

| Station | Modeled entries | Observed entries |
| --- | ---: | ---: |
| El Bosque | 12,498.5 | 10,841 |
| Observatorio | 15,410.9 | 6,980 |
| Copa Lo Martínez | 14,435.3 | 9,765 |
| Hospital El Pino | 17,453.2 | 23,039 |
| **Total** | **59,797.9** | **50,625** |

The error on the aggregate is 9,172.9 / 50,625 = **18.12%**. The sum of absolute errors at the four stations is 20,344.6 / 50,625 = **40.19%**. With all four stations closed, the model gives zero entries and both errors are 100%. The time gap and later network limit this to a descriptive comparison.

## Reproduce

Run from the root at the pinned commit. Use the shared scenario binary or build it with `cargo build --locked --release --manifest-path engine/Cargo.toml --bin scenario`. Set `scenario_bin` to its absolute path. The Line 2 helper builds and runs the population binary.

```sh
unset SANTIAGO_STATION_JOB_WEIGHTS
printf '[]\n' | "$scenario_bin" data/mvp - > /tmp/metro-april-scenario.json
jq '[.stations[] | {key: .id, value: .base_half_hour}] | from_entries' \
  /tmp/metro-april-scenario.json > /tmp/metro-april-bins.json
python3 -I tools/pack/check_score.py /tmp/metro-april-bins.json
git fetch origin a075a5aab7a82235ae6b801031d43a483ede87cd
git show a075a5aab7a82235ae6b801031d43a483ede87cd:tools/pack/station_opening_replay.py \
  > /tmp/metro-line2-opening.py
python3 -I /tmp/metro-line2-opening.py
```

The run used the original Line 2 helper with its Cargo call replaced by the shared `population` binary. The attached receipt includes that helper and both raw scores. [Run receipt](https://discord.com/channels/1573514576400752640/1573943617813348484/1573944383357845604)
