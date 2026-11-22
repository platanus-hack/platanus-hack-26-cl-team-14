# Vehicle yielding at Tobalaba and Larraín

The pitch shows city streets in motion. This pilot uses the shared road node
256366826 at Tobalaba and Larraín. The UOCT inventory lists a connected signal
at this junction (J071171). The packed road graph keeps the shared node and road
geometry. The inventory gives no signal schedule.
[UOCT inventory](https://www.transporteinforma.cl/wp-content/uploads/2023/10/Intersecciones-de-Semaforos-conectados-a-Sistema-UOCT-RM.pdf) · [Road graph](../../data/mvp/road_graph.md)

The viewer reserves the crossing for one routed commuter car at a time. Cars
wait upstream and pass in arrival order. The spacing and clearance times are
visual assumptions. The viewer uses the replay clock and leaves the engine,
route closures, and reported travel times unchanged. SUMO also reserves time
to clear an intersection between conflicting movements.
[SUMO signal logic](https://sumo.dlr.de/docs/Simulation/Traffic_Lights.html)

## Additional controls

| UOCT | Shared OSM node | Movement in saved MUT A |
| --- | --- | --- |
| J071171, Tobalaba/Larraín | 256366826 | Crossing streams |
| J005181, Antonio Varas/Pocuro | 332274486 | Crossing streams |
| J175111, Grecia Oriente/Américo Vespucio | 310951822 | Merge into the same exit |

The viewer reserves each site separately. A wait carries forward to the next
site on the same route. Grecia covers one mapped merge. Other approaches in
the roundabout keep their existing movement. The current locations passed a
route check for two conflicting approaches in saved MUT A. This rule has no
coverage at other nodes, for buses or for event traffic. A preliminary test
at busy Tobalaba/Apoquindo estimated a maximum wait of 856 seconds under
single occupancy, so that location stays outside this rule. The public
inventory supplies locations only.
