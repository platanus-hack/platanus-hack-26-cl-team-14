# Tobalaba half-hour descriptive references

The [complete half-hour summary](../../data/validation/mut_2023/half_hour_reference/line_bin_summary.csv) contains all **96 line/bin rows**: 48 half-hours each for L1 and L4. It shows reference counts, median percentage changes, Tobalaba's same-bin changes, and their differences in percentage points. No peaks were selected and no shape was fitted.

## Method and coverage

The [method](../../data/validation/mut_2023/half_hour_reference/method.md) was frozen at **2026-11-21T09:03:24Z**, before inspecting bin changes. It keeps the [earlier eligible set](mut-line-reference.md): Tobalaba plus 26 L1 references and 18 L4 references, with no new station selection. The source universe remains qualifying workbook records, not a verified physical-line roster.

For each fixed half-hour from 05:30 through 23:00, compute each station's `100 × (August / April − 1)`. Take the unweighted reference median, excluding Tobalaba, then subtract that median from Tobalaba's same-bin change. With an even count, average the middle two reference changes. Keep lines separate.

| Line | Fixed bins with a calculated contrast | Valid references in every fixed bin | Zero-April cell exclusions | Outside-window bins with null statistics |
| --- | ---: | ---: | ---: | ---: |
| L1 | 36 | 26 | 0 | 12 |
| L4 | 36 | 18 | 0 | 12 |

A zero April cell would make percentage change undefined for that station/bin; it would never become a zero change. No zero-April or other fixed-bin exclusions occurred in this eligible set. The 12 outside-window bins per line are 00:00 through 05:00 inclusive, plus 23:30. Their statistics are null, not zero. Original source cells remain preserved in the earlier source CSV.

Both retrieved XLSB boarding members have exactly two sheets: **`FICHA_DATOS` and `subidas_prom_laboral`**. They define weekday averages for April 17–21 and August 7–11, 2023, excluding evasion. August's row week label spans August 7–13; its source sheet and title restrict the average to weekdays August 7–11. Weekend observations are **unavailable from these members**, not zero. No other archives were requested for this comparison.

## Evidence and limits

The [provenance](../../data/validation/mut_2023/half_hour_reference/provenance.json) pins the original workbook hashes and the immutable PR #78 source CSV, eligible set, and provenance. The [results metadata](../../data/validation/mut_2023/half_hour_reference/results.json) records coverage and exclusions. All station/bin calculations and exclusions can be reconstructed from those committed sources and the frozen method; they are not duplicated here. Independent source audit checked all 2,208 station/bin rows, 1,584 reference percentages, 96 summaries, and 552 outside-window exclusions.

The full shape is descriptive. It cannot identify shoppers, workers, or trip purpose. Within-Metro interchange counting and line assignment remain unresolved. Two sampled weeks cannot establish parallel trends or isolate seasonal, occupancy, network, economic, or spillover differences. These contrasts do not identify MUT's causal contribution or validate its simulated closure. No model ratio or combined-line statistic was added.
