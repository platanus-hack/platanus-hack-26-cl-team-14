# First frozen temporal run: conditional flow overlap

Date: 2026-11-21. All three preregistered methods were available and are reported below.
This offline check conditions on observed 2024 origin **and destination** margins; neither margin is predicted.
The current Santiago UI remains reconstruction/sensitivity. This result is not its intervention score; the API is unchanged.

Freeze commit: `6f9daa650491bf0cce15b534645129c57ad77840`, before the recorded run.
Source code SHA256: `dbc22314c4391594d3ce71ad2d7809750cd2d6e922a4888f64b310057004be6d`; current evaluator code is unchanged from that freeze.
Recorded start: `2026-11-21T07:50:30.327485Z`; finish: `2026-11-21T07:50:30.871702Z`.
Recorded wall time including process and JSON: 0.544142 seconds.
[Full result and all prediction vectors](../../data/validation/census_2002/temporal_first_run.json), [actual run provenance](../../data/validation/census_2002/temporal_first_run_provenance.json), [frozen specification](santiago-temporal-allocation-experiment.md).
Result SHA256: `8090023718971dfdce36922c8db321a88b3f202744008c66ae2e353de313c59e`.

## Domain, inputs and outcomes

All 2,652 unequal pairs among the same 52 RM communes remain in the domain.
2002 included workers: 1,362,526; 2024 target workers: 1,853,330.
Actual input SHA256, 2002: `9ce6a30aa279a867fbb8dbd0f2180fdf9d0fbf66564be61f2dc32dfdbbb928ef`.
Actual input SHA256, 2024: `71b81b3715d00485218f11f37aa4118877387f3e89c8878395203e752c6f2b6e`.
The provenance records separately checked input hashes; runtime Rust only copies declared manifest provenance.

| Method | Column→row sweeps | CPC flow overlap | Summed absolute error, workers |
| --- | ---: | ---: | ---: |
| P: persistent origin shares | 0 | 0.8437263152974139 | 579253.4161396822 |
| C: one column→row sweep | 1 | 0.9274117021749765 | 269060.140016098 |
| IPF: fit both observed margins | 17 | 0.9310709744318728 | 255496.46191235457 |

P enforces supplied O1 and ignores supplied J1. C is IPF's first sweep, not a distinct allocation law.
IPF fits both margins; its maximum relative row/column residual is 2.7123473350819293e-11, below the frozen 1e-10 threshold.
Marginal fit is not prediction. IPF's extra column constraints must remain explicit when comparing flow overlap.
The 161 zero source cells stay zero in every model. They contain 329 target workers across 108 new positive target edges.
All comparators are retained; no outcome-based method selection, tuning, smoothing or support repair occurred in this recorded experiment.

## Independent review and limits

Independent review gave a qualified pass: all 7,956 saved predictions match an independent column→row calculation exactly.
It independently confirmed the 17-sweep stopping point; direct-formula C differs only by floating-point operation order.
This checks the frozen calculation under the supplied conditions and stated source limits.
No causal housing/job effect, future-margin prediction, actual API validation, or full-matrix cohort equivalence is established.
2002 home-work ambiguity, unequal nonresponse, multiple/no-fixed workplaces and employed-student assumptions remain.
These are repeated population snapshots, not tracked residents or observed relocation.

## Reproduce the frozen calculation

Use the unchanged evaluator and committed inputs under the frozen specification:

```sh
cargo run --release --manifest-path engine/Cargo.toml --bin historical_od -- data/validation/census_2002 /private/tmp/santiago-temporal-reproduction.json
```

This publication copied the recorded artifacts without rerunning the evaluator or changing its parameters.
