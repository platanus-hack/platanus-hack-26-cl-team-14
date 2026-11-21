# L5: afternoon school enrollment rejected

Keep L4b. L5 increased absolute error by **75.4611841167 boardings** on the
frozen 1,123 observed cells at 31 stations. No candidate code or artifact is merged.

Candidate: `3205b4528a48cee66c40486ead4b2837899d931f`, branch `feat/station-l5`.
Baseline: accepted L4b, merge `795398c4d288fa790f837c6d17724208447c9761`.
Final contract was posted on 2026-11-21 at 09:40:36 UTC for a 09:55 UTC run.
Independent Linux execution began at **09:55:16 UTC**, after the scheduled time.

## Frozen mechanism and input audit

Allocate `enrollment_no_preschool` from each school among Metro parents within
1,000 m, using the existing L4b inverse-distance weights and 50 m floor.
Multiply station enrollment by its fixed training class's afternoon profile.
The feature is zero in bins 0–23. Fit one nonnegative coefficient to observed
training-cell residuals of L4b. The code uses no held-out boarding values.

The fitted coefficient is `0.015928025620241863`. Assigned enrollment mass is
464,668 (Rust sum `464668.00000000006`). Independent geometry found 1,241 nearby
schools, including 246 zero-mass rows. Enrollment is pupils, not observed trips.

A pre-run diagnostic reported 1,243 schools and 465,754 pupils using per-point
cos(latitude) and ky=110574. The frozen code uses fixed kx=111320*cos(-33.45°)
and ky=110540. RBD 9307 (166 pupils) lies 1000.650613 m from LQU; RBD 25571
(920 pupils) lies 1000.906396 m from LME. Both are excluded by the frozen rule.
The data owner reproduced the corrected counts. No formula or radius changed.

Source review confirms the school term is fixed in both MUT baseline and scenario;
home/job changes do not remove it. Serving was not acceptance-tested because the
candidate failed the score gate.

## Actual result

| Layer | Absolute error | Half-hour WAPE | Daily WAPE | AM error / full denominator | PM error / full denominator |
| --- | --- | --- | --- | --- | --- |
| L4b | 254347.4405635792 | 0.4459345830326435 | 0.3382643522121866 | 0.2581783447138252 | 0.1877562383188188 |
| L5 | 254422.9017476959 | 0.4460668853337780 | 0.3377210229732622 | 0.2581783447138252 | 0.1878885406199530 |

Denominator: 570,369.4 average weekday boardings. Morning predictions are exactly
unchanged. The frozen adoption rule requires at least 1,000 fewer error boardings,
or error at most 253347.4405635792. This is a prospective engineering threshold,
not significance testing. L5 is worse, rather than a small gain below the threshold.
The repeatedly used holdout remains development validation.

## Reproduce at the candidate commit

```sh
cargo run --release --manifest-path engine/Cargo.toml --bin stations -- data/mvp /tmp/l5.json 1000 public classes split inverse schools
cargo run --release --manifest-path engine/Cargo.toml --bin stations -- data/mvp /tmp/l4b.json 1000 public classes split inverse
python3 -I tools/pack/check_score.py /tmp/l5.json
python3 -I tools/pack/check_score.py /tmp/l4b.json
```

Independent Linux prediction SHA256:
- L5: `392edc5103a7d435d324c257df4cea1095f4ad80b7cca4ac8f92b7e496a3a0fc`.
- L4b: `6ad1688be39afc840344c4d133a2f7af48d85eba28d1e5f24d22da976bc3ae22`.

The comparator matches the previous Linux L4b predictions byte for byte.
Model `engine/src/stations.rs` SHA256: `a57ae9404869428445a9a84fab9b3cb25ac3c1bef7047c75c148aa78452b742a`.
Frozen scorer `tools/pack/check_score.py` SHA256:
`ea807561049a3e62a847455b98863594744ae1768201a498f3d720af2991dcaa`.

| Input under `data/mvp` | SHA256 |
| --- | --- |
| work_od.csv | `442b1a12ad6b65820c173bf1c4805bb93dadf2b2dc53ae7967147196a8010038` |
| mode_share.csv | `340ae79468bc66975120ff755aa1e0f611718f9d8836e6572ea61158967cd3d1` |
| blocks.csv | `ca8314b26c470d243369dfb62d68e2477e2f14aaa017c2cde19456a0d33ed57b` |
| jobs.csv | `4420a3956bc414601e8bf07bcdc99eddc3c865aebad47d2f137b2fc7ed15c481` |
| metro_training.json | `6811e2044d2e777d6f0b350493c43c2ca7a4eb0bc892f6f876bb4f7c4bf35639` |
| schools.csv | `d95ce837d3ca2437a0fad6eb4b1eb1e36d899936887b17d65422a120fee0027b` |
| metro_observations.json | `ecd8131fac2bfcf401bb5700dda7a7a0a31534990b8b962134d31449651ef5a3` |
