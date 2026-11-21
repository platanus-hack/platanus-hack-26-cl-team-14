# Jury question stress test

Twelve unprepared questions went through the real MCP server (`tools/santiago-mcp/server.mjs`, stdio) to the engine on main `dfe3124` with the point-target fix. Each call took 120–300 ms. The agent translation is the tester's, as a jury-side agent would do it.

| Question | Changes | Result the agent reads (daily Metro boardings) |
| --- | --- | --- |
| Script: Estación Central residents ×1.5, Las Condes jobs ×1.5 | 2 comuna changes | Manquehue +21,721; Universidad de Chile −1,284 (same as the pitch number) |
| Close Tobalaba | station set 0 | Tobalaba −39,824; El Golf +19,049; Los Leones +16,422 |
| New mall in Puente Alto, 5,000 jobs | point add | Plaza de Puente Alto +4,102 |
| 10,000 new apartments in Quilicura | comuna residents add | Plaza Quilicura +981 (3,778 new workers at the comuna rate) |
| Remote work: Las Condes loses 30% of jobs | comuna jobs ×0.7 | Manquehue −13,552; centre stations up about +1,000 each |
| Highway: Maipú → Santiago 15 min faster | pair minutes −15 | Universidad de Chile +2,910; Plaza Maipú −2,645 |
| Remove the MUT by coordinates (−70.6017, −33.4180) | point jobs set 0 | The nearest-point rule took block 13114131008007 (35 m, 5,215 jobs), **not** the registered MUT block 13114131009033: Tobalaba −3,432 is for that neighbour block. For the MUT, use the named place `mut` (`data/mvp/named_places.csv`): Tobalaba −2,547.9, 4,681 workers moved |
| Concert, 60,000 at Estadio Nacional | event | `events[]` only; station deltas stay 0 by design |
| New Metro station in Lo Barnechea | station at a point | `not_modeled`, with a clear reason |
| Mix: close Baquedano, Providencia jobs ×1.2, Puente Alto–Santiago +10 min | 3 changes | Baquedano −20,727; Salvador +11,327 |
| Station name with accents ("Estación Central") | station set 0 | Matched; Estación Central −18,848 |
| Empty list | — | All zero |

## Defect found and fixed

A point target matched only a point with the same coordinates to 6 decimals. Any other point became a new place with 0 jobs. So "remove / reduce this existing place" at the agent's coordinates was in `applied` but did nothing. Now, if the change gives 0 on an empty point (`set 0`, `multiply`, `add` ≤ 0), the engine uses the nearest existing point within 500 m. `applied[].understood` shows the block it took (`block`, `block_distance_m`) and `jobs_before`. The nearest block is not always the named place: for a named place, use `named_places.csv`. New buildings keep the new-point rule.

## Limits for the presenter

- A concert or a match changes no station bar and no city height. The answer comes only from `events[]`.
- "Remove a place" takes one block point. A large complex can span more blocks than the one the engine takes.
- New lines or stations are not modeled. The agent must say so.
