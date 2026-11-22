# Route coverage for the Monumental and MUT fan groups

The [saved MUT comparison](v3-mut-replay.md) gives one estimated fan count per home comuna and mode in `events[0].fan_base.by_home`. A match at the Monumental and a match at the MUT each use the same 39,222 attendees. Join each home `cut` and `fan_base.venue_cut` as `origin-destination` to the existing route dictionaries.

| Venue | Public reference paths with a ride | Fans in the covered public groups | Car reference paths | Fans in the covered car groups |
| --- | ---: | ---: | ---: | ---: |
| Monumental | 34 of 52 origins | 21,118.1 of 23,687.5 | 50 of 52 origins | 12,565.2 of 12,758.9 |
| MUT | 34 of 52 origins | 24,740.5 of 27,789.9 | 51 of 52 origins | 8,851.4 of 9,397.8 |

I read both full cached results from the V3 bundle and joined positive group counts to the route dictionaries on main `9183b83`. A public route counts only if it contains a ride segment. The diagonal is absent from the reference routing. These are matches to comuna reference paths. Keep the uncovered counts in the output. [V3 cache bundle](https://discord.com/channels/1573514576400752640/1573809606277337280/1573844937198411918)

## What the route packs supply

- [Public routes](../../web/data/public_routes.json) give WGS84 `path` plus ordered `segments`. A ride has `mode`, `service`, `stop_ids` and inclusive path indices. The source is a weekday 08:00 GTFS headway model. It gives no scheduled event departure, per-leg duration, or network walking path. [Builder](../../tools/pack/public_routes.py) · [Timing reference](../../tools/pack/travel_times.py)
- [Car routes](../../web/data/car_routes.json) give WGS84 paths between road anchors. They are baseline paths. The event comparison closes roads at the MUT in case B. Use the edited road network for the car path where it changes. Neither dictionary identifies parking or its capacity. [Road graph scope](../../data/mvp/road_graph.md)

The public reference endpoints lie about 2.15 km from the Monumental and 4.28 km from the MUT event point. Use a venue stop and a walking route to a named gate for the final public leg. Use a declared parking or drop-off point plus walking for car. Label origin points as comuna anchors. A before-kickoff departure window and a schedule for the event's day type need explicit inputs. The 17:00 assumption in the saved request is the event end.
