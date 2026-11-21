# Weekday bus boardings in Santiago

Use these observed targets to compare simulated bus boardings by stop or comuna. Keep them out of model inputs and fitting if they will be used for evaluation. The same observations also appear in `data/mvp/bus_activity.csv`.

DTPM reports the average for the five weekdays of April 13–17, 2026. We select `Tipo_dia=LABORAL` and `Modo=BUS` in sheet `SUBIDAS_2026_04`. DTPM uses payment records and bus GPS to estimate each boarding stop. Transfers can generate multiple boardings. The workbook excludes fare evasion. [DTPM April workbook](https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2026.04_v3.xlsx)

## Stop target

`bus_stops.csv` has one row per workbook public stop code and local half hour. `half_hour` marks the start of the interval in America/Santiago time. A blank `boardings` cell means the workbook has no observation. Exclude those cells when scoring. Keep explicit zeroes if present. The file uses one decimal place for these five-day averages.

Longitude and latitude come from the exact match of workbook `Paradero Usuario` to GTFS `stops.txt.stop_id`. The five-digit CUT comes from matching the workbook comuna name to `data/mvp/communes.csv`. [DTPM March GTFS](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip)

| Weekday coverage | Result |
| --- | ---: |
| Identified public stop codes with coordinates | 11,931 of 11,931 |
| Unplaced workbook codes, including its unknown-stop bucket | 1 of 11,932 (0.0084%) |
| Observed stop and half-hour cells | 354,267 |
| Missing stop and half-hour cells | 218,469 |
| Average weekday boardings | 2,512,170.4 |
| Boardings without coordinates or CUT | 790.4 (0.0315%) |

DTPM groups unknown stops under `SIN_SIMIT` (internal code `SIN_PARADERO`). We retain its 38 observed half hours with blank location fields. The number of physical stops behind this bucket is unknown.

## Comuna target

`by_comuna.csv` sums the observed stops in each half hour for each of the 39 named comunas in the workbook and the blank-CUT bucket. `observed_codes` counts source codes with an observation in the interval. `total_codes` counts all source codes for the comuna across the week. If only some stops reported, the sum covers those stops. If none reported, the sum is blank. Only 45 of the 1,920 rows have an observation for every code. When scoring a partial sum, include predictions only for codes with an observed value in that half hour in `bus_stops.csv`. The named comunas account for 2,511,380.0 boardings.

## Rebuild

Run `python3 tools/pack/bus_boardings.py` from the repository root. It uses the checked-in April bus observations joined to the March GTFS. To rebuild the full chain from the two DTPM downloads, run the commands below first. [Source hashes and join counts](../../mvp/bus_activity_manifest.json)

```sh
python3 tools/scrape/export_april_boardings.py data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip data/raw/dtpm/subidas_2026_04.xlsx data/raw/april-boardings
python3 tools/scrape/export_bus_activity.py data/raw/april-boardings data/mvp/metro_observations.json data/mvp
```
