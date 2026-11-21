# Overnight MVP: one urban engine

Status: coordinator-confirmed execution contract, 2026-11-21; demo story remains an agent proposal.
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
Keep this assumption explicit for the next human taste/direction checkpoint. Propose the other two use cases after the first shared loop works.

## Owners and immediate outputs

| Part | Owner | Output and next action |
| --- | --- | --- |
| Integration and Rust core/API | witty-llama, Bruno's session; core subagent already active | Implement the HTTP contract below. Publish exact run command and model assumptions. |
| Census, geometry, work OD, schools | gentle-fox, Emerson's session | Publish the agreed CSV files and their actual available fields. Use only work-place codes 2/3 for commuting OD. Validate destination codes against the comuna dictionary; report excluded categories separately. |
| Transport | clever-heron, Juan's session | Export April boarding averages and exact March GTFS BUS joins. Preserve unmatched Metro/Tren and unmatched totals. |
| 3D scene | gentle-fox's `web/` task; calm-puma implements | Consume the HTTP contract; show baseline, changed run and reset. Sole scene owner, confirmed by witty-llama. |
| Independent browser/model review | Francisco/bright-condor and Vicente, pending acknowledgment | Review the running shared result; post concrete defects and evidence. No duplicate scene. |
| Pitch proposal | gentle-fox | Propose story wording separately; leave `pitch.md` to humans. |

Allocation evidence: [coordinator's owners](https://discord.com/channels/1573514576400752640/1573579088684253235/1573580064757391495), [gentle-fox's started tasks](https://discord.com/channels/1573514576400752640/1573579088684253235/1573580414243840170).
[Coordinator confirmation](https://discord.com/channels/1573514576400752640/1573579088684253235/1573581127711465555): gentle-fox owns `web/`; witty-llama owns `engine/` on `feat/rust-mvp-core`. Internal `data/pack/` staging is allowed; publish only one `data/mvp/` contract.

## One data contract

Use the coordinator's [published schema](https://discord.com/channels/1573514576400752640/1573579088684253235/1573580063306424331):

| File under `data/mvp/` | Columns |
| --- | --- |
| `communes.csv` | `cut,name,population,households,dwellings,longitude,latitude` |
| `work_od.csv` | `origin_cut,destination_cut,workers` |
| `schools.csv` | `rbd,name,cut,longitude,latitude,enrollment` |
| `communes.geojson` | EPSG:4326 geometry; property `cut` matches `communes.csv` |

CSV uses UTF-8, commas and a header. CUT stays five-digit text. Coordinates use WGS84.
Report missing fields instead of inventing values. Include source/year, units, row counts, exclusions and a reproducible command in `data/mvp/README.md`.
Only small public aggregates enter git; raw/person records stay out.

Map the old proposal to this contract: `data/pack/od_work_comuna.csv` becomes `data/mvp/work_od.csv`; `comuna_cut` becomes `cut` for comuna attributes.
The proposed manzana `zones.csv` becomes a later `data/mvp/blocks.csv`, not the core's comuna input.
Buildings, blocks and schools can enrich the scene after the first loop; their availability must not block the comuna baseline.
Transport uses `stop_id` from `GTFS_20260321_v3.zip`; its owner publishes the separate boarding schema before UI integration.
Do not infer station joins by name or treat missing observations as zero.

## One core/UI contract

Use the coordinator's [HTTP v1](https://discord.com/channels/1573514576400752640/1573579088684253235/1573580063541301285), on `localhost:8787`:

- `GET /api/world`: metadata, `zones[{id,name,longitude,latitude,population,households,dwellings,workers,jobs}]`, and `flows[{origin,destination,workers}]`.
- `POST /api/simulate`: `{zone_id,jobs_multiplier,housing_multiplier}`. Return the same zone/flow structure plus summary and measured `elapsed_ms`.
- Multipliers of 1 select baseline. Zone IDs and flow endpoints use the same CUT namespace.

The core owner defines summary fields before the UI depends on them.
[Accepted v1 mechanism](https://discord.com/channels/1573514576400752640/1573579088684253235/1573581128105857168): housing weights residential mass and renormalizes the global total; jobs weight destination odds and preserve each origin worker total.
These are relative weights, not hard occupancy or calibrated relocation. No prices, vacancies or preferences are inferred.
Comuna job totals derived from OD are observed destination counts with exclusions, not verified jobs at MUT or at a building.
`web/data/run.json` is not a second engine interface. The scene consumes the HTTP response.

## Dependency order and manual acceptance

1. **First data handoff:** data owners publish comuna attributes and filtered work OD. Core and UI agree on available columns. Record counts and exclusions.
2. **First complete loop:** Rust loads those files; the scene calls the API and renders the baseline in 3D. Use labeled modeled flow motion, not a static dashboard.
3. **Changed conditions:** change Providencia jobs or housing; run again; the returned values and visible flow weights change. Reset returns the baseline values.
4. **Manual review:** a second session starts from the documented commands and repeats baseline → change → reset. Record the machine, data revision and elapsed time for each run. Target response time is seconds; report the actual result without a benchmark promise.
5. **Evidence check:** labels distinguish observed Census counts, modeled flow allocation, and observed transport boardings. A run shows finite, nonnegative results and the declared conservation totals. Explain any excluded population.
6. **Iterate:** post a release with run command, 1–3 screenshots and the largest remaining defect. Assign that defect, merge the reviewed fix, and repeat until humans return.

A fitted OD replay is not independent predictive validation. Keep any baseline score separate from a counterfactual forecast.
Station boardings remain an observation layer until a reviewed mode/route/stop assignment mechanism connects the model to them.
Do not show a station-change score before that mechanism and matching observation coverage exist.

## Existing research that constrains this slice

[Prior art](research/sources/prior-art-simulators.md) supports reuse of activity-chain methods; its toy timings do not establish whole-city latency.
[Transport verification](research/santiago-transport-verified.md) establishes April day-type averages, half-hour bins and March GTFS calendar selection.
[People/work evidence](research/santiago-people-work.md) distinguishes residential counts from work destinations.
No new catalog pass is needed. Prefer these bounded inputs and working mechanisms over more source discovery.
