# Generalist cases

Small cases outside the MUT ↔ Monumental swap, to check that the engine is general (Mac, 16:16 UTC). Main `acdd1c3`, release engine, 200–300 ms each. Values are modeled weekday Metro boardings. Requests are the exact `changes` lists.

## The engine can represent

| Case | `changes` | Result |
| --- | --- | --- |
| 3,000 new homes in Puente Alto | `[{"kind":"residents","target":{"type":"comuna","id":"13201"},"op":"add","value":9000}]` | +3,157 workers, +2,627 boardings; Plaza de Puente Alto +340 |
| Office tower, 8,000 jobs, Quilicura | `[{"kind":"jobs","target":{"type":"point","longitude":-70.7302,"latitude":-33.3586},"op":"add","value":8000}]` | 9,712 workers change job; Plaza Quilicura +6,377 |
| Congestion: trips to and from Santiago +10 min | `[{"kind":"minutes","target":{"type":"comuna","id":"13101"},"op":"add","value":10}]` | 160,278 workers change job, −36,271; U. de Chile −8,621, Plaza de Armas −7,270 |
| Concert, 45,000, Estadio Nacional, ends 23:00 | `[{"kind":"event","target":{"type":"point","longitude":-70.61064,"latitude":-33.46447},"op":"set","value":45000,"assumed_end_half_hour":46}]` | `events[]` only: 27,928 by public transport, 8,861 Metro users with access |
| Same concert, 15,000, Movistar Arena | same, point −70.661845, −33.462739 | 3,045 Metro users with access |

An event does not change station bars or trip groups: its estimate is separate by design.

## The engine cannot represent yet

**More telework, or more people at home.** `kind:"telework"` goes to `not_modeled`. The nearest supported change, jobs ×0.8 in Las Condes, Providencia and Santiago, is wrong for this question: the engine keeps every worker and sends 115,451 of them to jobs in other comunas (Plaza Maipú +1,643). Telework removes trips; it does not move jobs.

Baseline for a `stay_home` change (EOD 2012, normal weekday, persons who work full or part time, 14,691 persons, 2.75 M expanded): 21.7 % make no trip to work that day (full time 18.8 %, part time 41.6 %); 3.1 % stay home "por trabajo". The engine's workers are the Census usual-commute universe and all travel every day, so a share of workers who stay home is a lever with default 0. Proposed in cut 5; not built.

**Household car ownership** (`kind:"cars"`) is cut 5 link 3, branch `link3-car-ownership`, waiting for the `scenario.rs` wiring.
