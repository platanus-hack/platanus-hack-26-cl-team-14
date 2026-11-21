# Santiago floor-area weights missed the adoption threshold

The candidate reduced frozen half-hour error by 761.23 boardings relative to the normalized April jobs input. The registered rule required 1,000. Keep the candidate out of the runtime.

| Integrated persons input | Absolute error boardings | Half-hour WAPE | Tobalaba over observed bins |
| --- | ---: | ---: | ---: |
| Original jobs | 238,349.70 | 0.4178865487 | 39,823.55 |
| Normalized April work arrivals | 234,698.53 | 0.4114851405 | 53,717.01 |
| Santiago floor area | 233,937.31 | 0.4101505189 | 40,159.45 |

The scorer used 31 stations, 1,123 observed cells and 570,369.4 observed boardings. Tobalaba has 81,612.8 observed boardings and belongs to training. The zero-change floor-area run kept all 2,688,814 workers and moved zero.

Run `python3 -I tools/pack/jobs_landuse.py` on the land-use inputs at `bb298b9`. It uses 2021 office, commerce and industry floor area in Santiago comuna and the preregistered UK floor-area densities. Each receiving census block gets its intersection share of each source polygon's weight. The builder distributes 318,359 jobs over the covered blocks, including all 532 new receiving points (438 with positive weight and 94 with zero weight). It preserves 42,289 jobs outside the crosswalk in Santiago and all stock in the other 51 comunas. Santiago retains 360,648 jobs. The rule excludes education and health floor area. The municipal data do not specify vacancy or whether areas follow the UK gross-area convention. The densities are relative placement assumptions. The rule assigns zero weight to 31 covered blocks that previously had 10,782 jobs.

The model build was `98ae956`. The run substituted `jobs_landuse.csv` for the jobs input in an isolated pack. A local exporter read the complete baseline half-hours directly from `Scenario::load`. The candidate CSV SHA256 was `fc10bbe8564ce6aba552a54c630f811af1c8f51935753fea12a5b30071146034`. The GTFS-parent predictions SHA256 was `d6a8206652816b3d0029a97c297e2aabb508aa3635e58f6887927ca41135f971`.
