# Metro closure only from 07:00 to 10:00

Peumo owns the V1 source. Stacked on bold-boi's [full closure](metro-closure-choice.md). Boundary rule: Eliseo-21, V1 thread message 1573813050232938691.

Add `"half_hours": [14, 20]` to a change (input: [metro_closure_window.json](metro_closure_window.json)). The window is `[start, end)` in half-hours from 00:00.

- Rule: a worker whose normal-weekday departure half-hour is inside the window faces the closed network for the whole trip. Other workers keep their baseline mode. The departure comes from the EOD 2012 profile of the worker's mode (`work_departures.csv`), at a quantile per person from a SplitMix64 hash of its index, independent of the home and job samplers.
- Station entries change only inside the window. Outside it they keep the baseline.
- Output `closure_window`: workers who leave home in the window, by mode, base and scenario. Label: "trabajadores que salen entre 07:00 y 10:00".

The `scenario` binary at commit `89beed10a3d63dbd3f93fb8e9dddcc820b765a8d` ran the standard `metro_closure_window.json` fixture against `data/mvp`. The fixture closes Metro from 07:00 to 10:00 and includes no residents edit.

| Workers leaving 07:00–10:00 | Baseline | Metro closed |
| --- | ---: | ---: |
| Public transport | 772,186 | 662,807 |
| Car | 504,342 | 591,740 |
| Walk | 92,737 | 94,607 |
| Bicycle | 68,637 | 74,138 |
| Other | 49,553 | 64,163 |
| Total | 1,487,455 | 1,487,455 |

Modeled weekday Metro entries fell from 2,457,348.4 to 1,748,979.3. Tobalaba: zero in bins 14–19, baseline in the other bins.

The window does not show reopening at 10:00 for a trip in progress: a trip that starts at 09:30 uses the closed network for the whole trip. When a request also changes residents, the window compares the same adjusted population with normal service and with the closure. Outside the window, station entries and map trip groups use the adjusted population with normal service. The top-level baseline still describes the original city.

- Use the window with `"temporary": true`. Without it, workers outside the window keep their mode but still get the job moves of a full-day change.
- Map trip groups: groups whose display half-hour is outside the window show the baseline; groups inside show the scenario. A display group has its own half-hour, so the map is a picture, not a count. The window counts are `closure_window` and the station half-hours.

