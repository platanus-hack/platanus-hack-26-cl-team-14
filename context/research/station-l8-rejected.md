# L8: measured pair modes rejected

Keep L4b. L8 increased error by **3,243.547392002 boardings** on the frozen
1,123 observed cells at 31 stations. Rejected model code is not merged.

Candidate: `f33a0acbb0c6f852c06caacfcfe36c46df58a74b`, branch `feat/station-l8`.
Baseline: accepted L4b, merge `795398c4d288fa790f837c6d17724208447c9761`.
The formula and engineering adoption threshold were frozen before the run.
The owner announced 09:30 UTC but ran at 09:25 UTC on 2026-11-21 and disclosed
that five-minute deviation. Independent Linux reproduction followed at 09:27.

## Mechanism and decision

Use measured mode-2 commuter counts for each RM-internal pair in `mode_od.csv`,
instead of total workers multiplied by the origin comuna's public share.
Sum these counts into home and job masses. Keep L4b's 1,000 m catchment,
normalized inverse distance with a 50 m floor, and training-only class,
profile, and scale refit. The candidate source follows this frozen formula.

Independent input arithmetic gives a maximum home-marginal shift of
3.9076372937% (comuna 13505), below the declared 3.91% limit. Scenario conversion
factors are 33,416/104,980 for residents and 185,775/305,601 for inbound workers;
these are comuna transfers to MUT, not measured site behavior.

| Layer | Absolute error boardings | Half-hour WAPE | Daily WAPE | AM error / full denominator | PM error / full denominator |
| --- | --- | --- | --- | --- | --- |
| L4b | 254347.4405635792 | 0.4459345830326435 | 0.3382643522121866 | 0.2581783447138252 | 0.1877562383188188 |
| L8 | 257590.987955581 | 0.4516213316415306 | 0.3540190987663750 | 0.2514870117550234 | 0.2001343198865069 |

Denominator: 570,369.4 average weekday boardings. From L8 onward, adoption
requires at least 1,000 fewer error boardings against the accepted layer.
This is a prospective engineering threshold, not a significance or noise test.
L8 is worse, not merely a smaller gain below that threshold. L4b retains its
previous acceptance rule. Repeated holdout reuse remains development validation.

## Reproduction

At the candidate commit:

```sh
cargo run --release --manifest-path engine/Cargo.toml --bin stations -- data/mvp /tmp/l8.json 1000 public classes split inverse pair_mode
cargo run --release --manifest-path engine/Cargo.toml --bin stations -- data/mvp /tmp/l4b.json 1000 public classes split inverse
python3 -I tools/pack/check_score.py /tmp/l8.json
python3 -I tools/pack/check_score.py /tmp/l4b.json
```

Independent Linux prediction SHA256:
- L8: `7032c16ea90665d9b3e05f5bc5b7686f88935556e149284f773e7107a150de65`.
- L4b: `6ad1688be39afc840344c4d133a2f7af48d85eba28d1e5f24d22da976bc3ae22`.

The comparator matches the previous Linux L4b run byte for byte. The owner's
Mac run reported the same scores but different prediction bytes; cross-platform
byte identity is not claimed. No model or scoring parameters were changed.

Candidate `engine/src/stations.rs` SHA256:
`43d2b8a3943be110b64a501e38cefaa3bebdb95de19eedf38b65ff88bd8dce45`.
Frozen `tools/pack/check_score.py` SHA256:
`ea807561049a3e62a847455b98863594744ae1768201a498f3d720af2991dcaa`.

| Input under `data/mvp` | SHA256 |
| --- | --- |
| work_od.csv | `442b1a12ad6b65820c173bf1c4805bb93dadf2b2dc53ae7967147196a8010038` |
| mode_share.csv | `340ae79468bc66975120ff755aa1e0f611718f9d8836e6572ea61158967cd3d1` |
| blocks.csv | `ca8314b26c470d243369dfb62d68e2477e2f14aaa017c2cde19456a0d33ed57b` |
| jobs.csv | `4420a3956bc414601e8bf07bcdc99eddc3c865aebad47d2f137b2fc7ed15c481` |
| metro_training.json | `6811e2044d2e777d6f0b350493c43c2ca7a4eb0bc892f6f876bb4f7c4bf35639` |
| mode_od.csv | `75e88e787f39b31f02ad3ec0ef3d836480f9999035fd9a5504037f7dc5c87de0` |
| metro_observations.json | `ecd8131fac2bfcf401bb5700dda7a7a0a31534990b8b962134d31449651ef5a3` |
