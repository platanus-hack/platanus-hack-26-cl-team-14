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

## Limits

- Routing uses a fixed 08:00 departure on 13 April 2026. There is no service window (07:00–10:00) yet.
- Road times are free flow: 160,000 more car commuters cause no congestion in the model.
- Bus capacity is not modeled: the bus network takes 1.2 million workers with no crowding.
- October 2019 comparison: the closure was on a weekend with no matched public counts by mode (#343). The comparison for the demo is service state only.
