# Weekday health trips

`health` in the scenario result has weekday outbound health trips by destination comuna, before and after a change. It uses the same rule as the shopping layer ([shopping-trips-engine.md](shopping-trips-engine.md)), with health data:

- Trips: EOD 2012 health trip rate per resident of the home comuna (`eod_trip_rates.csv`, purpose `health`) × Census 2024 residents. Total 382,133 per weekday.
- Destinations: Huff model by comuna, P(d | o) ∝ (points_d + 1)^α × exp(−β × minutes_od). points_d = OSM hospital (99) and clinic (453) points in `activity_points.csv`. Fit on EOD 2012 trips with Proposito 5 (De salud), 1,612 sample trips, 351,210 expanded: α = 1.170, β = 0.1354 per minute. Reproduce: `python3 -I tools/pack/shopping_huff.py 5 hospital,clinic`. The same script without arguments still gives the shopping fit (α 0.586, β 0.2047).
- Fit: destination total variation against EOD 2012 health destinations is 0.198 (shopping: 0.192).

Health points are not editable yet; a change of travel minutes moves the trips.

## Who sees a changed network

Only the public-transport trips that leave inside the closure window see the scenario minutes. Per home comuna, that share is the EOD 2012 public mode share × the departure share inside the window (`eod_trip_profiles.csv`); without a window it is the public mode share. The other trips keep the base minutes. The same rule now applies to the shopping layer. Before, a 07:00–10:00 Metro closure moved every shopping trip of the day: Santiago fell from 228,720 to 114,597 shopping trips. Now it falls to 227,860. Results outside `shopping` and `health` do not change.

## Result (release CLI, data/mvp)

| Scenario | Health trips that see the change | Santiago | Providencia | Lo Prado |
| --- | ---: | ---: | ---: | ---: |
| Baseline | — | 44,073 | 43,835 | 11,719 |
| Metro closed 07:00–10:00 | 83,929 | 38,650 | 41,009 | 10,253 |
| Metro closed all day | 214,542 | 29,796 | 35,782 | — |

Health is 3.3% of 07:00–10:00 trips, twice its daily share ([v1-missing-services.md](v1-missing-services.md)). The trips are comuna flows, not identified patients or a chosen hospital. Urgency, opening hours and capacity are not modeled.
