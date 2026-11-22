# Viewer yielding at mapped junctions

The viewer matches ordinary car routes to directed edges in the packed OSM
graph. A route enters a ground control only when both edges connect to the
same ground node. This keeps traffic on a bridge or tunnel separate.

Each junction admits the first eligible car in each approach. Compatible
movements can run together. A short group already crossing can finish
before the next conflicting movement. A car waits at the upstream stop
when the next lane has no room. The room check includes cars due to reach
the receiving lane before it. A wait adds elapsed time to later arrivals.

Road closures and detours apply before this calculation. The waits are simulated and
apply only to the viewer. Coverage requires a matched ground approach and
exit. Scheduled buses and event traffic use separate clocks.

[Road graph](../../data/mvp/road_graph.md) · [SUMO signal phases and clearance](https://sumo.dlr.de/docs/Simulation/Traffic_Lights.html)
