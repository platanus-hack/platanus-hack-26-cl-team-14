# Mapocho chart and capture

Handoff for the [Mapocho storyboard](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/338). The draft contract below follows the [V1 ferry proposal](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/359) at `de072bdce878ac69da296de224c4804985592716`. Confirm the fields against the final cached result.

## Chart

| Screen label | Source and unit |
| --- | --- |
| Viajes al trabajo en barco por hora de salida | For each direction, add adjacent bins of `line_riders[].riders_half_hour`. The bins use weekday work departure shares. Label the values as modeled one-way trips, grouped by their departure hour. They do not give the times at which riders board. |
| Muelles con más subidas | Use `line_riders[].stops[].boardings` by pier and direction. These are modeled boardings for the full weekday. Keep them off the 07:00–10:00 chart. |
| Modo anterior | Use `line_riders[].taken_from` only to describe the modeled source of ferry trips over the full weekday. The totals cover public transport, car, and other modes. Keep them off a road congestion chart. |

For the proposed 07:00–10:00 chart, start at bin 14 and end after bin 19. Use 14+15, 16+17, and 18+19 for the three hourly bars. Filter to `MAPOCHO-E` and `MAPOCHO-W` and label each direction. Tag this window as a view of weekday departures until V1 supplies a service start and end that govern the rider calculation.

The draft supplies no street speed or segment delay. Leave the congestion panel pending. The source-mode counts do not say where cars left the road or how fast traffic moved. Keep the EOD travel-time comparison as a separate screen figure with its own denominator. [Trip-level comparison](boat_trips.md)

## Capture

- Save the same weekday demand and 07:00 scene for the baseline and experiment. Save both directed line definitions, pier order and coordinates, speed, dwell, headway, capacity, and any time limits. Record the engine revision, replay IDs, and chart values.
- Use the [Parque Forestal mock](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/347) as a close-up reference. Its center is longitude −70.637000 and latitude −33.435590, with north up and a vertical span of about 400 m.
- The draft line geometry connects consecutive pier points with straight segments. Use a river-following track for the vessel capture. Check the mapped route and the boat position at the shot time with V2.
- Open both cached results and confirm that playback makes zero simulation calls. Check chart values against the saved JSON and the frame against the route geometry. The engine output and native capture still need verification.
