# Weekday health trips

`health` in the scenario result has weekday outbound health trips by destination comuna, before and after a change. It uses the same rule as the shopping layer ([shopping-trips-engine.md](shopping-trips-engine.md)), with health data:

- Trips: EOD 2012 health trip rate per resident of the home comuna (`eod_trip_rates.csv`, purpose `health`) × Census 2024 residents. Total 382,133 per weekday.
- Destinations: Huff model by comuna, P(d | o) ∝ (points_d + 1)^α × exp(−β × minutes_od). points_d = health establishments in `data/mvp/health_points.csv`: the 1,061 DEIS establishments of the Región Metropolitana in operation that patients visit (CESFAM, SAPU, SAR, hospitals, clinics, private centres, labs and others), from the official registry of 17 November 2026 (`tools/pack/deis_health_points.py`, [DEIS](https://datos.gob.cl/dataset/establecimientos-de-salud-vigentes)). Fit on EOD 2012 trips with Proposito 5 (De salud), 1,612 sample trips, 351,210 expanded: α = 1.188, β = 0.1272 per minute. Reproduce: `python3 -I tools/pack/shopping_huff.py 5 all health_points.csv`.
- Fit: in the engine (Census 2024 residents), destination total variation against EOD 2012 is 0.181, against 0.198 with OSM points. In the fit script (EOD weights) it is 0.165, against 0.195 against EOD 2012 health destinations is 0.165 with the 552 OSM hospital and clinic points of the first version. OSM misses most primary care (CESFAM, SAPU).

Health points are not editable yet; a change of travel minutes moves the trips.

## Who sees a changed network

Only the public-transport trips that leave inside the closure window see the scenario minutes. Per home comuna, that share is the EOD 2012 public mode share × the departure share inside the window (`eod_trip_profiles.csv`); without a window it is the public mode share. The other trips keep the base minutes. The same rule now applies to the shopping layer. Before, a 07:00–10:00 Metro closure moved every shopping trip of the day: Santiago fell from 228,720 to 114,597 shopping trips. Now it falls to 227,860. Results outside `shopping` and `health` do not change.

## Result (release CLI, data/mvp)

| Scenario | Health trips that see the change | Santiago | Providencia | Maipú |
| --- | ---: | ---: | ---: | ---: |
| Baseline | — | 61,805 | 46,315 | 16,142 |
| Metro closed 07:00–10:00 | 83,929 | 54,040 | 43,180 | 17,733 |
| Metro closed all day | 214,542 | 41,642 | 37,535 | 20,801 |

Health is 3.3% of 07:00–10:00 trips, twice its daily share ([v1-missing-services.md](v1-missing-services.md)). The trips are comuna flows, not identified patients or a chosen hospital. Urgency, opening hours and capacity are not modeled.
