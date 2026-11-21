# Pitch and demo

Source of truth for scope. If a moment is not here, the team does not build for it.
Humans own this file. Agents can propose changes in Discord.

- Status: proposal — waiting for human approval

## Problem (one sentence)

Santiago makes city decisions without seeing how they change the day of each person.

## Story (3 minutes or less)

Order and times come from the [rehearsal sheet](research/v3-rehearsal.md) (PR #342). Spoken lines are in Spanish.
The MCP replay tools (`list_replays`, `run_replay`) are planned in [V3](verticals.md). They are not in `tools/santiago-mcp/server.mjs` yet.

| Time | What the audience sees | What must work behind it |
| --- | --- | --- |
| 0:00–0:15 | Santiago map. The camera follows one person on the way to Tobalaba. «Cambiar un lugar cambia el viaje de miles de personas. Probemos tres cambios en Santiago.» | Viewer camera that follows one simulated person ([idea](research/pitch-storyboard.md)). Terminal visible in the map view. |
| 0:15–1:10 | Question: Colo-Colo plays at the MUT. Terminal shows assumptions: 39.222 asistentes, same fans, end 17:00. Camera: Monumental, then MUT, same frame. Final chart: two stacked bars, public transport / car / other. | MCP `run_replay` opens the cached MUT replay: two `simulate_city` runs ([storyboard](research/v3-mut-replay.md), PR #340). Chart reads `events[0].fan_base.by_mode`. |
| 1:10–1:45 | Question: cross Santiago by boat on the Mapocho. Assumptions: 10 piers, 17 km/h, a boat every 10 min, 07:00–10:00. Camera: full river, then one pier and the walk to it. Chart: people per hour and delay in minutes, with and without boats. | MCP `run_replay` opens the cached Mapocho pair ([storyboard](research/mapocho/replay-storyboard.md), PR #338). V1 ferry service properties. Flow and delay outputs from the engine. |
| 1:45–2:25 | Question: close the full Metro, 07:00–10:00, weekday. Scene at 08:00: the Metro lines fade, zoom to Tobalaba. Chart: trips by mode, before and after. Then the panel «Santiago, octubre de 2019». | MCP `run_replay` opens the cached closure ([storyboard](research/metro-closure-replay.md), PR #341). V1 time-bounded Metro service change. Historical panel from the DTPM 2019 report. |
| 2:25–2:50 | Side by side: ‹N› simulations. Each panel has a scenario name and a unit. | Comparison view that reads only cached results. ‹N› is open ([verticals](verticals.md)). |
| 2:50–3:00 | Hold the final view. «Antes de cambiar Santiago, podemos ver a quién le cambia el viaje.» | Nothing new. |

## The wow moment

**1:45–2:25: the Metro lines fade at 08:00. Then the screen shows «Santiago, octubre de 2019».**

Why this moment:

- Everyone in the room remembers the closure. It is the most emotional scene.
- It is the only scene where the simulation meets a real event. Metro closed the full network at 19:20 on Friday, 18 October 2019. On Monday, 21 October, 20 Line 1 stations reopened ([DTPM 2019 report, ch. 7](https://www.dtpm.cl/descargas/memoria/InformeGestion_2019_DTPM.pdf#page=56), cited in `context/research/metro-closure-replay.md`).
- `context/verticals.md` requires this comparison in the demo.

Limit: the replay is a weekday morning. The real closure covered a weekend, with curfews and bus changes. Label the historical panel as a check of service availability only. Label mode shifts and congestion as model estimates.

## What "looks like the mock" means

- Each scenario runs from the typed question to the final chart. The terminal shows the assumptions first. It hides during playback and comes back at the end.
- Playback makes zero simulation calls. The MCP trace shows only cache reads.
- The two runs of a scenario use the same frame, the same clock, and the same scale.
- Every chart shows a unit and a time window. A person at the back of the room can read it.
- In the Metro scene, the lines fade at 08:00. The historical panel shows the 2019 dates.
- No car moves on the river. No boat moves on a street.
- The full run, from the first line to the last, takes 3:00 or less.

## Figures and sources

| Figure | Source | State |
| --- | --- | --- |
| 39.222 asistentes, Superclásico, 31 Aug 2025 | `context/research/monumental-match-attendance.md` | Two match reports. No official open series. |
| 23.688 → 27.790 asistentes en transporte público (Monumental → MUT) | `context/research/v3-mut-replay.md` (test in PR #323) | (sin verificar). The test changed only the venue. Recalculate with the full swap before use. |
| 10 piers, 17 km/h, a boat every 10 min | `context/research/mapocho/replay-storyboard.md`, `context/research/mapocho/piers.json` | Assumptions, not results. |
| 10.871 de 23.941 viajes cerca de muelles ahorrarían tiempo (45,4 %), EOD 2012 | `context/research/mapocho/boat_trips.md` | Full weekday, not 07:00–10:00. Keep it out of the replay chart. The comuna-point comparison found a longer boat trip on all 42 pairs. Say this limit if you quote the figure. |
| Closure at 19:20, 18 Oct 2019; 20 Line 1 stations reopen on 21 Oct | `context/research/metro-closure-replay.md` (DTPM 2019 report) | Official record. |
| Replay outputs: modes, people per hour, delay, trips by mode | none yet | (sin verificar). No cached replay exists. |
