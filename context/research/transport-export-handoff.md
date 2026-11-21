# April boarding export

Offline data preparation for the Rust MVP. The source is already aggregated by stop, mode, day type, and half hour. The exporter preserves these observations; it does not simulate trips or infer station joins.

## Run

Python 3.11 or later; no dependencies. Download the two public source files listed in [the manifest](transport-export-manifest.json), or reuse the verified local files:

```sh
python tools/scrape/export_april_boardings.py \
  /tmp/santiago-people/data/raw/transport-verified/gtfs.zip \
  /tmp/santiago-people/data/raw/transport-verified/boarding.xlsx \
  data/raw/april-boardings
```

Arguments are positional: GTFS ZIP, boarding XLSX, output directory. The command makes no network requests. Large outputs stay in ignored `data/raw/`; Rust consumers can read the CSV directly.

- `boardings.csv`: `source_row,day_type,mode,internal_stop,user_stop,comuna,half_hour,boardings,gtfs_stop_id,stop_lat,stop_lon,join_status`.
- `unmatched_stops.csv`: unmatched records grouped by day type, mode, internal/user stop, comuna, and reason, with row counts and boarding sums. The complete unmatched observations also remain in `boardings.csv`.
- `manifest.json`: source URLs, SHA-256 hashes, byte sizes, period definitions, row counts, join totals, duplicate counts, cached-summary comparison, and output hashes. Decimal totals are JSON strings to preserve source precision.

`half_hour` is local HH:MM in America/Santiago, converted from Excel day fractions. `boardings` preserves the source numeric text. LABORAL is the average of April 13–17, not their sum; SABADO is April 11 and DOMINGO is April 12. Empty GTFS fields mean unresolved location, not zero demand. Missing worksheet observations are not filled with zeros.

## Manual run: 2026-11-21

Read the complete worksheet: **761,557 observations**, no duplicate source keys and no observed zero rows. Read **18,364 GTFS stops**. Output CSV: 84,979,065 bytes. The committed manifest is a copy from this run; the full CSV is not committed.

| Day type | Matched BUS rows | Unmatched BUS rows | Unmatched BUS boardings |
| --- | ---: | ---: | ---: |
| DOMINGO | 174,594 | 25 | 253 |
| LABORAL | 354,229 | 38 | 790.4 |
| SABADO | 218,611 | 38 | 777 |

All unmatched BUS rows use `SIN_PARADERO / SIN_SIMIT / SIN_COMUNA`. The other **14,022 unresolved rows** are Metro and Tren; their user code is not mapped here. The unmatched summary has 411 stop/day keys across all modes.

Recomputed all nine mode/day sums. Largest absolute difference from the publisher's cached summary is 0.000000806 boardings, consistent with stored floating-point values. Weekday totals, rounded for display: BUS 2,512,170.4; Metro 2,469,425.4; Tren 82,233.6. No tests were added or run, per repository rules.

Verified join example: worksheet internal `E-1-4-SN-50`, user code `PB1559` → GTFS `stop_id=PB1559`, latitude -33.403198097893515, longitude -70.68147008515123. GTFS `stop_code` is empty. The exporter joins exact BUS user codes to `stop_id`; it does not match internal codes or station names. Duplicate source keys, if present in another workbook, remain separate and are counted; consumers must not silently sum them.

## Research and limits

The [verified source report](santiago-transport-verified.md) documents the matching GTFS calendar and the workbook's fields and measurement method. Local source records were reread before implementation. Boardings derive from payment validations and bus GPS; they exclude estimated fare evasion and are not unique riders. GTFS supplies scheduled geometry, not observed vehicle movements. These observations do not identify MUT-level flows or causal effects.

Implementation choice: use Python's standard-library ZIP/CSV/XML readers, already used by the local verifier. The existing `dtpm_subidas_peek.py` only reads shared strings, so it cannot supply worksheet records. The [ElementTree documentation](https://docs.python.org/3/library/xml.etree.elementtree.html#xml.etree.ElementTree.iterparse) and [CPython implementation](https://raw.githubusercontent.com/python/cpython/3.13/Lib/xml/etree/ElementTree.py), opened on 2026-11-21, support incremental parsing. Remove processed rows from the XML parent to avoid retaining the full worksheet. Read workbook relationships and shared strings, then numeric cells; do not infer rows from shared strings alone. No new package is needed for this fixed verified workbook.

The exporter targets this April workbook layout, including its cached RESUMEN cells. It is not a general spreadsheet reader. A new publication needs its schema and periods checked. Rust simulation work is separate. Boardings and DTPM trip/stage data share a measurement process; they are not independent holdouts for each other.
