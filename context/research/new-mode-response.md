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

- Social value of time for appraisal, CLP per passenger-hour (MDSF): 2025 table **3,126 in vehicle, 6,252 waiting, 6,252 walking** ([Precios Sociales 2025](https://sni.gob.cl/wp-content/uploads/Precios_Sociales_2025.pdf)); 2026 table **3,338 and 6,676** ([Precios Sociales 2026](https://sni.gob.cl/wp-content/uploads/Precios-Sociales-2026.pdf)). These are appraisal prices, not estimated choice parameters; the 2× weight on waiting and walking is a scenario assumption.
- Ferry preference from a comparable case: existing Sydney Ferries riders valued ferry over bus at about 17 in-vehicle minutes in the peak (stated preference, [Daniels & Streeting 2002](https://australasiantransportresearchforum.org.au/wp-content/uploads/2022/03/2002_Daniels_Streeting.pdf)). Use it only as a sensitivity input; it gives no Santiago value or bound.

## What our engine already does

`engine/src/choice.rs`: per comuna pair, gravity for the work place (jobs × e^(−β·minutes)) and a 3-way logit for mode (public / car / other). Residual factors reproduce the Census 2024 flows at baseline. A `minutes` change moves both choices: `choice.rs:41–53` uses the same comuna-pair public-transport minutes in job gravity and in the public-mode utility. Car is the reference mode and "other" uses distance. So `minutes` changes the existing public mode; there is no separate alternative for a new mode, no mode constant for it, and no separate wait and walk weights.

## Limits that matter for the pitch cases

- **Scale.** Access is measured from the comuna point. A mode that serves a thin corridor (a river boat) loses to the walk from the comuna point. Test in [the Mapocho brief](mapocho-navegable.md#7-second-engine-test-the-boat-as-faster-travel-minutes): 0 of 42 pairs faster.
- **New stations.** The station projection treats a new point like Metro (L9 access). Good for Metro lines, wrong for a slower, smaller mode.
- **Moved place (MUT ↔ Monumental).** Jobs move through gravity; event attendees come from workers of the venue comuna (`attendance.rs`), so moving a venue changes who attends. Published models keep the attendee origin fixed and change only the trip cost.

## Smallest next step (proposal, not built)

A separate new-mode choice needs a fourth alternative, available only for eligible trips (for a river line: trips near piers), with its own time, constant and wait/walk weights. This extends the logit already in `choice.rs`. Owner: the `scenario.rs` writer (Lucila-42), if the team wants case 2 in the engine.
