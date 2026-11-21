# Frozen descriptive line reference method

Frozen before inspecting comparison station changes, 2026-11-21T08:57:28Z. Source workbooks are the unchanged official April and August 2023 boarding members already retrieved. No new sources or fit.

Population: all exact station-name rows whose `servicio Sonda` is exactly L1 or L4, with nonblank `parada SIMT`, in either original workbook. Match by exact `parada SIMT` plus line. A station must appear exactly once in each workbook for its line. Do not aggregate duplicate rows. Include only if both cuts have exactly one column for each fixed half-hour from 05:30 through 23:00 (36 bins), all 36 cell values are numeric, and the April 36-bin sum is positive. Record all excluded station/line keys and exact reasons, including missing rows, duplicates, bin-column differences, missing/non-numeric values, or non-positive April sum. Retain source rows, metadata, values, nulls, headers, and source member hashes.

Calculate each included station's April and August sums over those 36 bins, then `100 * (August / April - 1)`. For each line, exclude TOBALABA from its own reference and take the unweighted median of the remaining included station percent changes. For an even number of observations, median is the arithmetic mean of the middle two values. Primary descriptive statistic: Tobalaba percent change minus this median, in percentage points. Keep Tobalaba separate for each line. Retain the full included table and all exclusions; no station selection after viewing results, no weighting, and no combined-line statistic.

This is a descriptive two-week comparison. It is not an identified MUT effect or closure validation. These two weeks cannot establish parallel trends. Season, occupancy ramp, service/network changes, economic changes, and spillovers remain unresolved. Nearby stations are references, not validated causal controls.

No model, UI, scoring, tests, new downloads, or dependencies. First save source extracts, method, results and exclusions privately, and report before repository edits. A later separate docs/data PR can preserve this evidence after review.
