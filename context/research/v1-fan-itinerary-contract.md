# Fan itineraries for the stadium replay

V1 supplies `events[].itineraries`. Cueca uses the ride options to place passengers on vehicles. Tinta uses the same chosen schedule for the final walk. The V2 consumers confirmed the group counts, local time units, and WGS84 walk geometry in [the V1 thread](https://discord.com/channels/1573514576400752640/1573809708047798364/1573855046125232179). Public ride bindings follow the [viewer source at 93d8d33](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/93d8d33e26c7ec7d21f8a0ef72d484b8ea591da3/viz/src/movers/riders.rs).

## Group fields

| Field | Meaning |
| --- | --- |
| `id` | `home_cut:mode`, unique within one event |
| `home_cut`, `mode` | Home comuna CUT and the top-level mode from `fan_base.by_home` |
| `people` | Modeled fan count, which can be fractional |
| `departure_window_minutes` | Inclusive start and exclusive end in Chile local minutes after midnight |
| `legs` | Ordered legs from the comuna anchor to the venue gate |
| `status`, `reason` | `routed` for a complete path. Use `route_unavailable` and explain the gap if a group has no path. |

Return each positive origin and mode group. An unavailable route keeps its count, an empty `legs` array, and a null departure window. The sum of group counts must match the fan total. Show routed and unrouted counts by mode.

## Legs and schedules

Each leg has `mode` and `minutes`. Give movement legs a WGS84 GeoJSON `LineString`, with longitude before latitude, or a route reference that resolves in a pinned pack. Include access, transfers, wait and final walking time in the duration used for the departure window.

A transit ride includes `service_id`, `board_stop_id`, `alight_stop_id`, and `departure_options`. Bus or surface rail options use `template`, `start_seconds`, `from`, and `to` in the pinned `rider_services` pack. Metro options use `track`, `run`, `from`, and `to` in the pinned `metro_schedule` pack. The bus endpoints are stop-call indices in the template. The Metro endpoints are station indices on the track. Use only options that run on the stated day and keep transfers feasible. Cueca and Tinta use one shared schedule. Where an option cannot be bound to a saved vehicle, retain its group count without drawing a boarding figure. [Viewer handoff](https://discord.com/channels/1573514576400752640/1573809708047798364/1573855485621313618)

Use the 15:00 kickoff and assumed gate-arrival window [14:00, 15:00). A 45-minute itinerary gives departures [795, 855) in local minutes, or 13:15 to before 14:15. Label the day type and the route source. The final walk should end at the mapped perimeter; use a separate return itinerary if the replay shows departures after the match. [Kickoff and window brief](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/463) · [Reference route gaps](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/462)

For the acceptance run, open both saved MUT comparison results. Check that groups conserve 39,222 fans in each, route coverage is explicit, every drawn leg resolves, and any shared ride schedule reaches the gate before 15:00.
