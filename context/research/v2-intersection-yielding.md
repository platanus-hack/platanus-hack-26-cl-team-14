# Viewer yielding at mapped junctions

The viewer uses the packed OSM car graph to find shared nodes reached by
routed commuter cars. Ground arcs establish the node topology. A projected
intersection across a bridge or tunnel does not establish a connection. A
node gets a control when routes on separate incoming approaches conflict
inside it. A car's incoming geometry upstream of its stop groups routes
using common pavement. Sampled entrance, turn and exit geometry determines
which movements can share the control.

The scheduler reserves conflict clearance and maintains car spacing in each
approach. It gives waiting conflicting approaches a turn and carries any
delay to later stops on the same route. The viewer retains the same route
geometry and closure filters. The waits and clearance are visual assumptions.
They do not change engine results or represent measured signal timing.

This coverage applies to matched ordinary car paths. Transit and event
traffic use different path clocks. Nodes too close to stop at separately and
paths missing a valid approach or exit can fall outside this rule.

[Road graph](../../data/mvp/road_graph.md) · [SUMO signal phases and clearance](https://sumo.dlr.de/docs/Simulation/Traffic_Lights.html)
