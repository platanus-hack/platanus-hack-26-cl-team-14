# Direct bus access to Metro stations

Adding one direct bus connection reduced half-hour boarding error by 16,010.60 on the frozen development score. Closing the three Quilicura extension stations increased boardings at Los Libertadores by 12,113 per weekday in the model. The historical comparison gives 11,839 in the reverse direction. The model overestimates combined April boardings at the three new stations by 44.8%.

## Model and inputs

L9 uses the original L4b commuter flows, home points, job weights, station classes and half-hour profiles. A point first splits its mass among open Metro parents within 1,000 m. The weight is inverse distance with a 50 m floor. A point outside walking range can reach Metro on one direct bus. The first walk can cover up to 1,000 m. The full trip to the Metro parent can take up to 30 minutes. The fastest eligible path wins. Ties use the parent ID.

The bus path includes the walk to the bus, half the scheduled headway, the ride time and the final walk of up to 300 m. Walk speed is 4.8 km/h. Both halves of the day use the 08:00 weekday bus service. The 30-minute cap and transfer behavior are assumptions. The pack contains 71,857 stop-to-parent links. [Pack and builder](../../data/mvp/README.md#direct-bus-access-to-metro-stations) · [GTFS frequency service](https://gtfs.org/documentation/schedule/reference/#frequenciestxt)

The fit blends the walk projection A with the walk-plus-bus projection B. B uses one scale to match total training boardings over observed bins. The blend minimizes absolute error across the same training cells. The smallest minimizing share of B is 0.2067312528. The scales are 1.7744717886 for A and 1.0629939821 for B. All fits use 3,430 observed cells at 95 training stations. The training file masks every held-out station. Both scenarios use the same fitted parameters. [Preregistration](https://discord.com/channels/1573514576400752640/1573579088684253235/1573671669392736327)

## Frozen April 2026 score

| Metric | L4b | L9 |
|---|---:|---:|
| Absolute boarding error across half-hours | 254,347.44 | 238,336.84 |
| Half-hour WAPE | 0.4459345830 | 0.4178640054 |
| Daily WAPE | 0.3382643522 | 0.2967913242 |

The frozen scorer covered 1,123 observed cells at 31 stations, totaling 570,369.4 weekday boardings. The adoption threshold is 1,000 fewer error boardings. This run used the original `jobs.csv`. [Frozen scorer](../../tools/pack/check_score.py)

## Quilicura extension

With FER, LCR and PQU open, Los Libertadores has 11,661.18 modeled boardings across 48 half-hours. With these three closed, it has 23,774.34. Both networks use the March 2026 bus timetable.

The historical comparison runs in the opposite direction. In 2023, Los Libertadores had 39,062.0 weekday boardings in August and 21,485.4 in November. Across 91 single-line comparison stations outside Line 3, the median November/August ratio was 0.853120. Applying this ratio to the August level gives an adjusted opening change of −11,839.18. The three new stations recorded 26,836.8 boardings in November. This comparison spans six weeks after opening. Bus routes and travel demand could also have changed. [2023 source rows](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/4374bc3/data/validation/metro_extension_2023/boardings.csv) · [Control stations](https://discord.com/channels/1573514576400752640/1573579088684253235/1573669586723934228)

| April 2026 station | Split | Model | Observed |
|---|---|---:|---:|
| Ferrocarril | Training | 8,557.31 | 4,483.80 |
| Lo Cruzat | Held out | 15,188.09 | 9,649.00 |
| Plaza Quilicura | Training | 33,355.75 | 25,309.40 |
| Three new stations | Mixed | 57,101.14 | 39,442.20 |
| Los Libertadores | Training | 11,661.00 | 28,847.60 |

Each row uses only the 36 half-hours with April weekday observations at that station. The three new stations have 57,155.98 modeled boardings across all 48 bins. The historical change is close, but the model misses the absolute level at all four stations. [April observations](../../data/mvp/metro_observations.json)

## Reproduce

Run from the repository root at `c4dc5aac8ab34e98f682348d73de968ba479c066`.

```sh
cargo run --release --manifest-path engine/Cargo.toml --bin station_access -- data/mvp /tmp/l9-open.json /tmp/l9-closed.json FER LCR PQU
python3 -I tools/pack/check_score.py /tmp/l9-open.json
```

The JSON output maps each of 126 GTFS parent IDs to 48 half-hour counts. Each closed parent has 48 zeros. To use this in a city scenario, load `stations::access::StationAccess` once. Start with `all_parent_ids()`, remove or restore known parent IDs and call `project_points(homes, jobs, &active)`. Use the same projection for both sides of a comparison. Each point contains longitude, latitude and public-transport-weighted commuter mass.

On the local release build, loading and fitting took 148 ms. Projecting all points took 44 ms for each network. These timings cover the Rust station projection. HTTP, MCP and the city view have separate owners.

## Person-level projection

The city caller also ran on 3,966,150 synthetic records. It assigned the L4b public transport share to each of the 2,688,814 workers. Students did not enter the station projection. Closing the extension added 12,114.45 weekday boardings at Los Libertadores. Reopening restored all 6,048 original station-bin values. Each projection took 42 ms on this host.

In the continuous run, reassigned walking trips contributed 1,575.43 of the 12,113.15 extra boardings at Los Libertadores. Direct bus access contributed the other 10,537.72 (87%). Using the walk-only L4b scale on the same closed network gives a change of 1,717.82.

## Same access model with the L10 job weights

Changing only the point weights to L10 gave a frozen half-hour WAPE of 0.4114697894, a daily WAPE of 0.2963945781 and 234,689.78 absolute error boardings. The frozen holdout covered the same 1,123 cells. Error fell by 3,647.07 against L9 alone and 17,368.13 against L10 alone. The fitted bus mixture was 0.1868281671. [Job weights at 211133c](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/211133c/data/mvp/jobs_2026.csv)

The modeled closure increase at Los Libertadores fell to 10,604.89. The gap from the historical magnitude grew from 274 to 1,234 boardings. April modeled boardings on matched bins were 16,531.92 at Los Libertadores, 6,495.91 at Ferrocarril, 15,071.67 at Lo Cruzat and 30,864.93 at Plaza Quilicura. The three new stations sum to 52,432.51 versus 39,442.20 observed. The weights use transit work arrivals at the training stations. Their effect combines workplace distribution and transit use.

To reproduce the combined run, copy the six inputs to a separate data directory and substitute the L10 weights in that copy. Use the original checkout for the frozen scorer.

```sh
out=$(mktemp -d)
for f in blocks.csv work_od.csv mode_share.csv metro_training.json bus_station_access.csv; do cp "data/mvp/$f" "$out/$f"; done
git show 211133c:data/mvp/jobs_2026.csv > "$out/jobs.csv"
cargo run --release --manifest-path engine/Cargo.toml --bin station_access -- "$out" "$out/open.json" "$out/closed.json" FER LCR PQU
python3 -I tools/pack/check_score.py "$out/open.json"
```

Removing all 4,619 held-out arrival rows before rebuilding the weights gave the same weight file. It has SHA-256 `c247e549425e69bed0941019a36ad42866e35ce1b64501a842596da338fa16d4`. The team knew the 2023 historical target when it chose the L9 mechanism. The closure result is a retrospective comparison. [Combined run](https://discord.com/channels/1573514576400752640/1573579088684253235/1573674966774194279)

## Line 2 extension check

The unchanged L9 projection closes EB, OB, CM and PI at once. La Cisterna rises from 21,075.95 to 38,996.44 modeled boardings per weekday (+17,920.48). The four extension stations have 83,924 modeled boardings when open. We recorded these predictions before inspecting the historical result. [Registration](https://discord.com/channels/1573514576400752640/1573579088684253235/1573678534734381057)

The April 2026 matched-bin comparison gives 83,664.43 modeled boardings against 60,255.60 observed (+38.8%) across the four extension stations. The April 2024 workbook reports 50,625 at those stations. At La Cisterna, the trend-adjusted historical opening difference is −86,642.45 in the published figures. The August 2023 workbook records separate entries for L2 and L4A. April 2024 has one station row. The available data do not establish a common definition of physical entries, so we cannot score the terminal change. [Line 2 source comparison](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/08ceb17/data/validation/metro_extension_2023/line2.md)
