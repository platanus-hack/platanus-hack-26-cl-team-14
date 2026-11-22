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

## Why Los Leones increases

I ran the release CLI at main `370b090417987dfdf460a0ed516e6cae0a8ecb9e` on four variants of the frozen split. The first edit cuts the line. The remaining five edits close station entrances. Each run applied all its edits. The table gives modeled entries at Los Leones per half-hour. Scores use the same observed changes and Thursday denominators as above.

| Request | 13:30 | 14:00 | Four cells | Eight cells |
| --- | ---: | ---: | ---: | ---: |
| No edits | 1,455.6 | 1,406.5 | 0.8830 | 0.6072 |
| Five station closures | 2,865.2 | 2,768.4 | 1.0082 | 1.1256 |
| Line cut | 807.1 | 779.8 | 0.6415 | 0.2479 |
| Line cut and station closures | 1,588.7 | 1,535.0 | 0.3185 | 0.3464 |

The line cut lowers the 13:30 count by 648.5. Adding station closures reallocates access and adds 781.6 after the same 0.555 reach multiplier. This leaves a net increase of 133.1, while the observed count fell by 655. Station access assigns each point's walking mass across its reachable open stations. When no station is within walking range, the model uses bus access if available. The reach multiplier then scales all the entry mass at each station together. The model has no rider destination for this step. [Access allocation](../../engine/src/station_access.rs) · [Reach calculation](../../engine/src/scenario.rs)

The cut-only control gives a lower eight-cell error and a higher four-cell error than the full request. It leaves the closed station entrances open and is a diagnostic control. The observed combined-line counts and the Thursday reference cannot identify how many people transferred, used the shuttle or chose a different route. [Observations](v4-line1-interruption.md)

Reproduce the four runs from the repository root after building `scenario` in release mode.

```sh
edits=context/research/v4_line1_split.json
for filter in '[]' '.[1:]' '.[:1]' '.'; do
  jq "$filter" "$edits" | engine/target/release/scenario data/mvp - |
    jq '{applied: (.applied | length), stations: [.stations[] |
      select(.id == "LE-LEN" or .id == "TB-TOB" or .id == "BA-BQ" or .id == "EM") |
      {id, entries: .scenario_half_hour[27:29]}]}'
done
```
