# Worker trips for the city view

Call `population::sample_trips(&population, 300)` on the baseline and on the
population after workers choose again. Each row contains home and job coordinates
in `[longitude, latitude]`, a mode, a representative departure half-hour (0–47),
and the number of synthetic workers it represents. Use `count` for totals.
`origin_cut` and `destination_cut` are the commune codes of the representative
home and job points. Use this pair to look up a display path. Some workers in a
group can have different home or job communes.

The function groups workers by mode, then by proximity of both endpoints on a
1-km grid. It splits that spatial order into groups of similar size. It snaps
the mean home and job location independently to a known point from that group.
Each row covers at least two workers. Students have no observed commute mode
in the source and stay outside this view.

For each mode, the displayed times follow that mode's weighted distribution of
weekday work departures in the 2012 Santiago travel survey. A group's time is a
display assignment. The Census gives no departure times for these workers.
The survey and mode mapping are documented in the [data pack](../../data/mvp/README.md).

The row budget limits the number of groups. When it permits, each mode with at
least two workers gets a row. With a smaller budget, larger modes take priority.
The sum of returned counts gives the workers covered. Identical states give
identical rows. A row index can represent different workers across scenarios.

## Baseline with 300 rows

| Mode | Workers | Weighted share | Share of dots |
| --- | ---: | ---: | ---: |
| Car | 799,249 | 29.725% | 29.667% |
| Public | 1,416,237 | 52.671% | 53.000% |
| Walk | 243,674 | 9.063% | 9.000% |
| Bicycle | 102,545 | 3.814% | 3.667% |
| Other, including unknown | 127,109 | 4.727% | 4.667% |

The run covered all 2,688,814 synthetic workers. The smallest group contained
8,907. The 300 groups took 142 ms and used 48,753 JSON bytes on the review Mac.
Two rows covered the two largest modes and 2,215,486 workers. Zero rows covered
zero workers.

In a separate run, we raised Las Condes job weights by 20% and ran
`rechoose` from `d02ca5e`. It moved 52,847 workers to a different job point.
The groups covered the same total. Public transport rose by 6,938 workers,
and 257 of 300 representative endpoint pairs changed. Grouping took 156 ms. We matched both representative CUT codes and coordinates
to the original point tables in all 300 baseline and 300 changed groups.
The existing travel-time matrix covers 128 of 159 public-mode CUT pairs in the
baseline display and 126 of 159 in the changed display.
