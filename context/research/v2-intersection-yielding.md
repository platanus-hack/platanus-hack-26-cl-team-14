# Viewer yielding at mapped junctions

The viewer matches ordinary car routes to directed edges in the packed OSM
graph. A route enters a ground control only when both edges connect to the
same ground node. This keeps traffic on a bridge or tunnel separate.

A single stop can serve several nearby junctions. The scheduler uses each
movement's route samples to find where two cars could touch. It records the
times when those movements cannot start together. Cars keep their order in
each approach, and compatible movements can pass at the same time. A wait
carries forward to the next control.

Road closures and detours apply before this calculation. The waits are simulated and
apply only to the viewer. Coverage requires a matched ground approach and
exit. Scheduled buses and event traffic use separate clocks.

[Road graph](../../data/mvp/road_graph.md) · [SUMO signal phases and clearance](https://sumo.dlr.de/docs/Simulation/Traffic_Lights.html)
