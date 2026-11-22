# Shopping trips in the engine

The engine now models weekday shopping trips of residents and the shop points that attract them.
A change of shop points at a place moves these shoppers. There is no rule for one place.

## Sources

| Property | Source |
| --- | --- |
| Shopping trips per resident, by home comuna | EOD 2012, purpose 11 (`data/mvp/eod_trip_rates.csv`) |
| Residents by home comuna | Census 2024 blocks (`data/mvp/blocks.csv`) |
| Shop points (size of a destination) | OSM shop, mall and supermarket points, Nov 2026 (`data/mvp/activity_points.csv`) |
| Destination rule | Huff model, α 0.586, β 0.2047 per minute, fitted on EOD 2012 shopping trips (`data/mvp/shopping_huff.md`) |
| Minutes between comunas | The same matrix as the work choice (`choice.rs`) |
| Departure half-hour and mode, by home comuna | EOD 2012 shopping trips (`data/mvp/eod_trip_profiles.csv`); modes grouped as in `tools/pack/eod_leisure.py` |

Code: `engine/src/shopping.rs`. Output: `shopping` in the scenario result.

## Change

`{"kind":"shops","target":{"type":"point","longitude":x,"latitude":y},"op":"add","value":k}`.
A negative `k` removes the `|k|` nearest shop points. A positive `k` adds `k` shop points at the point.
The MCP also accepts a place target, for example `{"type":"place","id":"mut"}`.

## Result for the MUT ↔ Monumental swap

Request: MUT −26 shop points, Monumental +26 shop points. The 26 removed points are within 65 m of the MUT point.

| Site | Weekday shopping trips | Top home comunas |
| --- | ---: | --- |
| MUT shops, base | 1,228 | Las Condes 479, Vitacura 236, Providencia 162 |
| Same shops at the Monumental | 2,647 | Macul 725, Ñuñoa 682, Peñalolén 322 |

Total: 1,778,260 weekday shopping trips. Destination total variation against EOD 2012 at baseline: 0.192 (the offline fit gives 0.180).

Each site also gives `by_mode` (car, public, walk, bicycle, other) and `by_departure_half_hour` (48 bins from 00:00). MUT shoppers in the base: car 577, walk 483, public 129, bicycle 35. At the Monumental: walk 1,582, car 647, public 290.

Read a site figure with care. Site shoppers are an equal share of the comuna's shoppers per shop point. Macul has few shop points, so each point gets a larger share: about 1,190 of the 2,647 trips come from existing Macul shops. Use `destinations_changed` for the net effect: Macul +1,455, Las Condes −575.

The engine total, 1,778,260 trips in 45 comunas, equals the Census-weighted run in PR #353.

## Limits

- Outbound trips, not distinct persons. Return trips are not modeled yet.
- A trip keeps the departure and mode profile of its home comuna at any destination. A move of the shops does not change mode choice.
- 7 RM comunas have no EOD 2012 rate and give zero trips: 13203, 13303, 13404, 13502, 13503, 13504, 13505.
- Each shop point inside a comuna gets an equal part. A point is an OSM object, not floor area or sales.
- `shopping_huff.csv` (5,745) and PR #353 (5,696 at the MUT, 6,996 at the Monumental) add the MUT as a separate destination with 26 shops, on top of the Las Condes shops. That is a rule for one place, and it counts the 26 points two times. The engine keeps the fitted comuna model and splits each comuna's trips by shop point.
- The operator reports about 26,000 visits a day for all purposes (#201). Shopping trips by residents are one part of this.
- Weekday only. 2012 trip rates with 2024 residents and 2026 shops.
