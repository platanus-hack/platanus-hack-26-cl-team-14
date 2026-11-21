# Overnight MVP: one urban engine

Status: integrated baseline on main, 2026-11-21; demo story remains an agent proposal.
Coordinate in [the MVP thread](https://discord.com/channels/1573514576400752640/1573579088684253235).
This document replaces the conflicting data paths and file-based engine contract in gentle-fox's initial proposal.
It does not change the human-owned pitch.

[Native/local execution is allowed](decisions/20261121-0308-local-mvp-interface.md). A public website or deployment is not required.
The local Rust HTTP API and browser scene below are the current agent implementation choice, not a human mandate.
This permission does not start a native UI rewrite or change the active owners.

## Human direction and working assumption

- [Bruno asks agents to iterate until humans return](https://discord.com/channels/1573514576400752640/1573561159955648553/1573578948246245487), toward a solid MVP.
- [Juan asks whether the MVP is described](https://discord.com/channels/1573514576400752640/1573561159955648553/1573579028697186430).
- [Juan selects Rust](https://discord.com/channels/1573514576400752640/1573561159955648553/1573575465258717349); [the accepted decision](decisions/20261121-0247-rust-simulation-core.md) applies to simulation execution.
- [Accepted scope](decisions/20261121-1200-urban-dynamics-engine-scope.md): one shared urban engine, mutable housing/work conditions, eventual three-use-case pitch. MUT replacement and effects near Tobalaba are the accepted example.
- [Editable conditions](decisions/20261121-0115-editable-initial-conditions.md) and [a moving 3D scene](decisions/20261121-0100-demo-needs-3d-simulation.md) remain required.

`pitch.md` is still empty. The first working slice is an agent proposal: change housing/jobs in Providencia and show comuna-level redistribution in 3D.
The Providencia (`13123`) comuna demo is separate from the proposed MUT point scenario. MUT is at Av. Apoquindo 2730 in Las Condes (`13114`); use that CUT for the replacement site. See the [site evidence and mode-share limits](research/tobalaba-scenario-contract.md#site-geography-and-mode-share).
Do not claim observed MUT employment, household relocation, or station-level effects from a comuna OD matrix.
Keep this assumption explicit for the next human taste/direction checkpoint. Propose the other two use cases at the next direction checkpoint.

## Current baseline and owners

The Rust API, real-data pack, Tobalaba scenario and Santiago scene are merged. The [Release v1 checkpoint](https://discord.com/channels/1573514576400752640/1573566182496272506/1573596100990869565) records actual Chrome baseline/change/reset review, screenshots and limits. Use that checkpoint; do not post a duplicate release.

Start the API with the [engine run instructions](../engine/README.md#rust-scenario-core). In a second terminal, serve the repository root on a free port, for example `python3 -m http.server 5175`, then open `http://localhost:5175/web/`. If that port is occupied, choose another and update the URL. Data definitions and rebuild commands live in [the data README](../data/mvp/README.md).

| Part | Owner | Current responsibility |
| --- | --- | --- |
| Coordination and independent review | Main session orchestrators; Juan's clever-heron session reviews | Allocate bounded tasks, review exact published heads, and integrate accepted changes. |
| Rust API | witty-llama, Bruno's session | Maintain comuna/site contracts and mode outputs. |
| Data and web coordination | gentle-fox, Emerson's session | Publish canonical data and coordinate keen-panda's scene work. |
| 3D scene | keen-panda, coordinated by gentle-fox | Maintain the reviewed mode display; review the score view in a real browser. |
| Independent station model | lucky-llama | Own the single station predictor and the next preregistered scored layers. |
| Fixed development score | calm-condor; Juan's session reviews | Preserve frozen targets, coverage and eligibility. |
| Pitch proposal | gentle-fox and keen-koala | Propose wording separately; leave `pitch.md` to humans. |

Keep one `data/mvp/` contract. Publish small changes from separate branches; do not replace another owner's current files with an older copy.

## One data contract

Use the coordinator's [published schema](https://discord.com/channels/1573514576400752640/1573579088684253235/1573580063306424331):

| File under `data/mvp/` | Columns |
| --- | --- |
| `communes.csv` | `cut,name,population,households,dwellings,longitude,latitude` |
| `work_od.csv` | `origin_cut,destination_cut,workers` |
| `mode_share.csv` | `cut,mode,commuters` (home comuna; same filter as `work_od.csv`) |
| `schools.csv` | `rbd,name,cut,longitude,latitude,enrollment` |
| `communes.geojson` | EPSG:4326 geometry; property `cut` matches `communes.csv` |

`work_od.csv` keeps all 2,740,454 commuters. The engine models the 2,688,814 RM to RM commuters and reports the 51,640 to destinations outside the RM as external.

CSV uses UTF-8, commas and a header. CUT stays five-digit text. Coordinates use WGS84.
Report missing fields instead of inventing values. Include source/year, units, row counts, exclusions and a reproducible command in `data/mvp/README.md`.
Only small public aggregates enter git; raw/person records stay out.

Map the old proposal to this contract: `data/pack/od_work_comuna.csv` becomes `data/mvp/work_od.csv`; `comuna_cut` becomes `cut` for comuna attributes.
The former manzana `zones.csv` is now `data/mvp/blocks.csv`; the HTTP core still reads comuna aggregates. Buildings, blocks, schools and modeled job points are published with their source limits.
Transport crosswalks use March `GTFS_20260321_v3.zip`; [Metro observations](research/metro-observed-targets.md) preserve April source rows and missing bins.
Do not infer station joins by name or treat missing observations as zero.

## One core/UI contract

Use the coordinator's [HTTP v1](https://discord.com/channels/1573514576400752640/1573579088684253235/1573580063541301285), on `localhost:8787`:

- `GET /api/world`: metadata, `zones[{id,name,longitude,latitude,population,households,dwellings,workers,jobs}]`, and `flows[{origin,destination,workers}]`.
- `POST /api/simulate`: `{zone_id,jobs_multiplier,housing_multiplier}`. Return the same zone/flow structure plus summary and measured `elapsed_ms`.
- Multipliers of 1 select baseline. Zone IDs and flow endpoints use the same CUT namespace.

The [engine README](../engine/README.md#http-contract) defines current summary, site and mode fields.
`POST /api/site-scenario` serves the reviewed Tobalaba accounting model; the scene renders its response.
[Accepted v1 mechanism](https://discord.com/channels/1573514576400752640/1573579088684253235/1573581128105857168): housing weights residential mass and renormalizes the global total; jobs weight destination odds and preserve each origin worker total.
These are relative weights, not hard occupancy or calibrated relocation. No prices, vacancies or preferences are inferred.
Comuna job totals derived from OD are observed destination counts with exclusions, not verified jobs at MUT or at a building.
`web/data/run.json` is not a second engine interface. The scene consumes the HTTP response.

## Model boundaries and next work

The comuna model conserves area population, households and internal commuters when changing relative housing/jobs weights. Dwelling stock can change. Its OD source retains 51,640 external commuters; the model includes only 2,688,814 RM-internal commuters. These are usual workplace counts, not observed daily trips.

Keep two station calculations separate:

- **Tobalaba site scenario:** observed April boardings minus estimated baseline site contribution plus estimated changed site contribution. This is accounting under explicit occupancy, attendance, mode and access assumptions. It has no citywide conservation claim and no independent forecast score. Missing observed bins remain null.
- **Independent station prediction:** the [station binary](../engine/README.md#station-prediction-l0-stations-binary) uses geographic masses and training-only boarding profiles. The [canonical scorer](../tools/pack/check_score.py) freezes 31 parent IDs, 1,123 observed weekday cells and the target-file hash. WAPE (total absolute error divided by total observed boardings) compares those fixed cells; incomplete predictions are ineligible. Tobalaba, Los Leones and El Golf are excluded from holdout because the demo already used them. Repeated scoring is development validation, not an untouched final test or causal validation.

Merged evidence now available:

- [PR48](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/48) published the original L2 score artifact: WAPE_bins 0.4688. [PR52](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/52) added L7's frozen station classes and training-only profiles. [PR55](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/55) now publishes the **L7** artifact: WAPE_bins 0.46686200825. The base-score binding checks model-source, five fit-input, target and scorer hashes, layer/arguments, all 31 held-out IDs, 1,123 observed cells and matching predictions. This binds a development base score, not the validity of a changed-site effect.
- [PR50](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/50) adds served-pair travel costs from March GTFS: expected headway waits, train dwell and walking, not an exact 08:00 timetable. Missing paths are omitted, not zero. [PR53](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/53) makes modeled job allocation deterministic and marks known-high estimates; these are not verified building employment.
- [PR54](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/54) freezes the separate conditional 2002–2024 allocation experiment; [PR58](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/58) publishes independently reproduced results. [The result note](research/santiago-temporal-allocation-first-run.md) reports flow overlap 0.843726 / 0.927412 / 0.931071 for persistence / one scaling sweep / IPF. Observed 2024 margins are supplied conditions. This is retrospective allocation, not percentage accuracy, unknown-margin forecasting, or intervention validation.
- [PR57](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/57) publishes April weekday bus activity and nearby Metro links for L3. [The handoff](research/bus-activity-handoff.md) preserves missing bins and overlapping links. Concurrent boardings are an association input, not observed transfers or independent causal evidence.

Next work:

1. The mode UI review is complete in [PR44](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/44). Review [PR56's score view](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/56) in a real browser against the actual artifact. Keep resident commuter stock distinct from daily departures and broad public transport distinct from Metro.
2. Integrate the HTTP route and UI for [PR55's merged station scenario](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/55). Score binding and equal baseline/scenario arrays at zero change are complete. Keep this unvalidated sensitivity separate from the observed-plus-delta site accounting above; show its assumptions and baseline error.
3. Preregister the L3 bus mechanism through lucky-llama before scoring. Declare how overlapping station links contribute; use no held-out observations or derived profiles in fitting. Report worse scores as well as improvements on the same frozen target cells.
4. After each accepted change, review the affected real-data/browser flow and record the exact revision and measured result in the canonical thread. Post a release checkpoint for a changed presentation. No new catalog or parallel model implementation is needed.

A fitted OD replay and the site model's reconstructed baseline are not independent predictive validation. Keep model scores separate from counterfactual story claims. Humans still decide pitch taste and direction.

## Existing research that constrains this slice

[Prior art](research/sources/prior-art-simulators.md) supports reuse of activity-chain methods; its toy timings do not establish whole-city latency.
[Transport verification](research/santiago-transport-verified.md) establishes April day-type averages, half-hour bins and March GTFS calendar selection.
[People/work evidence](research/santiago-people-work.md) distinguishes residential counts from work destinations.
No new catalog pass is needed. Prefer these bounded inputs and working mechanisms over more source discovery.
