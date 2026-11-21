# Stable groups for the city view

Build one `TripPlan` per baseline and row budget. Call `sample` on each state,
then match rows by `id`. Counts cover current workers. Representative endpoints
come from current group members. The plan keeps the baseline mode, spatial
boundaries and departure slot. It holds one entry per group.

Equal spatial keys stay together. An empty group has no row. One remaining
worker joins an adjacent occupied group of the same mode. A mode with fewer
than two workers, or outside the baseline row budget, has no row.
Rebuilding the plan for a changed state loses comparability.

## MUT job removal

We set jobs to zero on block 13114131009033 and ran the worker choice code
from `11f8a16`. The scenario removed 4,679 modeled jobs. It moved 4,681 synthetic
workers to different job points. Both states had 2,688,814 workers.

| Row budget | Equal endpoint pairs, old → fixed groups | Equal full rows, old → fixed groups | Median home shift, old → fixed groups | Base + changed compact JSON |
| ---: | ---: | ---: | ---: | ---: |
| 300 | 43 → 290 | 0 → 18 | 3.24 km → 0 | 51,338 + 51,338 bytes |
| 2,000 | 299 → 1,924 | 132 → 409 | 0.43 km → 0 | 343,842 + 343,843 bytes |

The old comparison used output positions. The fixed comparison matched IDs.
At 2,000 rows, the plan took 33 ms to build. Applying it took 92 ms per state
(median of three warm release runs). These bytes cover only the two arrays.
In separate runs, adding and removing 100 workers gave totals of 2,688,914 and
2,688,714. Repeating the baseline gave identical rows.
