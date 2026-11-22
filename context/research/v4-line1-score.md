# V4 score · Line 1 interruption, 18 November 2024

The current engine scores **1.0082** on the Line 1 interruption. The no-change reference scores 0.8830. Lower is better. The engine is worse than "no change". The target, cells, and formula come from [the event brief](v4-line1-interruption.md).

## Run

- Engine commit: `22ceabe8689e04a66a774a9e732cc6e15a1ad9c9` (origin/main).
- Binary: `scenario`, release build, SHA-256 `1a4b0974ca6fa6fc50b3fbd931e52ad08928491dd49cf5f3321b30be39d47618`.
- Data: `data/mvp`. Default station weights (L9).
- One command: `python3 tools/v4/line1_score.py [path/to/scenario]` after `cargo build --release --manifest-path engine/Cargo.toml --bin scenario`.

Baseline request: `[]`. Event request (station closures inside half-hours `[27, 29)`, which are 13:30 and 14:00):

```json
[{"kind":"station","target":{"type":"station","id":"SA"},"op":"set","value":0,"half_hours":[27,29]},
 {"kind":"station","target":{"type":"station","id":"MM"},"op":"set","value":0,"half_hours":[27,29]},
 {"kind":"station","target":{"type":"station","id":"PV"},"op":"set","value":0,"half_hours":[27,29]},
 {"kind":"station","target":{"type":"station","id":"GO"},"op":"set","value":0,"half_hours":[27,29]},
 {"kind":"station","target":{"type":"station","id":"AL"},"op":"set","value":0,"half_hours":[27,29]}]
```

The engine applied all five closures. `not_modeled` was empty.

## Result

Score = sum |predicted change − observed change| / 3,368 Thursday stages.

| Station | Bin | Baseline | Event | Predicted change | Observed change |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baquedano (BA-BQ) | 13:30 | 749.5 | 1,317.3 | +567.8 | +536 |
| Baquedano (BA-BQ) | 14:00 | 724.1 | 1,272.8 | +548.7 | +878 |
| Escuela Militar (EM) | 13:30 | 1,175.0 | 1,924.9 | +749.9 | −734 |
| Escuela Militar (EM) | 14:00 | 1,135.3 | 1,859.9 | +724.6 | −826 |

Sum of absolute errors: 3,395.7. Score 3,395.7 / 3,368 = **1.0082**.

- Baquedano: the direction is correct and the 13:30 size is close (error 32). At 14:00 the engine is 329 low.
- Escuela Militar: the sign is wrong. The engine sends the riders of Alcántara and El Golf to Escuela Militar, the nearest open station. The observed entries fell by about 80%. This cell pair gives 89% of the error.
- The five closed stations: 0.0 in both bins. This check passes. It is not part of the score.
- Not scored, for information: the engine adds +1,410 and +1,362 at Los Leones, and +995 and +961 at Tobalaba. Observed Monday-minus-Thursday changes were −655 and −471 (Los Leones), and −1,000 and −931 (Tobalaba).

## Limits

- The engine models a station-only closure. It reassigns the walk/bus access of closed stations to the nearest open stations. It does not stop trains. Line 1 keeps running through the closed segment, so Escuela Militar and Tobalaba stay useful for westbound trips in the model.
- The engine does not disable the Los Leones and Tobalaba Line 1 transfers. It does not model the bus shuttle or the parallel Red service.
- Riders do not choose to skip Metro or change departure time. Only the station of entry changes.
- Baquedano and Tobalaba mix lines (L1 + L5, L1 + L4). The observed counts combine all lines at the station.
- The reference is one Thursday (21 Nov 2024). The engine baseline is a 2026 weekday. Baseline engine entries at Escuela Militar (1,175) are higher than the Thursday count (909).
- The cells are four numbers. One event cannot separate the suspension effect from day-to-day variation.
