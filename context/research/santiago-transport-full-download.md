# DTPM April 2026: facts from the full files

Checked: 2026-11-21. This file adds to `santiago-transport-verified.md`, which read only the first 64 KiB of the trip and stage archives.
Here the full archives were downloaded and all 7 days were counted. Every number below comes from real rows.

## Join of boardings to GTFS (tested)

- `Paradero Usuario` in the boardings XLSX = `stop_id` in `GTFS_20260321_v3.zip`.
- 11,953 of the 11,954 bus codes are in the feed. The one that is not is `SIN_SIMIT`.
- The trip and stage files use only the internal code `Paradero` (`E-1-4-SN-50`). The XLSX is the lookup table from that code to `stop_id`.
- Metro and train rows have station names only (upper case, no accents). The name match to GTFS was not tested.

## Boardings XLSX

- 761,557 data rows. 12,090 stops: 11,954 bus, 126 Metro, 10 train.
- Sums computed from the rows: LABORAL 5,063,829; SABADO 2,766,864; DOMINGO 1,585,544.
- The Saturday and Sunday sums are equal to the stage row counts of April 11 and 12. Thus the XLSX counts raw validations. It is not expanded.

## Trips (`Viajes_2026-04.zip`, 1.66 GB, 7 daily `.csv.gz`)

| Day | Apr 11 | Apr 12 | Apr 13 | Apr 14 | Apr 15 | Apr 16 | Apr 17 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Rows | 1,320,747 | 691,101 | 2,617,616 | 2,664,062 | 2,683,405 | 2,668,412 | 2,545,727 |

Wednesday April 15:

- 3,814,010 expanded trips (sum of `factor_expansion`, mean about 1.42).
- All rows have a destination. The table keeps only trips with an estimated destination, and the factor covers the others.
- Top origins (expanded): TOBALABA 74,878; MANQUEHUE 51,199; LOS LEONES 51,192; UNIVERSIDAD DE CHILE 50,076; ESTACION CENTRAL 49,139.
- Stages for each trip: 73% have 1, 25% have 2, 3% have 3.

Facts that change how we use it:

- `tipodia` is a number: `0` on April 14 and 15, `1` on Saturday April 11.
- `proposito` has HOGAR, TRABAJO, ESTUDIO, OTROS. DTPM estimates it; it is not a survey answer. Role: TARGET for activity shares by hour.
- The card code is the same through the week: 63% of the cards of April 15 are also in April 14, and 63% of those start at the same stop on the two days.
  Use this only as an aggregate (regular riders for each stop). Do not publish card codes.

## Stages (`Etapas-2026-04.zip`, 2.0 GB, 35 columns)

| Day | Apr 11 | Apr 12 | Apr 13 | Apr 14 | Apr 15 | Apr 16 | Apr 17 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Rows | 2,766,864 | 1,585,544 | 5,027,059 | 5,055,365 | 5,108,476 | 5,098,041 | 5,030,206 |

- On April 15, 23% of stages (1,181,092) have no alighting stop.
- Bus stages have two modes: `BUS` and `ZP` (pay zone).

## GTFS, April against November

18 stop IDs are only in the April feed (V161). 140 are only in the November feed (V171). 3 routes are different each way.

## Not checked

Metro and train name match to GTFS. Days other than April 14 and 15 beyond row counts. Reuse licence.
The raw files are in `/tmp/transport-verified-data` on `v4rgas@laptop` (tmpfs; a restart deletes them).
