# L3 bus activity: rejected development layer

The formula was [frozen before the run](https://discord.com/channels/1573514576400752640/1573579088684253235/1573610424077778975). The owner ran it at 08:15 UTC on 2026-11-21. Independent review reproduced the result. The candidate remains on `feat/station-l3`; its code is not merged.

Candidate commit: `79a2324b6e413b79f29e5bed255342701ca49a84`. Run on that commit:

```sh
cargo run --release --manifest-path engine/Cargo.toml --bin stations -- data/mvp /tmp/l3.json 1000 public classes bus
python tools/pack/check_score.py /tmp/l3.json
```

The comparator uses the same command without `bus`, on the same files. L7 predictions remain byte-identical to the accepted run.

| Run | Half-hour WAPE | Daily WAPE | Prediction SHA-256 |
| --- | ---: | ---: | --- |
| L7 comparator | 0.4669 | 0.3421 | `375d550c616e8dab5e17efd04703b83742c7ee5281b586ee0cea14e70edd7dc1` |
| L3 candidate | 0.4884 | 0.2890 | `76ee03a929cd7d4fcd1f1c67016eed4b6a638320023d5222298868c1b741e025` |

The frozen scorer verifies 31 held-out stations, 1,123 observed cells and 570,369.4 average weekday boardings. The acceptance metric is half-hour WAPE. Daily improvement does not change that rule. L3 is rejected; L7 remains accepted.

## Formula and independent calculation

Each stop within 300 m contributes its reported April LABORAL bus boardings to each linked Metro parent, divided equally among those parents. This forms B[station,bin]. Eight shared stops have two parents. Empty source bins contribute no reported mass; they do not establish zero demand.

First fit L7 unchanged. Then fit one nonnegative coefficient on observed training cells only:
`beta = max(0, sum((observed - L7) * B) / sum(B * B))`.
Candidate predictions are `L7 + beta * B`. The fit minimizes squared error; the evaluation uses WAPE. No radius search or held-out coefficient fit occurs.

Independent Python calculation from the actual CSVs and the reproduced L7 predictions matches all 6,048 candidate station-bin values exactly. Beta is 0.3465394412775471. There are 1,172 linked stops and 988,442.6 reported boardings in B across all parents. Of 56,256 linked stop-bins, 15,446 are empty (27.4566%). The actual Rust review run took 94.5 ms, excluding compilation. No tests were added or run.

This is concurrent nearby bus-boarding activity, not observed feeder alightings or transfers. Better daily totals and worse half-hour errors do not identify the cause of the error. No causal or future-demand validation follows.

## Exact inputs

| File | SHA-256 |
| --- | --- |
| work_od.csv | `442b1a12ad6b65820c173bf1c4805bb93dadf2b2dc53ae7967147196a8010038` |
| mode_share.csv | `340ae79468bc66975120ff755aa1e0f611718f9d8836e6572ea61158967cd3d1` |
| blocks.csv | `ca8314b26c470d243369dfb62d68e2477e2f14aaa017c2cde19456a0d33ed57b` |
| jobs.csv | `4420a3956bc414601e8bf07bcdc99eddc3c865aebad47d2f137b2fc7ed15c481` |
| metro_training.json | `6811e2044d2e777d6f0b350493c43c2ca7a4eb0bc892f6f876bb4f7c4bf35639` |
| bus_activity.csv | `f14dfe0d81547fa3d07dbf94bbfd340b5bfdac7ed12177a6393d73a25493fd63` |
| bus_metro_300m.csv | `af0e1359760aa41f5026b304a3938e334d197bc667eec64736f731b87820ff0b` |
| metro_observations.json (target) | `ecd8131fac2bfcf401bb5700dda7a7a0a31534990b8b962134d31449651ef5a3` |

Scorer `tools/pack/check_score.py` SHA-256: `ea807561049a3e62a847455b98863594744ae1768201a498f3d720af2991dcaa`. Source periods, missingness and joins are documented in [the bus activity handoff](bus-activity-handoff.md).
