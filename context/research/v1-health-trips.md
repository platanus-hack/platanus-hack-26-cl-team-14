# Weekday health trips

`health` in the scenario result has weekday outbound health trips by destination comuna, before and after a change. It uses the same rule as the shopping layer ([shopping-trips-engine.md](shopping-trips-engine.md)), with health data:

- Trips: EOD 2012 health trip rate per resident of the home comuna (`eod_trip_rates.csv`, purpose `health`) × Census 2024 residents. Total 382,133 per weekday.
- Destinations: Huff model by comuna, P(d | o) ∝ (points_d + 1)^α × exp(−β × minutes_od). points_d = health establishments in `data/mvp/health_points.csv`: the 1,061 DEIS establishments of the Región Metropolitana in operation that patients visit (CESFAM, SAPU, SAR, hospitals, clinics, private centres, labs and others), from the official registry of 17 November 2026 (`tools/pack/deis_health_points.py`, [DEIS](https://datos.gob.cl/dataset/establecimientos-de-salud-vigentes)). Fit on EOD 2012 trips with Proposito 5 (De salud), 1,612 sample trips, 351,210 expanded: α = 1.188, β = 0.1272 per minute. Reproduce: `python3 -I tools/pack/shopping_huff.py 5 all health_points.csv`.
- Fit: destination total variation against EOD 2012 health destinations. Engine (Census 2024 residents): 0.181 with DEIS points, 0.198 with the 552 OSM hospital and clinic points of the first version. Fit script (EOD weights): 0.165 and 0.195. OSM misses most primary care (CESFAM, SAPU).

Health points are not editable yet; a change of travel minutes moves the trips.

## Who sees a changed network

Only public-transport trips within the closure window use the changed minutes. For each home comuna and purpose, `eod_trip_profiles.csv` gives the weighted share of trips that use public transport and start in each half hour. Each share divides by the weight of all trips with known mode and departure time. Sum these shares inside the window. An all-day change uses the public mode share. Other trips keep their base minutes.

The morning Metro scene uses bins 14 to 19. The former rule multiplied separate mode and departure shares. It gave 83,929 health trips and 12,145 shopping trips eligible to respond. The joint table gives 86,715 and 19,350. The sum of absolute errors across the two regional totals falls from 9,991 to zero at whole-trip precision. This check is fitted to EOD 2012. It measures eligible trips. EOD provides no observed response to this hypothetical closure.

Rebuild the profile with `python3 -I tools/pack/eod_trip_rates.py --survey /path/to/EOD_STGO`. Run `engine/target/release/scenario data/mvp context/research/metro_closure_window.json` and read `health.exposed_trips` and `shopping.exposed_trips`. The old packed profile reproduces the separate-share rule. [Santiago EOD 2012 files](https://users.dcc.uchile.cl/~egraells/gds-data/eod2012.tgz)

## Result (release CLI, data/mvp)

| Scenario | Health trips that see the change | Santiago | Providencia | Maipú |
| --- | ---: | ---: | ---: | ---: |
| Baseline | — | 61,805 | 46,315 | 16,142 |
| Metro closed 07:00–10:00 | 86,715 | 54,043 | 43,121 | 18,023 |
| Metro closed all day | 214,542 | 41,642 | 37,535 | 20,801 |

Health is 3.3% of 07:00–10:00 trips, twice its daily share ([v1-missing-services.md](v1-missing-services.md)). The trips are comuna flows, not identified patients or a chosen hospital. Urgency, opening hours and capacity are not modeled.
