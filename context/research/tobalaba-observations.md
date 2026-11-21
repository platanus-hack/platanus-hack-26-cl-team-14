# Tobalaba observed boarding layer

`data/mvp/tobalaba_observations.json` is a small browser-ready observation layer. It contains one station marker and 105 real April 2026 records. It does not change with a jobs/housing scenario and is not a forecast or a count of MUT visitors.

## Rebuild

First run [the April exporter](transport-export-handoff.md), then:

```sh
python tools/scrape/export_tobalaba_observations.py \
  /tmp/santiago-people/data/raw/transport-verified/gtfs.zip \
  /tmp/santiago-transport-export/data/raw/april-boardings/boardings.csv \
  data/mvp/tobalaba_observations.json
```

Python 3.11+, standard library only. Inputs are local; no downloads. The JSON records source URLs, hashes, raw GTFS station/platform records, exact crosswalk rule, source worksheet row numbers, periods, and limits. Coordinates are WGS84 latitude/longitude. Observation `boardings` values are JSON numbers; source hashes point back to the full-precision CSV.

## Why this station match is supported

Read official March GTFS records on 2026-11-21. There is exactly one `stop_name` equal to `TOBALABA` after case conversion: `TB-TOB`, named `Tobalaba`, with `location_type=1` and no parent. Coordinates: latitude -33.418119835792474, longitude -70.60192866420536.

The four platform records `TB_L1_V1`, `TB_L1_V2`, `TOB_L4_V1`, `TOB_L4_V2` all name Tobalaba and point to `parent_station=TB-TOB`. The [official GTFS reference](https://gtfs.org/documentation/schedule/reference/#stopstxt), opened this session, defines location type 1 as a station and the parent relationship for its platforms. This supports a station-level marker, rather than choosing an entrance or one platform.

The DTPM worksheet records use `Modo=Metro`, internal `Paradero=TOBALABA`, user code `-`, and comuna `PROVIDENCIA`. We explicitly map that single reviewed station to `TB-TOB`. The code does not perform generic or fuzzy name matching. No boardings are divided among platforms, directions, or entrances.

## Manual result

| Day type | Source period | Half-hour records | Boarding total |
| --- | --- | ---: | ---: |
| LABORAL | Average April 13–17, 2026 | 37 | 81,612.8 |
| SABADO | April 11, 2026 | 35 | 46,461 |
| DOMINGO | April 12, 2026 | 33 | 27,151 |

The exporter was run on the complete April CSV. The resulting JSON, station record, platform references, first/last observations, and totals were read manually. No tests were run, per repository rules. No nearby bus stops are included in this bounded version.

The measured quantity is validation-derived boardings, excluding estimated fare evasion. Transfers may affect person counts. Missing half-hours remain missing, not zero. The geographic point represents the station, not a boarding gate or the MUT building. UI labels must retain the day-type average and April 2026 period.
