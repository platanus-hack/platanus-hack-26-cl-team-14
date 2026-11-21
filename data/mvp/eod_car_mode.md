# eod_car_mode.csv

Weekday trips to work in Santiago EOD 2012, by home comuna, household car status and mode.
Engine input of `engine/src/car_ownership.rs` (cut 5, link 3).

`cut,households,households_with_car,household_has_car,mode,sample_trips,expanded_trips`. 45 comunas × 2 × 3 rows.

- Households: `Hogares.Factor`. A household has a car if `Vehiculo.TipoVeh` is 1 (auto), 3 (pickup or SUV) or 5 (van or minivan). `NumVeh` is not used: it also counts motorcycles, trucks, taxis and buses.
- Trips: `viajes.Proposito` = 1, weight `FactorLaboralNormal × personas.Factor_LaboralNormal`, as `work_departures.csv`.
- Modes, the engine groups: car = `ModoAgregado` 1; public = 2–7 and 9–12; other = all other codes (walk, bicycle, car with Metro, other).
- Rebuild: `python3 -I tools/pack/eod_car_mode.py` after `python3 -I tools/scrape/eod2012.py` and extraction to `data/raw/eod2012/x`.

| Region | public | car | other | sample trips |
| --- | --- | --- | --- | --- |
| household without a car | 0.595 | 0.072 | 0.333 | 7,372 |
| household with a car | 0.252 | 0.527 | 0.221 | 6,068 |

## Limits

- 2012 survey, not 2024. Association in the survey, not a measured effect of a change in ownership.
- Two denominators: the input is the household car share; the mode mixture uses the share of trips to work from car households (`car_ownership.rs`).
- 13116, 13132 and 13602 have fewer than 30 sample trips in one group, and 7 comunas are not in the sample: the engine uses the pooled region values there.
- `vehicles.csv` counts circulation permits by issuing comuna, not ownership by home comuna. It is not used.
