# How models simulate a new transport mode or a moved place

Brief for [Mac's research request (16:30 UTC)](https://discord.com/channels/1573514576400752640/1573561159955648553/1573737458426970134), axis "response to a new mode or a moved place". Writer: Marcia-44.

## What existing models do

| Model | How a new mode enters | Source |
| --- | --- | --- |
| Discrete choice (logit, nested logit) | Utility of each mode = constant (ASC) + β·in-vehicle time + β·wait + β·walk + β·cost. A new mode is a new alternative with its own constant, put in a nest with similar modes. | [ActivitySim tour mode choice](https://activitysim.github.io/activitysim/develop/dev-guide/components/tour_mode_choice.html) |
| ActivitySim | Nested logit at tour and trip level. Calibration adjusts the constants until modeled shares match observed shares. | [UDOT report UT-24.16](https://rosap.ntl.bts.gov/view/dot/77610/dot_77610_DS1.pdf) |
| MATSim | Each mode scores constant + β·time + β·distance + β·money. A new mode gets its own constant; agents replan over iterations. | [MATSim user guide §6.3](https://svn.vsp.tu-berlin.de/repos/public-svn/publications/vspwp/2014/14-20/user-guide-0.6.0-2014-09-12.pdf/), [Melbourne model](https://www.sciencedirect.com/org/science/article/pii/S1547245024000306) |
| ESTRAUS (Santiago, SECTRA) | Hierarchical logit for mode split, with distribution and assignment in equilibrium. Constants per mode, time and cost parameters. Values not read here. | [ESTRAUS update report](https://www.u-cursos.cl/ingenieria/2010/1/CI63D/1/material_docente/bajar?id_material=281298) |

Consensus: a new mode changes choices only through **time, cost and a mode constant**. Without a constant estimated from data, the model treats the new mode as a faster or slower version of an existing mode.

## Santiago values to use

- Social value of time 2025 (MDSF, Table 2.1): **3,126 CLP/h in vehicle, 6,252 CLP/h waiting, 6,252 CLP/h walking**. Waiting and walking count double ([Precios Sociales 2025](https://sni.gob.cl/wp-content/uploads/Precios_Sociales_2025.pdf)).
- Ferry constant from a comparable case: Brisbane ferry users valued ferry over bus at about 17 in-vehicle minutes in the peak ([Daniels & Streeting 2002](https://australasiantransportresearchforum.org.au/wp-content/uploads/2022/03/2002_Daniels_Streeting.pdf)). Stated preference from current users: an upper bound.

## What our engine already does

`engine/src/choice.rs`: per comuna pair, gravity for the work place (jobs × e^(−β·minutes)) and a 3-way logit for mode (public / car / other). Residual factors reproduce the Census 2024 flows at baseline. A `minutes` change moves both choices. This is the same mechanism as the models above, at comuna scale, without a separate constant per new mode and without separate wait and walk weights.

## Limits that matter for the pitch cases

- **Scale.** Access is measured from the comuna point. A mode that serves a thin corridor (a river boat) loses to the walk from the comuna point. Test in [the Mapocho brief](mapocho-navegable.md#7-second-engine-test-the-boat-as-faster-travel-minutes): 0 of 42 pairs faster.
- **New stations.** The station projection treats a new point like Metro (L9 access). Good for Metro lines, wrong for a slower, smaller mode.
- **Moved place (MUT ↔ Monumental).** Jobs move through gravity; event attendees come from workers of the venue comuna (`attendance.rs`), so moving a venue changes who attends. Published models keep the attendee origin fixed and change only the trip cost.

## Smallest next step (proposal, not built)

Add an optional per-mode constant and a wait/walk weight to the `minutes` change, so a `minutes` change can say "a slower mode that people like" (ASC) or "a faster mode with long waits". This reuses the logit already in `choice.rs`. Owner: the `scenario.rs` writer (Lucila-42), if the team wants case 2 in the engine.
