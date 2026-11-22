# V5 out-of-sample check · Line 5 at Baquedano, Tuesday 19 November 2024

Result: **not informative.** The observed change at Baquedano is inside the normal Tuesday-to-Thursday noise. Both model variants score worse than "no change": the cut scores 0.235, the one-direction skip 0.067, and no change 0.046. The cells and the requests were frozen and pushed in commit 5034c89 before any count was read or any engine run was made.

## Event

- Tue 19 Nov 2024, 13:08: Metro reported Baquedano L5, direction Vicente Valdés, "sin detención de trenes". Cause given to users: water leak and cleaning. 14:33: Metro reported Baquedano L5 operating in both directions. No source reports Line 1 at Baquedano or the L1–L5 transfer as affected. [CNN Chile, embedded @metrodesantiago post, read 21 Nov 2026](https://www.cnnchile.com/pais/metro-baquedano-linea-5-sin-detencion-trenes-filtracion-de-agua/)
- Bins fully inside the window: 27 (13:30) and 28 (14:00).
- Control: Thu 21 Nov 2024, the same DTPM release and the same reference as the V4 Line 1 check.

## Search for other events (checked 21 Nov 2026)

DTPM publishes dated etapas for these weeks: Apr and Aug 2022, Apr, Aug and Nov 2023, Apr and Nov 2024, Apr 21–27 and Nov 8–14 2025, Apr 11–17 2026. [DTPM catalog](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje)

| Sampled week | Weekday incidents found in news |
| --- | --- |
| 18–24 Nov 2024 | Mon 18: Line 1 split (V4, fitted, excluded). Tue 19: this event. Wed 20: Line 1 technical faults, no closure. |
| 21–27 Apr 2025 | None. The nearest were 7 Apr (Line 1) and 16 Apr (Line 5), both outside the week. |
| 8–14 Nov 2025 | None. The nearest were 3–4 Nov and 17–18 Nov, outside the week. |
| 11–17 Apr 2026 | None. The nearest were 5–6 Apr and 28 Apr (Line 4), outside the week. |
| 6–12 Nov 2023 | None. Line 2 closed on 13 Nov, one day after the week. |

News search is not complete. A scan of the DTPM counts for zero-entry stations in each week would be complete, but each week is about 2 GB at about 1 MB/s.

## Frozen cells

- Scored: BA-BQ, BE (Bellas Artes), PB (Parque Bustamante) × bins 27 and 28. Six cells.
- Observed change = Tue 19 entries − Thu 21 entries (DTPM etapas, `METRO`, local date in `tiempo_subida`, grouped by `parada_subida` × `media_hora_subida`).
- Score = sum |predicted − observed| / 2,605 Thursday entries in the six cells. The no-change reference uses sum |observed| with the same denominator. Lower is better.
- Noise check: Tue − Thu at Baquedano in bins 22–25 and 30–33. The event counts as detectable only if both event-bin changes are outside that range.

## Observed and predicted

Engine: main 7ff285d (includes #502). Rule parameters are unchanged. Baseline request: `[]`. Both requests applied all changes. `not_modeled` was empty.

| Station | Bin | Thu 21 | Tue 19 | Observed | Cut (primary) | Skip (sensitivity) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Baquedano | 13:30 | 706 | 702 | −4 | −103.3 | −45.5 |
| Baquedano | 14:00 | 712 | 650 | −62 | −99.7 | −43.9 |
| Bellas Artes | 13:30 | 228 | 225 | −3 | −111.6 | −33.2 |
| Bellas Artes | 14:00 | 246 | 261 | +15 | −107.8 | −32.1 |
| Parque Bustamante | 13:30 | 334 | 338 | +4 | −138.7 | 0.0 |
| Parque Bustamante | 14:00 | 379 | 346 | −33 | −134.0 | 0.0 |

| Score (6 cells, /2,605) | Value |
| --- | ---: |
| No change | **0.0464** |
| Cut: L5 cut at BA-BQ, both directions ([request](v5_baquedano_l5_cut.json)) | 0.2350 |
| Skip: Baquedano removed from 34 L5 patterns toward Vicente Valdés ([request](v5_baquedano_l5_skip.json)) | 0.0668 |

- Noise check fails. Baquedano Tue − Thu in the noise bins: +6, +24, −1, −34 (11:00–13:00) and −22, +64, −64, −13 (15:00–17:00), so the range is −64 to +64. The event-bin changes (−4, −62) are inside that range. One direction skipping one line at a shared L1 + L5 station does not show in the station totals.
- Reach at 13:30: the cut gives Baquedano 0.862, Bellas Artes 0.869 and Parque Bustamante 0.764. The skip gives 0.939, 0.961 and 1.0.
- One weak result is still usable. The cut predicts about −100 at Baquedano, which is outside the noise range. The counts do not show that drop. This agrees with the expected overstatement: the cut removes L5 through trains that really ran. For the skip, every predicted change is inside the noise range.
- The Thursday file matches the V4 hash (`4a8fef77…`, 2,307,896 Metro rows), and Baquedano matches the V4 Thursday cells (706, 712). The Tuesday member `2024-11-19.etapas.csv.gz` has SHA-256 `afbc70d3cf040a28c3b9c4f611f11f80de77e65a397ed66c6abef5d95eb1a882` and 2,338,836 Metro rows.

## Limits

- One event that is too small to detect cannot validate or reject the reach rule. This is a negative result for the out-of-sample test.
- The cut (the V4 mechanism) splits L5 in both directions. The real event stopped only trains toward Vicente Valdés, and they passed through. The skip is closer to the event. `metro_minutes` ignores `pickup_type`, so a skip needs a changed stop list.
- Station counts combine all lines at a station. Baquedano mixes L1 and L5.
- The engine baseline is a 2026 weekday. At 13:30 it gives Bellas Artes 849 entries where Thursday 2024 counted 228. Predicted changes scale with this baseline.
- The control is one Thursday. The sources do not say if the direction toward Plaza de Maipú also stopped between 13:08 and 14:33.
- Case file in the `tools/v5/line1.json` format, for `run_evals.py` later: [v5_baquedano_l5_case.json](v5_baquedano_l5_case.json).
