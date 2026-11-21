# L3b: daily bus profile rejected

The frozen half-hour score worsened. Keep L4. Do not merge the candidate code.

Candidate: `feat/station-l3b`, commit `255d9a0196a6b33bb826f56a45127981485b273d`.
The owner ran it on 2026-11-21 at 08:45 UTC. An independent run at 08:46 UTC
reproduced its predictions byte for byte.

## Frozen mechanism

Start with L4, including its equal split within 1,000 m and its fitted classes,
profiles, and scale. Sum each station's reported nearby bus boardings over
48 bins. Use the existing 300 m stop links; divide each shared stop's mass
equally among its linked parents. Empty bus cells add no reported mass. This
does not establish zero demand or observed transfers.

For each class, calculate the mean observed training value in each bin.
Normalize these raw means once over all 48 bins. Multiply that profile by
the station's daily bus mass. Fit one nonnegative coefficient to L4 residuals
using observed training cells only. Add this component to L4 predictions.
The fitted coefficient is `0.427101365361945`.

## Actual result

| Layer | Half-hour WAPE | Daily WAPE | 00–12 error / full-day observed | 12–24 error / full-day observed |
| --- | --- | --- | --- | --- |
| L4 | 0.4463522364604021 | 0.3499777989708394 | 0.2550164600319076 | 0.1913357764284946 |
| L3b | 0.4521900083998696 | 0.2825774260697705 | 0.2457666130895868 | 0.2064233953102833 |

Both cover the frozen 31 stations and 1,123 observed cells, with denominator
570,369.4 average weekday boardings. The daily improvement does not pass the
fixed half-hour acceptance rule. This idea followed L3's held-out result;
the holdout is reused development validation, not an untouched final test.

## Reproduce at the candidate commit

```sh
cargo run --release --manifest-path engine/Cargo.toml --bin stations -- data/mvp /tmp/l3b.json 1000 public classes split bus_daily
cargo run --release --manifest-path engine/Cargo.toml --bin stations -- data/mvp /tmp/l4.json 1000 public classes split
python3 -I tools/pack/check_score.py /tmp/l3b.json
python3 -I tools/pack/check_score.py /tmp/l4.json
```

L3b prediction SHA256: `5d85297218cccbc7213a1e5f3c8170ceddce5b0046b533fe9e49664a95e1d174`.
L4 prediction SHA256: `284cb5cf375e8439417d09c62f94b0634637ca5f78ad1fb5ec438907ff749448`.
The comparator matches accepted L4 exactly. Model source `engine/src/stations.rs`:
`0ee50eef21096f714d5eb71cfc8460989e0ef8aeca5ebf6698b4c6140adcff31`.

Input SHA256 values, independently checked:

| File under `data/mvp` | SHA256 |
| --- | --- |
| work_od.csv | `442b1a12ad6b65820c173bf1c4805bb93dadf2b2dc53ae7967147196a8010038` |
| mode_share.csv | `340ae79468bc66975120ff755aa1e0f611718f9d8836e6572ea61158967cd3d1` |
| blocks.csv | `ca8314b26c470d243369dfb62d68e2477e2f14aaa017c2cde19456a0d33ed57b` |
| jobs.csv | `4420a3956bc414601e8bf07bcdc99eddc3c865aebad47d2f137b2fc7ed15c481` |
| metro_training.json | `6811e2044d2e777d6f0b350493c43c2ca7a4eb0bc892f6f876bb4f7c4bf35639` |
| bus_activity.csv | `f14dfe0d81547fa3d07dbf94bbfd340b5bfdac7ed12177a6393d73a25493fd63` |
| bus_metro_300m.csv | `af0e1359760aa41f5026b304a3938e334d197bc667eec64736f731b87820ff0b` |
| metro_observations.json (target only) | `ecd8131fac2bfcf401bb5700dda7a7a0a31534990b8b962134d31449651ef5a3` |

Frozen scorer `tools/pack/check_score.py` SHA256:
`ea807561049a3e62a847455b98863594744ae1768201a498f3d720af2991dcaa`.
The bus component uses 1,172 linked stops and 988,442.6 reported daily
boardings. Of linked stop-bin cells, 27.4566% are empty.
