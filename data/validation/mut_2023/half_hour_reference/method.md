# Frozen half-hour descriptive line reference method

Frozen before inspecting bin changes, 2026-11-21T09:03:24Z. Reuse the unchanged April and August 2023 boarding XLSB members and the exact eligible station/line set from PR78: TOBALABA plus 26 L1 and 18 L4 reference records. No new eligibility selection, downloads, dependencies, fit, or selected peaks.

Retain all 48 source half-hour labels. Analyze only the fixed 36 bins 05:30 through 23:00. Mark the other 12 bins unavailable for this analysis, preserving any original source cells rather than replacing them with zero.

For each line and fixed bin, compute `100 * (August / April - 1)` for each eligible reference station only when April is numeric and positive and August is numeric. A zero April cell has undefined percentage change; never fill it with zero. Keep the exact per-bin reference membership, numeric values, calculations, valid reference count, and every exclusion reason. Reference membership starts from PR78 and excludes TOBALABA from its own line. A station can be unavailable for a bin without removing it from other bins.

Take the unweighted median reference change for each line/bin. For an even count, average the two middle changes. Calculate the same-bin TOBALABA percentage change only when its April cell is positive and numeric and August is numeric. Subtract the reference median from that target change, in percentage points; use null when either target percentage or reference median is unavailable. Do not combine lines or fit a temporal shape.

Enumerate the actual sheets in each retrieved boarding member. If they contain only FICHA_DATOS and subidas_prom_laboral, mark weekend observations unavailable from these members. Do not infer zero weekend boardings or request other archives.

All shape statements are descriptive. Half-hour peaks cannot identify workers, shoppers, trip purpose, or MUT's causal contribution. The two sampled weeks cannot establish parallel trends; occupancy, seasonal, network, economic, and spillover differences remain unresolved. Save outputs privately, report to root, and obtain independent source audit before any repository publication.
