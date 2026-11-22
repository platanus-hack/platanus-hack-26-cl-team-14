# Line 1 split service in the engine (V4 event, 18 Nov 2024)

Owner: bold-boi (V1). Target and score rule: [v4-line1-interruption.md](v4-line1-interruption.md). Input: [v4_line1_split.json](v4_line1_split.json).

## Property changes (no event code)

- `kind:line, op:"cut", target {type:"route", id:"L1"}, properties.stop_ids: [SA, MM, PV, LE-LEN, TB-TOB, GO, AL]`: all 89 Line 1 patterns split into the runs that avoid those stops (San Pablo–Baquedano, Escuela Militar–Los Dominicos). One reroute. `half_hours: [27, 29]` = 13:30–14:30, `temporary: true`.
- Stations Salvador, Manuel Montt, Pedro de Valdivia, El Golf, Alcántara closed (`kind:station`, value 0). Los Leones and Tobalaba stay open for L6 and L4.
- General rule `metro_reach`: after a Metro line edit, each station's entries are multiplied by its remaining accessibility on the operating Metro graph: other open stations weighted by baseline entries and exp(−β × scheduled minutes), over the same sum on the baseline graph. β is the job-choice coefficient that `choice.rs` fits on Census 2024 commute flows (not fitted on this event). Minutes are arrival minus departure along one GTFS pattern (dwell counts only at stops passed on board). Changing trains at a station costs an assumed 2 minutes (about half a peak headway), set before the score was run; it is longer than any dwell, so staying on board is never cheaper by a fake transfer. No Metro edit → no change.
- Not modeled: the announced bus shuttle and parallel buses.

## Result (engine run, port 8841)

Predicted event − no edit, bins 13:30 / 14:00; observed Mon 18 − Thu 21 Nov 2024.

| Station | Observed | Predicted |
| --- | ---: | ---: |
| Baquedano | +536 / +878 | +336 / +325 |
| Escuela Militar | −734 / −826 | −956 / −924 |
| Los Leones | −655 / −471 | +133 / +129 |
| Tobalaba | −1,000 / −931 | −483 / −467 |

| Score (sum of absolute errors / Thursday stages; lower is better) | No change | Connectivity reach (#473) | Time-weighted reach |
| --- | ---: | ---: | ---: |
| 4 cells (BA, EM), denominator 3,368 | 0.883 | 0.348 | **0.319** |
| 8 cells (+ LE, TB), denominator 9,933 | 0.607 | 0.780 | **0.346** |

Reach at 13:30: Baquedano 0.824, Escuela Militar 0.114, Los Leones 0.555, Tobalaba 0.43, Los Dominicos 0.155.

## Limits

- Los Leones still goes up (+133) where it went down (−655): closed-station access moves there, and the access model does not know which line a rider uses at a shared station.
- The Thursday reference is a different day; the observed change is not the effect of the suspension alone.
- A cut section reuses its pattern's departure times at its own first stop.
- The first wait is not in the reach minutes; the 2-minute transfer is an assumption.
