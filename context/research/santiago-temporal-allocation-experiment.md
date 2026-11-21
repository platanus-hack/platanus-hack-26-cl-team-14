# Preregistered conditional temporal allocation

Date: 2026-11-21. Status: preregistered implementation, not run; review and commit freeze precede the first evaluation.
This is an offline experiment, not validation of the API or a causal policy forecast.

## Frozen data and information boundary

Use all D={(i,j): i,j in the same 52 RM communes, i≠j}, in sorted origin/destination order: 2,652 cells.
F0=`data/validation/census_2002/intercommunal_2002.csv`; F1=`intercommunal_2024.csv` in that directory.
Declared SHA256 F0: `9ce6a30aa279a867fbb8dbd0f2180fdf9d0fbf66564be61f2dc32dfdbbb928ef`.
Declared SHA256 F1: `71b81b3715d00485218f11f37aa4118877387f3e89c8878395203e752c6f2b6e`.
The evaluator copies `comparison_manifest.json` as declared provenance; it does not verify hashes at runtime.
The lead records actual input, committed code and output hashes separately for the first run.
Define Ot_i=Σj Ft_ij and Jt_j=Σi Ft_ij over D only.
Allocation functions receive F0 plus observed O1/J1, never F1 pair values. Compute all fits before cell-level scoring.
Observed 2024 margins are explicit conditions, not predicted outcomes. No target shares, pair weights or tuning are allowed.

## Frozen models

- Persistence P: `Pij=O1i*F0ij/O0i`; ignores supplied J1 and enforces O1.
- One-sweep C: `gj=J1j/J0j`; `Cij=O1i*F0ij*gj/Σk(F0ik*gk)`; enforces O1.
- IPF: start at F0, scale columns to J1, then rows to O1. Repeat the same sweep.
  Stop when the maximum row/column `abs(predicted_margin-target_margin)/max(1,target_margin)` is ≤1e-10.
  Maximum 10,000 sweeps, fixed before running. C is IPF's first sweep, not a distinct allocation law.
- A zero required scaling denominator makes the method unavailable. IPF nonconvergence also makes it unavailable.
  Return a reason, attempted iteration count, null predictions and null metrics; do not assign a fake zero score.
- Preserve every F0 zero. No smoothing, new edges, mask shrinkage, external allocation or target-total renormalization.
  Equal double-margin fitting of multiplicatively related seeds would not demonstrate a different allocation law.

## Frozen outputs and claims

For available methods: full-precision prediction vector in common pair order, iteration count, predicted total,
`CPC=2ΣD min(predicted,F1)/(ΣD predicted+ΣD F1)`, summed absolute error,
and maximum absolute/relative row and column residuals against the supplied margins.
Also report F0-zero cell count, target mass/fraction on those cells, and new positive target edge count.
Unavailable methods retain the full declared domain; no altered mask or repaired support is scored.
Publish every comparator. No outcome-dependent parameter or method selection is permitted.
Basis: [equal inputs, constraints and CPC](https://arxiv.org/html/1506.04889v3), [independent temporal comparison](https://ops.fhwa.dot.gov/publications/fhwahop13015/sec8.htm).

## Cohort limits and first-run command

Use the [audited preparation](../../data/validation/census_2002/README.md), with diagonal and external/unknown/foreign mass excluded explicitly.
2002 cannot separate home workers within its diagonal; 2024 excludes home work. Full-matrix cohort equivalence is not established.
Unequal nonresponse, multiple/no-fixed workplaces and employed-student assumptions remain; snapshots do not follow the same people.
Missing 2024 pairs mean zero only under the canonical complete known-workplace aggregation assumption.
No observed housing/job response or future target-margin accuracy follows from conditional allocation.

Run only after independent code review and the commit freeze:

```sh
cargo run --release --manifest-path engine/Cargo.toml --bin historical_od -- data/validation/census_2002 /private/tmp/santiago-temporal-allocation.json
```

No predictions, metric values or outcome error analysis have been produced for this preregistration.
