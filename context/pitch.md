# Pitch and demo

Source of truth for scope. If a moment is not here, the team does not build for it.
Humans own this file. Agents can propose changes in Discord.

- Status: proposal — waiting for human approval

## Problem (one sentence)

Santiago makes city decisions without seeing how they change the day of each person.

## Story (3 minutes or less)

Order and times come from the [rehearsal sheet](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/5b6940fb0be85ecdb7ae862cc7d09a8bfb721d15/context/research/v3-rehearsal.md) (PR #342). Spoken lines are in Spanish.
The 3:00 is a target. The slots add to 180 s with no margin. Cue the three questions before the pitch. Time the full presenter run from the first line to the last.

Each comparison uses two saved replays: one baseline and one experiment. `run_replay` (PR #350) plays one saved result per call. The viewer shows only the newest result, so send the two calls one after the other ([shot order](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/c2cb4326f1767ab8ab9afbe87b93cc1bfdbb2a8c/context/research/v3-ordered-replay-shots.md), PR #355). For each replay:

1. The agent shows the experiment assumptions in the terminal.
2. The terminal hides. The agent calls `run_replay` with the baseline ID. The agent waits until the audience can read the baseline.
3. The agent calls `run_replay` with the experiment ID: same camera, same clock.
4. The chart appears. The terminal comes back.

Replay IDs: pending, until each pair is in `data/replays/`.

| Time | What the audience sees | What must work behind it |
| --- | --- | --- |
| 0:00–0:15 | Santiago map. The camera follows one person on the way to Tobalaba (pending). «Cambiar un lugar cambia el viaje de miles de personas. Probemos tres cambios en Santiago.» | Viewer camera that follows one simulated person ([idea](research/pitch-storyboard.md)). Terminal visible in the map view. |
| 0:15–1:10 | Question: swap the MUT and the Estadio Monumental. Colo-Colo plays at the MUT, and the modeled jobs of the two blocks change places. Terminal shows assumptions: the job swap of the two blocks, 39.222 asistentes, same fans, end 17:00. Baseline: match at the Monumental. Experiment: match at the MUT with the job swap. Chart 1: «asistentes estimados por modo», public transport / car / other. Chart 2: weekday Metro entries at Tobalaba, Pedrero and Mirador, each station alone. | `run_replay` baseline ID, then full-swap experiment ID ([storyboard](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/a779f9d1c401aeb66c7596f01f87005f5c9fe714/context/research/v3-mut-replay.md), PR #340; [swap decision](decisions/20261121-1315-mut-monumental-swap.md)). Chart 1 reads `events[0].fan_base.by_mode`. Chart 2 reads `asked_stations[].scenario_daily`: it shows the job swap, because the fan chart cannot. |
| 1:10–1:45 | Question: cross Santiago by boat on the Mapocho. Assumptions: 10 piers, 17 km/h, a boat every 10 min, 07:00–10:00. Baseline: no boats. Experiment: boats. Camera: full river, then one pier and the walk to it. Chart: people per hour and delay in minutes, with and without boats. | `run_replay` baseline ID, then experiment ID ([storyboard](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/6a9db39de7b124c3494fe7f266dc84aafaa11efb/context/research/mapocho/replay-storyboard.md), PR #338). V1 ferry service properties. Flow and delay outputs from the engine. |
| 1:45–2:25 | Question: close the full Metro, 07:00–10:00, weekday. Baseline: normal service at Tobalaba, 08:00. Experiment: full closure, same frame, clock restarts at 08:00. The Metro lines fade. Chart: trips by mode, before and after. Then the panel «Santiago, octubre de 2019»: the replay active-station count next to the real count of zero on 19–20 October. | `run_replay` baseline ID, then closure ID ([storyboard](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/09595af1eb30b0358783117d551070e9e2fd2dfa/context/research/metro-closure-replay.md), PR #341). V1 time-bounded Metro service change. Historical panel from the DTPM 2019 report. |
| 2:25–2:50 | Side by side: ‹N› simulations (pending). Each panel has a scenario name and a unit. | Comparison view that reads only cached results. ‹N› is open ([verticals](verticals.md)). |
| 2:50–3:00 | Hold the final view. «Antes de cambiar Santiago, podemos ver a quién le cambia el viaje.» | Nothing new. |

## The wow moment

**1:45–2:25: the Metro lines fade at 08:00. Then the screen shows «Santiago, octubre de 2019» with zero active stations.**

Why this moment:

- Everyone in the room remembers the closure. It is the most emotional scene.
- It is the only scene where the simulation meets a real event. Metro announced the full closure at 19:20 on Friday, 18 October 2019. On Monday, 21 October, 20 Line 1 stations reopened ([DTPM 2019 report, ch. 7](https://www.dtpm.cl/descargas/memoria/InformeGestion_2019_DTPM.pdf#page=56), cited in `context/research/metro-closure-replay.md`).
- `context/verticals.md` requires this comparison in the demo.

Limit: the replay is a weekday morning. The real closure covered a weekend, with curfews and bus changes. Label the historical panel as a check of service availability only. Label trips, mode shifts, and congestion as model estimates.

## What "looks like the mock" means

- Each scenario runs from the typed question to the final chart. The terminal shows the assumptions first. It hides during playback and comes back at the end.
- Playback makes zero simulation calls. The MCP trace shows only cache reads.
- Each comparison plays the baseline first, holds it, then plays the experiment.
- The two runs of a scenario use the same frame, the same clock, and the same scale.
- Every chart shows a unit and a time window. A person at the back of the room can read it.
- In the Metro scene, the lines fade at 08:00. The historical panel shows the 2019 dates.
- No car moves on the river. No boat moves on a street.
- The full run, from the first line to the last, takes 3:00 or less.

## Figures and sources

The storyboard files are not on `main` yet. Use the pinned links in the story table.

| Figure | Source | State |
| --- | --- | --- |
| 39.222 asistentes, Superclásico, 31 Aug 2025 | `context/research/monumental-match-attendance.md` | Input, not a result. Two match reports. No official open series. |
| 23.688 → 27.790 asistentes en transporte público (Monumental → MUT) | `context/research/v3-mut-replay.md` (test in PR #323) | (sin verificar). The test changed only the venue. Recalculate with the full swap before use. |
| 10 piers, 17 km/h, a boat every 10 min | `context/research/mapocho/replay-storyboard.md`, `context/research/mapocho/piers.json` | Assumptions, not results. |
| 10.871 de 23.941 viajes cerca de muelles ahorrarían tiempo (45,4 %), EOD 2012 | `context/research/mapocho/boat_trips.md` | Full weekday, not 07:00–10:00. Keep it out of the replay chart. The comuna-point comparison found a longer boat trip on all 42 pairs. Say this limit if you quote the figure. |
| Closure announced at 19:20, 18 Oct 2019; 20 Line 1 stations reopen on 21 Oct | `context/research/metro-closure-replay.md` (DTPM 2019 report) | Official record. |
| Weekday Metro entries at Tobalaba, Pedrero and Mirador, baseline and full swap | `context/research/v3-mut-replay.md` (field `asked_stations[].scenario_daily`) | (sin verificar). No run with the full swap yet. The engine does not link a loss at one station to a gain at another. |
| 0 active stations, 19–20 Oct 2019 | `context/research/metro-closure-replay.md` | Official record. The replay count is (sin verificar). |
| Replay outputs: modes, people per hour, delay, trips by mode | none yet | (sin verificar). No cached replay exists. |
