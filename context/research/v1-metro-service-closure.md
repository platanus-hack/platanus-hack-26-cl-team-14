# Model a Metro service closure

V1 research for the cached full Metro closure replay. Engine and data revision `f623d40bc40495a1824030861f2fd2445777e5a0`. The proposed 07:00–10:00 weekday replay and the October 2019 closure weekend use different service calendars. Compare each with the evidence for its own date and interval. [Verticals](../verticals.md)

## Current engine

| Change | What the engine does today |
|---|---|
| Close a station | Removes a physical station ID from the access model. The model projects entries at open stations. The station flag leaves travel times and mode choice inputs unchanged. |
| Remove a directed transit pattern | Rebuilds public transport paths between comuna points for departures at 08:00 on 13 April 2026, with arrivals by 11:00. A finite-to-finite difference adjusts the choice time. If the edited pair loses its last transit path, the receipt reports a null travel time and the choice model keeps its previous finite time. |
| Publish a viewer snapshot | Samples journey groups from the population after the choice step. The viewer can truncate a group’s itinerary at a closed station. The scheduled train replay uses its own packed services and has no scenario service mask. |

[Station state](../../engine/src/scenario_stations.rs) · [Scenario and choice inputs](../../engine/src/scenario.rs) · [Routing window](../../engine/src/editable_transit/timetable.rs) · [Viewer trips and train schedules](../../viz/src/movers.rs)

**Measured access-only check.** I closed all 126 physical stations in one HTTP request on the pinned engine. Revision 2 accepted all 126 and gave 2,457,348.4 → 0 modeled entries per weekday. The count of 2,688,814 workers, all 2,000 displayed trip groups, and every comuna's public transport share stayed the same. The run verified station access. The edit has no time selector and gives no evidence on mode shifts, road speeds or congestion. [Run receipt](https://discord.com/channels/1573514576400752640/1573809708047798364/1573810541732962445)

## Service properties for a general replay

- Select the service date and local interval. Use the mode and route or pattern ID to select affected services. The March 2026 pack has 389 Metro patterns across L1, L2, L3, L4, L4A, L5 and L6. Use trip and stop-call times to decide which scheduled departures an interval affects. Keep the physical station ID and boarding rules separate from trip availability. [Current catalog](../../data/mvp/editable_transit.md) · [GTFS Schedule](https://gtfs.org/documentation/schedule/reference/)
- Apply the availability change before routing. Recompute public transport paths using the remaining services. Keep buses available where a path exists. When a pair has no public transport path, model public transport as unavailable for that pair and time. Keep other mode costs and demand assumptions explicit. OpenTripPlanner uses trip updates for routing changes. MATSim disruption research also separates service disruption from passenger replanning. [GTFS Realtime](https://gtfs.org/documentation/realtime/reference/) · [OpenTripPlanner](https://docs.opentripplanner.org/en/latest/Version-Comparison/#other-features-removed-from-otp2) · [Müller, Leich and Nagel, 2020](https://depositonce.tu-berlin.de/items/d935f494-8d87-46f2-85c9-9d7b2edda905)
- Save the active service state, time and resulting journeys with the replay. The same state must govern entries, paths and drawn trains. The current choice output groups public transport together. A separate bus and Metro chart needs mode counts from the resulting paths. Road delay needs a traffic model with capacity or observed speeds. The current free-flow road times cannot measure it. [Mode categories](../../engine/src/choice.rs) · [Scenario result](../../engine/src/scenario.rs)

Use zero operated Metro service as an availability check for a confirmed closure interval. Score behavior only when observations cover the same period, geography and units. A closure notice supplies service state. It gives no mode shifts or road-delay measurements. The October 2019 evidence owner is checking those observations separately. [V1 evidence thread](https://discord.com/channels/1573514576400752640/1573809708047798364/1573810481746018444)
