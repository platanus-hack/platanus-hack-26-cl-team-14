# Metro closure only from 07:00 to 10:00

Owner: rigorous-agent (V1). Stacked on bold-boi's [full closure](metro-closure-choice.md). Boundary rule: Eliseo-21, V1 thread message 1573813050232938691.

Add `"half_hours": [14, 20]` to a change (input: [metro_closure_window.json](metro_closure_window.json)). The window is `[start, end)` in half-hours from 00:00.

- Rule: a worker whose normal-weekday departure half-hour is inside the window faces the closed network for the whole trip. Other workers keep their baseline mode. The departure comes from the EOD 2012 profile of the worker's mode (`work_departures.csv`), at a quantile per person from a SplitMix64 hash of its index, independent of the home and job samplers.
- Station entries change only inside the window. Outside it they keep the baseline.
- Output `closure_window`: workers who leave home in the window, by mode, base and scenario. Label: "trabajadores que salen entre 07:00 y 10:00".

Real run (`scenario` binary, `data/mvp`):

| Workers leaving 07:00–10:00 | Baseline | Metro closed |
| --- | ---: | ---: |
| Public transport | 772,186 | 663,173 |
| Car | 504,342 | 591,475 |
| Walk | 92,737 | 94,642 |
| Bicycle | 68,637 | 74,152 |
| Other | 49,553 | 64,013 |
| Total | 1,487,455 | 1,487,455 |

Weekday Metro entries: 2,457,348 → 1,748,979. Tobalaba: zero in bins 14–19, baseline in the other bins.

The window does not show reopening at 10:00 for a trip in progress: a trip that starts at 09:30 uses the closed network for the whole trip. A residents change in the same request turns the window off and the result lists it in `not_modeled`.

- Use the window with `"temporary": true`. Without it, workers outside the window keep their mode but still get the job moves of a full-day change.
- Map trip groups: groups whose display half-hour is outside the window show the baseline; groups inside show the scenario. A display group has its own half-hour, so the map is a picture, not a count. The window counts are `closure_window` and the station half-hours.

