# V6 activity step core

The Pocuro replay needs a finite activity. An entry queue or visit must change a later step. The same core must accept a route with timed starts, such as the 10K. [V6 approval and contract](https://discord.com/channels/1573514576400752640/1573903663821226187/1573919223246495797)

Daganzo models queues with discrete updates to traffic counts. SUMO separates movement from stops of a declared duration and reports waiting, moving and finished people. MATSim QSim uses time steps and queues on links. The V6 core uses only scheduled arrivals, entry limits, fixed travel times and optional stops. It has no downstream storage or speed response. [Daganzo](https://its.berkeley.edu/publications/cell-transmission-model-dynamic-representation-highway-traffic-consistent-hydrodynamic) · [SUMO person stages](https://sumo.dlr.de/docs/Specification/Persons.html) · [SUMO person counts](https://sumo.dlr.de/docs/Simulation/Output/PersonSummary.html) · [MATSim QSim](https://www.matsim.org/files/book/partOne-latest.pdf)

Each boundary releases people whose travel or visit is complete. They queue for the next segment or finish the route. Then scheduled arrivals join and each segment admits people up to its per-step limit. The last boundary completes due work and closes admissions. A person who remains is still counted. A visit is a step in the route. It is separate from a sale or an observed visitor count.

The route and operator hours can come from public evidence. Arrival counts by step, entry limits and shop-stop rules need stated assumptions where observations are missing. Compare the activity against the same initial city without the activity. The activity core does not change commuter trips or estimate sales.

## Step witnesses

These inputs use known route geometry and assumed demand and movement rules. Their counts check the model mechanics. They do not estimate real attendance or shopping.

- Pocuro uses twelve mapped segments and three participants. At 09:00, two start the first segment and one waits. With one extra minute of assumed shop dwell, admissions to the next segment change from 09:10, 09:11 and 09:12 to 09:10, 09:12 and 09:13. All three exit by the end. [Pocuro input](https://discord.com/channels/1573514576400752640/1573903663821226187/1573919913855426572)
- The 10K input uses two start waves with three participants. At capacity one, exits occur at 08:04, 08:05 and 08:19. At capacity two, two exit at 08:04 and one at 08:19. [10K input](https://discord.com/channels/1573514576400752640/1573903663821226187/1573920132114423998)

In both runs and their variants, every step satisfies `entered = exited + remaining`. Remaining is the sum of waiting, moving and visiting people. At the final boundary the route closes for the activity. People still present stay in the final count. The result carries aggregate counts by segment. Geometry, mode and the motor-access rule belong in the replay metadata.
