# Imputed household income in optional shopping visits

The paired worker example in `optional_visits.example_worker` includes an imputed household monetary income band. It draws from CASEN 2024 employed people aged 15 or older in Región Metropolitana (CUT 13). The source uses `expr` to weight workers by age and `ymonecorh` for monthly household monetary income. The thresholds and worker-band weights are in the reviewed input. The unit is November 2024 CLP per household per month. Zero income remains in the lowest band. [CASEN 2024](https://observatorio.ministeriodesarrollosocial.gob.cl/encuesta-casen-2024) · [Weighted input #489](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/489)

The engine builds synthetic persons from Census margins. Its demographic arrays preserve household sizes for eligible persons and carry no household ID. Each estimate describes the household income of one representative worker. It uses the stable baseline person index for both sides of the comparison. The output marks shared membership as unmodeled. A home block contains many households. Workers without a known private household size and people outside the source ages or worker population receive an unmodeled status. [Population pack](person-age-household.md)

The aggregate choice has a nonbinding spending budget. The example exposes `spending_budget_clp: null` with that status. Household monetary income, personal wage, deductions and cash available for a trip are distinct quantities. An explicit budget in a store-level fixture stays an input to `choose_visit`. No choice utility or preference uses the band.

## Reproduce the paired example

```sh
cargo build --release --locked --manifest-path engine/Cargo.toml --bin scenario
echo '[{"kind":"shops","target":{"type":"point","longitude":-70.771001,"latitude":-33.541841},"op":"add","value":-1}]' |
  engine/target/release/scenario data/mvp - |
  jq '.optional_visits.example_worker'
```

The request removes one mapped shop point in Maipú. The engine reports the change for the same worker before and after. This scenario returns one paired example and does not identify a store chosen by the worker. A change in resident stock skips the paired output because the two states no longer have the same people. [Optional shopping rule](optional-shopping-visits.md)
