# Optional attendance at two Santiago venues

The normal scenario accepts `kind: "optional_attendance"`. Its input names one synthetic person, the available time and cash budget, and supplied activities. Each activity gives an interest value, a duration, an admission price, venue availability, places available, and a round-trip route with seconds by travel component and fare. A missing route makes the option unavailable. Use a worker or student index from `Population.persons` or an other-resident index from `Population.residents`. The supplied home block must agree with the current pack. The choice receipt is in `applied[].understood`. [Fixed- and optional-attendance research](event-interest-travel-cost.md)

The engine ranks feasible activities with the existing optional-visit rule. Utility is interest in CLP-equivalent minus admission, fare and travel-time cost. Travel-time cost is the sum of component seconds times their values per hour divided by 3,600, rounded to CLP. The cash limit covers admission and fare. The time limit covers travel rounded up to minutes and activity duration. The stay-home option has utility zero. An activity must beat that score. If two options tie above zero, the first wins.

## Frozen choices

The [requests](optional-attendance-witness.json) use synthetic other-resident index 1,560,011. This unemployed resident (role 3, age band 45–64) lives in Maipú block 13119101001048, at −70.758097, −33.500088. Index and home come from the current deterministic Census pack, which represents synthetic people. [Resident pack](../../data/mvp/person_traits_residents.csv) · [Placement](../../engine/src/population.rs)

A and B assume the same hypothetical 90-minute music event, CLP 12,000 admission, CLP 13,000 cash budget and 260 available minutes. The resident has a bicycle and can park it for zero added cash. Route times are FOSSGIS bicycle estimates to named public perimeter gates. The source returns a 31.7 m starting-road snap, and the estimates omit queues, access inside the venue and special event restrictions. The free 20-minute neighborhood walk starts at the same home. Its duration and utility of 400 CLP-equivalent are scenario inputs. [Santander Arena access](https://www.santanderarena.cl/como-llegar-a-santander-arena) · [Estadio Nacional access](https://parqueestadionacional.cl/como-llegar) · [Route capture](https://discord.com/channels/1573514576400752640/1573863864624349259)

The SNI urban appraisal values are CLP 3,338 per traveler-hour in vehicle and CLP 6,676 waiting or walking. This scenario uses the in-vehicle rate for cycling as a sensitivity assumption. The high interest value of CLP 19,000-equivalent and low value of CLP 17,000-equivalent are explicit inputs. Hours and places available are supplied conditions. These choices describe one person under the stated conditions. There is no aggregate ticket allocation or attendance prediction. ENPCCL participation and EOD recreation inform future checks of travel and activity outcomes. [SNI 2026 appraisal prices](https://sni.gob.cl/wp-content/uploads/Precios-Sociales-2026.pdf) · [Source scope](event-interest-travel-cost.md)

| Request | Event travel | Total minutes | Event cash | Generalized event cost | Event utility | Expected choice |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `arena` | 6,591 s | 200 | CLP 12,000 | CLP 18,111 | 889 | Music at Santander Arena |
| `nacional` | 9,299 s | 245 | CLP 12,000 | CLP 20,622 | −1,622 | Neighborhood walk |
| `arena_low_interest` | 6,591 s | 200 | CLP 12,000 | CLP 18,111 | −1,111 | Neighborhood walk |
| `arena_sold_out` | 6,591 s | 200 | CLP 12,000 | CLP 18,111 | 889 | Neighborhood walk |
| `arena_stay_home` | 6,591 s | 200 | CLP 12,000 | CLP 18,111 | −1,111 | Stay home |

Both venues fit the same hard time and cash budgets. Only the route and venue differ in A and B. The low-interest control uses the exact A route. The sold-out control supplies zero places. The stay-home control makes the alternative unavailable. To pass these cases through the normal CLI use `jq '[.[]]' context/research/optional-attendance-witness.json | engine/target/release/scenario data/mvp -`. Each receipt is a separate counterfactual within that request.
