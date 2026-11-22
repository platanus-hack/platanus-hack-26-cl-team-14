# Cars around placed buildings

Task: keep cars on mapped streets when a placed building or stadium covers a road.
This supports the MUT and Monumental swap at 0:12–1:02 in `context/pitch.md`.
Vicente requested a separate worktree and a PR on 22 November 2026, Chile time.

## Sources read

- [SUMO rerouter](https://sumo.dlr.de/docs/Simulation/Rerouter.html), read 22 November 2026: close affected road edges and route approaching vehicles on the remaining network.
- [SUMO implementation](https://github.com/eclipse-sumo/sumo/blob/main/src/microsim/trigger/MSTriggeredRerouter.cpp), read 22 November 2026: closing actions resolve a road edge and store its permissions. The implementation uses the existing network and Dijkstra routing.
- [Local directed graph](../../data/mvp/road_graph.md): shared OSM nodes define junctions. Directed edges retain one-way and access rules. The export covers major roads and omits turn restrictions and traffic delays.

## Decision

Build a directed local-road pack from the existing OSM region extract. The major-road graph omits residential way 23350356, which carries the reported car route. Use original OSM node IDs and the existing access rules. Do not draw a new road along a building boundary.
Apply the same footprint checks to ordinary cars and venue cars.
Keep baseline routes and trip clocks when the placement does not affect them.
A view correction does not establish a new congestion or travel-time estimate.

## Verification

The full viewer source passed a Rust metadata compile with the existing release dependencies.
A separate geometry run used the actual detour, mask, road-edit and restored-road source with 3,086 real car paths from the saved MUT/Monumental replay. Its terrain was flat and its path constructor was a small stand-in; this checks geometry, not the native view.

- 39 affected paths: 37 rerouted, two omitted, zero remaining footprint collisions, zero retired-road segments, and unchanged trip clocks.
- Local group 391, route 326 ends on one-way OSM way 23350356. The stadium covers its only upstream approach. The endpoint matches the mapped edge within 0.36 m; no other edge approaches within 5 m. Local route 368 ends inside the footprint. Both whole journeys are omitted.
- A real building footprint translated onto another real road also produced a clear detour.
- Reset restored all 3,086 original path references.
- Source review covered ordinary cars, venue cars, displayed body bounds, directed joins and cache reset. Route anchors allow 1.5 m for packed coordinate precision.

Native playback and a screenshot remain pending. The change keeps the replay clocks and engine counts; it does not recalculate traffic demand or travel times.
