# Line 1 split service in the engine (V4 event, 18 Nov 2024)

Owner: bold-boi (V1). Target and score rule: [v4-line1-interruption.md](v4-line1-interruption.md). Input: [v4_line1_split.json](v4_line1_split.json).

## Property changes (no event code)

- `kind:line, op:"cut", target {type:"route", id:"L1"}, properties.stop_ids: [SA, MM, PV, LE-LEN, TB-TOB, GO, AL]`: all 89 Line 1 patterns split into the runs that avoid those stops (San Pablo–Baquedano, Escuela Militar–Los Dominicos). One reroute. `half_hours: [27, 29]` = 13:30–14:30, `temporary: true`.
- Stations Salvador, Manuel Montt, Pedro de Valdivia, El Golf, Alcántara closed (`kind:station`, value 0). Los Leones and Tobalaba stay open for L6 and L4.
- New general rule `metro_reach`: after a line edit splits the Metro graph, each station's entries are multiplied by the share of baseline entries at the other open stations it still reaches. No split → no change.
- Not modeled: the announced bus shuttle and parallel buses.

## Result (engine run, port 8841)

| Station | Bin | Observed Mon − Thu | Predicted event − no edit |
| --- | --- | ---: | ---: |
| Baquedano | 13:30 | +536 | +420 |
| Baquedano | 14:00 | +878 | +406 |
| Escuela Militar | 13:30 | −734 | −1,091 |
| Escuela Militar | 14:00 | −826 | −1,054 |

Score (sum of absolute errors / 3,368 Thursday stages): **0.348**. No-change prediction: 0.883. Lower is better.

Reach: the east section keeps 3–5 % of its baseline destinations (EM 0.044); the rest of the network keeps 88.8 %.

## Limits

- Los Leones and Tobalaba (not scored) go **up** in the model (+1,087 and +702 at 13:30) because closed-station access moves there; observed entries went down (L1 riders left). The access model does not know which line a rider uses at a shared station.
- The Thursday reference is a different day; the observed change is not the effect of the suspension alone.
- A cut section reuses its pattern's departure times at its own first stop.
- The 88.8 % reach applies to every station of the main network inside the window; no observation checks it.
