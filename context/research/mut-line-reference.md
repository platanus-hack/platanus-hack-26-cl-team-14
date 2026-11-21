# Tobalaba against qualifying L1/L4 workbook records

Tobalaba's April-to-August 2023 change exceeds its line reference median by **2.9157 percentage points on L1** and **21.8812 points on L4**. This is a descriptive comparison, not an identified MUT effect or validation of the closure scenario.

## Frozen rule and source universe

[Method](../../data/validation/mut_2023/line_reference/method.md) frozen at **2026-11-21T08:57:28Z**, before inspecting comparison station changes. Tobalaba's own line totals were already known from the [earlier source extraction](mut-2023-boardings.md).

Reuse the unchanged official [April](https://www.dtpm.cl/descargas/modelos_y_matrices/Tablas%20de%20Subidas%20y%20Bajadas.zip) and [August](https://www.dtpm.cl/descargas/modelos_y_matrices/Tablas%20de%20subidas%20y%20bajadas%20ago23.zip) boarding members. Both source sheets define sampled weekday averages: April 17–21 and August 7–11, excluding evasion. August's row label covers August 7–13, but its source sheet and title restrict the mean to weekdays August 7–11.

The source universe is all exact station-name rows with `servicio Sonda=L1` or `L4` in either workbook. Match exact `parada SIMT` plus line. Include a key only when it appears once in each cut, all fixed 36 bins from 05:30 through 23:00 have one column and numeric cells in both cuts, and the April sum is positive. Exclude missing or duplicate rows and record each reason. Do not aggregate duplicates or select stations after viewing changes.

Each source has **27 L1 and 20 L4 keys**. This is coverage of qualifying records in these workbooks, not verified coverage of every physical-line station. Stations absent from both cuts cannot appear in the union or exclusion table. No external station roster was added.

## Descriptive result

For each included station, sum only the fixed 36 bins and compute `100 × (August / April − 1)`. Each line reference is the unweighted median of its included station changes, excluding TOBALABA. With an even reference count, average the two middle changes. Subtract that median from Tobalaba's change.

| Line | Included keys | References excluding Tobalaba | Tobalaba change | Reference median change | Tobalaba minus median |
| --- | ---: | ---: | ---: | ---: | ---: |
| L1 | 27 | 26 | +2.7724% | −0.1433% | +2.9157 percentage points |
| L4 | 19 | 18 | +20.4042% | −1.4771% | +21.8812 percentage points |

The complete [included table](../../data/validation/mut_2023/line_reference/included_stations.csv) has all 46 eligible keys, both source sums, row references, changes, and reference membership. The [results JSON](../../data/validation/mut_2023/line_reference/results.json) preserves unrounded decimal calculations.

The sole [exclusion](../../data/validation/mut_2023/line_reference/exclusions.csv) is exact source key **PLAZA EGANA/L4**: its 23:00 cell is null in April row 38094 and August row 38563. It fails the frozen rule. All other keys occur once in each cut, with numeric values in every fixed bin and positive April sums. There are no unmatched, duplicate, or unnamed L1/L4 records in this source universe.

The [source CSV](../../data/validation/mut_2023/line_reference/station_half_hours.csv) preserves all 94 candidate records and all 48 half-hour cells, including the excluded station. Empty cells remain nulls. The [provenance JSON](../../data/validation/mut_2023/line_reference/provenance.json) records exact official member names, SHA-256 hashes, source definitions, headers, and artifact hashes. Values outside the fixed bins are retained for inspection but do not enter the statistic.

## Limits

Keep lines separate. Within-Metro interchange counting and line assignment remain unresolved; these records are not verified as unique station entrants. They were not merged with the 2026 single-station data.

Two sampled weeks cannot establish parallel trends. Season, service and network changes, occupancy ramp, and economic changes remain unresolved. Reference stations can share MUT demand or receive displaced demand. The reference median does not isolate these mechanisms or turn the difference into MUT's causal effect.
