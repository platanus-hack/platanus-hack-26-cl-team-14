# Home and destination sampling

The engine samples synthetic home and destination points from Census-based weights.
Workers and students use the same home step, the golden ratio minus one.
Their destinations use a different step, √2 − 1. With a shared step, each point sampler
moved through its ordered rows at the same rate within an origin-destination group.

For the 75,988 workers who live and work in Santiago comuna (CUT 13101), we compared
the projection at `036f0f6` with the new sampler. We used all modes in both runs.

| Aggregate | Shared step | Different steps |
| --- | ---: | ---: |
| Pearson correlation of home and job point indices | 0.967830 | 0.000835 |
| Spearman correlation, with tied indices assigned their mean rank | 0.999929 | 0.001017 |
| Mean straight-line distance between modeled points | 1,619 m | 2,128 m |
| Sum of absolute station-bin differences from L4b, divided by the L4b city total | 0.02090% | 0.02155% |

Distances use the station model’s flat projection at latitude −33.45°.
The change removes a correlation caused by the order of rows. The Census gives no
within-comuna home-to-job pairs to validate the resulting distances.

Both runs gave 2,688,814 workers and 1,277,336 students. We counted workers by
home and work CUT, then by both CUT codes and mode, and students by home and school CUT.
All counts matched the RM-internal source cells in `work_od.csv` (2,627),
`mode_od.csv` (11,288), and `school_od.csv` (2,547).

To reproduce the station comparison, run both projections from the repo root.

```sh
out=$(mktemp -d)
cargo run --release --manifest-path engine/Cargo.toml --bin population -- data/mvp "$out/persons.json"
cargo run --release --manifest-path engine/Cargo.toml --bin station_flows -- data/mvp data/mvp/work_od.csv "$out/l4b.json"
```

Sum the absolute differences for the 48 half-hours across all 126 parent IDs.
Divide by the sum of all L4b boardings. The figure measures reproduction of L4b.
