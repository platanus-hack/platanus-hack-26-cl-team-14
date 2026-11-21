# Weekday study trips missed the station acceptance threshold

The candidate reduced absolute error by 431.55 weekday boardings across 31
held-out Metro stations and 1,123 observed half-hours. The required reduction
was 1,000. The other-purpose experiment depended on acceptance of this layer.

| Frozen metric | April job weights | With study trips |
| --- | ---: | ---: |
| Half-hour absolute error (boardings) | 234,697.773506 | 234,266.225928 |
| Half-hour WAPE | 0.4114838095 | 0.4107271988 |
| Station daily WAPE | 0.2963993983 | 0.2978340231 |
| Tobalaba boardings in observed bins (observed 81,612.80) | 53,720.26 | 53,135.91 |

The model placed 771,859 weekday return tours. Preschool accounted for 46,406,
school for 695,698, and higher education for 29,755. It left 6,254 sampled
tours unplaced because dual-role workers had no verified work return time.
The geographic inputs omitted 176,824 in-person campus enrollments and 7,772
higher-education attendees in home blocks without an assigned survey sector.
One home block count was suppressed. These sources measure different populations.

Of 20,664 Metro-only legs, 5,201 had distinct reachable entry and exit stations.
The 95 training stations gave a purpose multiplier of 5.549357. The 31 reserved
stations supplied zero observed boarding values to either fit. The score
measures repeated development validation on the frozen April target.

[Registered comparison](https://discord.com/channels/1573514576400752640/1573688249287708682/1573707510806745180) · [Tour and station rules](student-other-trip-rule.md).

## Reproduce

Use three worktrees. In the commands below, APRIL, STUDY and INPUT are their
absolute paths. Check out these commits, respectively.

- APRIL (person CLI): `5f70e3a9e004c1343d7ad4ca30922cf573b808ab`
- STUDY (candidate): `a492b6b68bc0e4d0619a1ebe722650964a3b6b48`
- INPUT (data): `ebb492b08f9f37da3d433ba7ee1e04f20e48eae9`

```sh
cd "$INPUT"
cargo run --release --manifest-path "$APRIL/engine/Cargo.toml" --bin population -- \
  data/mvp /tmp/study-april-base.json april-transport
cargo run --release --manifest-path "$STUDY/engine/Cargo.toml" --bin study_stations -- \
  data/mvp data/validation/trip_purpose_2026_04/metro_arrivals.csv \
  /tmp/study-april-base.json /tmp/study-predictions.json > /tmp/study-coverage.json
python3 -I tools/pack/check_score.py /tmp/study-april-base.json
python3 -I tools/pack/check_score.py /tmp/study-predictions.json
```

The April prediction SHA-256 was
`9e261333bec55fbd8b271a8bc4af26d52b6b3fdfc2c887d6c1eddae262614b03`.
The study prediction SHA-256 was
`55b6bf1d05084887499daca38992d5045fc91fb9f673bad80a4df99f44b1ac10`.

## Cost and an example

Three new processes on the shared Apple M3 used the same inputs. Times include
startup and file I/O. RSS is the largest resident allocation reported by macOS
for that process. The projection process reads an existing April baseline.

| Process | Elapsed ms, three runs | Peak RSS MiB, three runs |
| --- | --- | --- |
| Build population and study day, count tours | 1,180 / 1,118 / 1,188 | 117.5 / 118.1 / 117.9 |
| Build population and study day, fit and write station projections | 1,409 / 1,551 / 1,520 | 118.5 / 119.0 / 119.4 |

For a separate example, reclassify 10% of each campus's in-person enrollment
as remote. Round the remaining in-person enrollment to the nearest integer
per campus. Keep the Census home weights and travel survey profiles fixed.
Running the study-day builder on this input assigned 27,145 university return
tours, down from 29,755. This synthetic example uses a changed CSV. The public
scenario API has no campus intervention or measured response to online classes.
