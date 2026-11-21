# V3 · Three-minute rehearsal

Run of show from the [pitch proposal](../pitch.md). Start the clock on the first spoken line and stop it on the last. The slots total 180 seconds with no margin. Include typing, loading, and transitions in each slot.

| Pitch time | Screen cue | Presenter cue |
| --- | --- | --- |
| 0:00–0:12 | Follow one person through Santiago. Keep the terminal visible. | “Cambiar un lugar cambia el viaje de miles de personas. Probemos tres cambios en Santiago.” |
| 0:12–1:02 | Ask about replacing the MUT with the Estadio Monumental. Show assumptions, then the city change and the two compact charts. | Read one change from the chart with its unit. |
| 1:02–1:35 | Ask about navigating the Mapocho. Show assumptions, then the river replay and its edge chart. | Point out the walk to the pier. Read one result from the replay with its unit. |
| 1:35–2:15 | Ask about closing the full Metro. Show assumptions, then the city change and the October 2019 comparison. | Name the actual footprint and dates in the historical comparison. |
| 2:15–2:33 | Show the V4 evaluation slide. | Give the prediction, observation, and score when verified. |
| 2:33–2:52 | Open the final side-by-side view. Give each panel a scenario name and a visible unit. Mark the comparison scale and window where they differ. | Point to one result per panel. |
| 2:52–3:00 | Hold the final view. | “Antes de cambiar Santiago, podemos ver a quién le cambia el viaje.” |

The number of side-by-side panels is open. Keep the 19-second slot and use the count the team approves.

## Rehearse the full run

1. Before rehearsal, record six saved result IDs. For each scenario, list the baseline first and the experiment second. Record each engine revision and the data used for each chart. Check that the cache lists all six IDs.
2. Keep the cache reader and presenter view running. Run all three prompts in order. Check the MCP trace and HTTP request log for cache reads and zero simulation calls. Keep any process that also serves the cache running for this check.
3. For each prompt, check that the terminal shows the planned edits, start time, duration, and measures before playback. Check that the terminal hides during playback and returns after the final chart. Let the city change play before revealing the chart. Check that the edge chart keeps the affected area visible. Hold each baseline until it appears on screen, then publish its experiment.
4. Check the final side-by-side view from projector distance. Match each displayed result to the cached output. Check the units, time window, and label for the historical Metro comparison.
5. Record the actual start and end time for every row. If the run exceeds 3:00, trim narration or holds and repeat the entire run. Record the final duration, replay IDs, and any visible defect in the V3 thread.

The three preview bundles provide cached results. The final build still needs a complete timed run and a verified comparison view.
