# Three reviewed Metro observation crosswalks

Scope: Tobalaba, Los Leones and El Golf only. These April 2026 station observations are fixed reference data, not scenario output. The exporter uses an explicit allowlist, not generic name matching.

## Source evidence

Read the complete local April boarding CSV and March GTFS `stops.txt` on 2026-11-21. Source URLs, input hashes, source worksheet row numbers and full parent/platform records are in each JSON. The [April export handoff](transport-export-handoff.md) records the workbook meaning and dates; [Tobalaba evidence](tobalaba-observations.md) records the original parent mapping.

| April internal stop (`mode=Metro`, user code `-`) | April comuna label | Unique exact-name GTFS parent (`location_type=1`) | Child platforms (`location_type=0`) |
| --- | --- | --- | --- |
| TOBALABA | PROVIDENCIA | `TB-TOB` | `TB_L1_V1`, `TB_L1_V2`, `TOB_L4_V1`, `TOB_L4_V2` |
| LOS LEONES | PROVIDENCIA | `LE-LEN` | `LE_L1_V1`, `LE_L1_V2`, `LEN_L6_V1`, `LEN_L6_V2` |
| EL GOLF | LAS CONDES | `GO` | `GO_L1_V1`, `GO_L1_V2` |

Each exact station name occurs once in this GTFS after case conversion. Los Leones has L1 and L6 platforms under the same parent. DTPM gives one LOS LEONES observation per day type/half-hour, with no line identifier. Map it once to `LE-LEN`; never copy the total to each line or platform. The parent relation establishes geography, not a measured split between lines. The same rule applies to Tobalaba L1/L4. No entrances receive allocated boardings.

## Published outputs and manual result

| JSON under `data/mvp/` | LABORAL records / total | SABADO records / total | DOMINGO records / total |
| --- | ---: | ---: | ---: |
| `tobalaba_observations.json` | 37 / 81,612.8 | 35 / 46,461 | 33 / 27,151 |
| `los_leones_observations.json` | 36 / 54,840 | 35 / 28,574 | 33 / 11,012 |
| `el_golf_observations.json` | 36 / 14,953.6 | 34 / 4,602 | 33 / 2,600 |

LABORAL is the average of April 13–17; Saturday is April 11; Sunday is April 12. Actual source rows and output arrays were read. Each station has no duplicate day-type/half-hour keys. Tobalaba's existing observations and totals are unchanged. No tests or downloads were run.

All three files retain `observations` and add `half_hour_bins[day_type]`, a 48-value array. Index k starts at local k×30 minutes in America/Santiago. Missing source bins are JSON `null`, not zero. These arrays and the observation records are two representations of the same data: do not sum them together. Render a missing bin as unavailable; do not interpolate, carry a previous value, or use a truthiness fallback. Published totals sum only present observations.

These are validation-derived boardings, excluding estimated fare evasion. They are not unique passengers, onboard train loads, arrivals or site visitors. Platform geography does not prove an April route schedule; the crosswalk uses March station geometry only.

## Rebuild from existing local files

Run the existing exporter once for each station, with the matching output name:

```sh
python3 tools/scrape/export_tobalaba_observations.py \
  /tmp/santiago-people/data/raw/transport-verified/gtfs.zip \
  /tmp/santiago-transport-export/data/raw/april-boardings/boardings.csv \
  data/mvp/los_leones_observations.json --station 'LOS LEONES'
```

Use `--station 'EL GOLF'` with `el_golf_observations.json`; the default remains TOBALABA with `tobalaba_observations.json`. Standard library only. The scene consumes the station ID and arrays directly; it does not infer another join by name.
