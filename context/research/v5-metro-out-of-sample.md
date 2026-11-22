# V5 out-of-sample check · Line 5 at Baquedano, Tuesday 19 November 2024

Status: cells frozen. No engine run and no DTPM count read yet for this event.

## Event

- Tue 19 Nov 2024. 13:08: Metro reported Baquedano (L5), direction Vicente Valdés, without train stops. Cause given to users: water leak and cleaning. 14:33: Metro reported Baquedano L5 operating in both directions. Line 1 at Baquedano is not reported as affected. [CNN Chile, read 21 Nov 2026](https://www.cnnchile.com/pais/metro-baquedano-linea-5-sin-detencion-trenes-filtracion-de-agua/)
- Bins fully inside the window: 27 (13:30) and 28 (14:00).
- Control: Thu 21 Nov 2024, same DTPM release, the same reference as the V4 Line 1 check.

## Frozen cells (written before any count or run)

- Scored: BA-BQ, BE (Bellas Artes), PB (Parque Bustamante) × bins 27, 28. Six cells.
- Observed change = Tue 19 entries − Thu 21 entries (DTPM etapas, METRO, `parada_subida` × `media_hora_subida`, local date).
- Score = sum |predicted − observed| / sum Thursday entries in the six cells. No-change reference = sum |observed| / same denominator. Lower is better.
- Noise check: Tue − Thu at the same three stations in bins 22–25 (11:00–13:00) and 30–33 (15:00–17:00). The event is detectable only if both BA-BQ event-bin changes are outside the range of the BA-BQ noise-bin changes. If not, the score is reported as not informative.

## Requests (written before the run)

- Primary, the V4 mechanism: `kind:line, op:cut` on route L5 at `BA-BQ`, `temporary`, `half_hours [27, 29]`. No station closure, because Line 1 still served Baquedano. File: [v5_baquedano_l5_cut.json](v5_baquedano_l5_cut.json). This splits L5 in both directions, so it removes through trains that really ran. It overstates the event.
- Sensitivity, closer to the event: `op:set` on the 34 L5 direction-0 patterns (toward Vicente Valdés), with Baquedano removed from the stop list and the two adjacent segment minutes added. Same window. File: [v5_baquedano_l5_skip.json](v5_baquedano_l5_skip.json).
- Baseline: `[]`. Rule parameters unchanged (main 7ff285d, includes #502).
