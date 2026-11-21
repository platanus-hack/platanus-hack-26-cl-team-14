# Full Metro closure: people choose again

Owner: bold-boi (V1). Boundary before this change: [v1-metro-service-closure.md](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/349) (station closures zeroed entries and changed no choice).

## Change properties

- `{"kind":"line","op":"remove","target":{"type":"mode","id":"metro"}}` removes all active patterns of a mode in one reroute (389 Metro patterns in the GTFS 2026 pack). Buses stay.
- `"temporary": true` on a line change: its comuna-pair minute changes move **mode** choice only. Workers keep their job comuna. Use it for a closure of one day; leave it out for a permanent network change. This is the short-run versus long-run split of activity-based models: a disruption causes replanning of the trip, not relocation (Müller, Leich and Nagel 2020, cited in #349).
- Close the 126 stations too (`kind:"station"`, value 0), so station entries are zero.

Input file: [metro_full_closure.json](metro_full_closure.json).

## Real run (weekday, `data/mvp`, port 8841)

| Measure | Baseline | All Metro closed |
| --- | ---: | ---: |
| Metro entries per weekday (station model) | 2,457,348 | 0 |
| Workers by public transport (now bus only) | 1,416,237 | 1,216,062 (−14 %) |
| Workers by car | 799,249 | 958,990 (+20 %) |
| Walk / bicycle / other | 243,674 / 102,545 / 127,109 | 247,194 / 112,668 / 153,900 |
| Workers who change job place | — | 0 |

- 1,086 comuna pairs get slower by public transport, median +26.3 min. No pair loses every transit path, so the rule for a lost path (keep the previous time) does not act in this run.
- Largest drop in public share: Puente Alto 60.4 % → 45.7 %, La Florida 58.9 % → 44.4 %, San Miguel 60.0 % → 46.8 %, Maipú 57.6 % → 46.8 %. Rural comunas without Metro do not change.
- Empty `changes` gives the same output as before this change (all keys except `elapsed_ms` and `revision`).

## Closure window 07:00–10:00

Add `"half_hours": [14, 20]` to a change (input: [metro_closure_window.json](metro_closure_window.json)). The window is `[start, end)` in half-hours from 00:00.

- Rule: a worker whose normal-weekday departure half-hour is inside the window faces the closed network for the whole trip. Other workers keep their baseline mode. The departure comes from the EOD 2012 profile of the worker's mode (`work_departures.csv`), at a fixed quantile per person.
- Station entries change only inside the window. Outside it they keep the baseline.
- Output `closure_window`: workers who leave home in the window, by mode, base and scenario. Label: "trabajadores que salen entre 07:00 y 10:00".

Real run (`scenario` binary, `data/mvp`):

| Workers leaving 07:00–10:00 | Baseline | Metro closed |
| --- | ---: | ---: |
| Public transport | 771,476 | 662,465 |
| Car | 503,747 | 590,755 |
| Walk | 92,603 | 94,564 |
| Bicycle | 68,658 | 74,170 |
| Other | 49,377 | 63,907 |
| Total | 1,485,861 | 1,485,861 |

Weekday Metro entries: 2,457,348 → 1,748,979. Tobalaba: zero in bins 14–19, baseline in the other bins.

The window does not show reopening at 10:00 for a trip in progress: a trip that starts at 09:30 uses the closed network for the whole trip. It assumes residents do not change; with a residents change the window rule is not applied correctly.

## Limits

- Routing uses a fixed 08:00 departure on 13 April 2026. There is no service window (07:00–10:00) yet.
- Road times are free flow: 160,000 more car commuters cause no congestion in the model.
- Bus capacity is not modeled: the bus network takes 1.2 million workers with no crowding.
- October 2019 comparison: the closure was on a weekend with no matched public counts by mode (#343). The comparison for the demo is service state only.
