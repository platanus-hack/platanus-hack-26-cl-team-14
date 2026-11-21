# Mapocho chart and capture

Handoff for the [Mapocho storyboard](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/338). The field names follow the [merged V1 ferry contract](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/359). The capture runner can use the routed preview below to check the final cached result.

## Routed preview

The saved pair `mapocho-baseline` → `mapocho-ferry` starts at 07:00. The ferry uses 355 river-following points in each direction and the corresponding travel times. Both saved results played with one relay call each and zero simulation calls. Preview source `f6da8dec5b7a72b37d1112a73e278a6b3e6206e4`. [Requests, caches, and trace](https://discord.com/channels/1573514576400752640/1573809606277337280/1573822794423345193)

The chart uses `line_riders.0.riders_to_work` for east and `line_riders.1.riders_to_work` for west. The saved values are **3,470.2 east** and **932.2 west**, in modeled work trips per weekday. The baseline has no ferry. Canelo independently matched the chart values, IDs, and geometry to the saved results. The payload sets an 11-second chart delay. The first visible frame still needs verification. [Independent check](https://discord.com/channels/1573514576400752640/1573809606277337280/1573823306749181964)

## Pier sequence

Use the 33-second Mapocho slot in the pitch. Allow 8 seconds for the question and assumptions and 5 seconds for the visible baseline. Then play the 20-second ferry shot. Both shots start at 07:00. [Pitch schedule](../../pitch.md)

| Time in ferry shot | What to show |
| --- | --- |
| 0–4 s | Open the route-wide view with the ferry service. |
| 4–9 s | Fly to Parque Forestal. Follow the eastbound ferry approaching MAP07 near Pío Nono. Show people walking to the pier. |
| 9–16.5 s | At the pier, show arriving passengers get off and continue on foot. Show people at the pier board the boat. |
| 16.5–20 s | Show the ferry leave with the new passengers. Bring in the compact chart at the edge once the departure is visible. |

The draft display timetable assumes service is underway at 07:00, with departures at :00, :10 and each later ten-minute mark. The eastbound 06:10 ferry reaches MAP07 at about 07:01:12 and leaves at about 07:02:12. The saved one-minute dwell gives 7.5 seconds at 8× playback. The 11-second chart delay in the preview is timing evidence. Use 17 seconds as the first rehearsal target for the chart on the extended pier action. The full sequence needs a native run. [Combined request and cache](https://discord.com/channels/1573514576400752640/1573809606277337280/1573831669855813803)

At MAP07, the model gives 312.2 eastbound boardings and 1,059.2 eastbound alightings per weekday. Use the counts to guide the size of the groups. Label the individual people, their paths, and their boat assignment as illustrative. The pier sequence shows arriving and departing groups on the same stop. [Mapocho saved result](https://discord.com/channels/1573514576400752640/1573809606277337280/1573822794423345193)

## Chart fields

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
- Check the vessel position on the routed geometry with V2. The preview saves the path. The native vessel motion and pier access still need verification.
- Rebuild both results on the integrated engine and viewer revision. Confirm zero simulation calls during playback. Check the chart values, real-time reveal, wide camera, close-up, and terminal return in the native rehearsal.
