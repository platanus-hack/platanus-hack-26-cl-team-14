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

## Limits

- Outbound trips, not distinct persons. Return trips, departure time and mode are not modeled yet.
- 7 RM comunas have no EOD 2012 rate and give zero trips: 13203, 13303, 13404, 13502, 13503, 13504, 13505.
- Each shop point inside a comuna gets an equal part. A point is an OSM object, not floor area or sales.
- `shopping_huff.csv` gives 5,745 trips for the MUT. It adds the MUT as a separate destination with 26 shops, on top of the Las Condes shops. The engine splits the comuna's trips by shop point. The two numbers measure different rules.
- The operator reports about 26,000 visits a day for all purposes (#201). Shopping trips by residents are one part of this.
- Weekday only. 2012 trip rates with 2024 residents and 2026 shops.
