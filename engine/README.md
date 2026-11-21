# Rust scenario core

The [general-scenario startup fix](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/136) is in `main` at `29c56a9`.
Independent canonical and renamed-input checks passed through HTTP, real MCP, SSE and cache.
Native-view acceptance is separate. Response additions in PR #137 are still pending review.

## Start the scenario API

Use an installed Rust toolchain with Rust 2024 support and Node.js 22 or later.
From the repository root, run:

```sh
cargo run --release --manifest-path engine/Cargo.toml --bin santiago-scenario-core -- data/mvp 8787
```

Wait for `Scenario core listening on http://127.0.0.1:8787`.
Keep that terminal open. Press Ctrl+C there to stop the server.
The first argument selects the data directory; the second selects a free port.
The defaults are `data/mvp` and `8787`.

Start or reopen your local agent in this checkout with `SANTIAGO_API_URL=http://127.0.0.1:8787` in its environment.
The project config loads `tools/santiago-mcp/server.mjs`.
Ask it to call `simulate_city` with the complete `changes` list.
An empty list restores the baseline.
The MCP uses `http://127.0.0.1:8787` unless its process has `SANTIAGO_API_URL` set to another API origin.
The checked default uses L9 station access and original `jobs.csv`.
The optional April station weighting is a different configuration and has a different score.

## General-scenario HTTP routes

| Method | Route | Result |
| --- | --- | --- |
| POST | `/api/scenario` | Run `Scenario::run` once with `{ "changes": [...] }` and publish the full result. |
| GET | `/api/scenario-latest` | Return the latest result, or `null` before the first publication; `Cache-Control: no-store`. |
| GET | `/api/scenario-events` | Keep an SSE connection open for published results. |

The POST response, latest cache and SSE event carry the same full result and `revision`.
The MCP omits `trips_base`, `trips_scenario`, `trips_delta` and the station half-hour arrays, and adds `trips_omitted: true`.
Read `applied`, `not_modeled`, and `assumptions` before explaining a result.
The engine supports jobs, residents, travel minutes, existing station closures or reopenings, and events (point target, op `set`, attendance 0 or more, end half hour 0 to 47).
The event estimate in `events` is demand for the whole return period. The station deltas and the trip deltas do not include it.
New stations and new lines remain unsupported.
The output describes a model scenario, not a measured intervention.

The native city view has a separate acceptance run in progress.
Its candidate defaults to `SANTIAGO_ENGINE=http://127.0.0.1:8790`, while engine and MCP default to 8787.
A combined launch must set `SANTIAGO_ENGINE` and `SANTIAGO_API_URL` to the same selected engine origin.
Do not use the old web launcher for this server cut: its controls use removed routes and an earlier response shape.
A verified native-view command will follow its acceptance run.

## Historical MVP contracts and model records

The sections below retain earlier input, route and model records.
Their legacy HTTP routes return 404 in PR #136.
Use the general-scenario routes above for this startup cut.
Standalone binary examples below remain historical records; they were not rerun for this change.

## Inputs

`communes.csv` has these headers:

```csv
cut,name,population,households,dwellings,longitude,latitude
```

`work_od.csv` has these headers:

```csv
origin_cut,destination_cut,workers
```

CUT values are five-character strings. Coordinates use WGS84 longitude and latitude.
`communes.csv` contains the 52 RM communes. All origins must occur in this file.
The source OD file can retain valid destinations outside RM. The core counts and excludes those rows without modifying the CSV.
For valid CUT codes, the first two digits identify the region: `13` is RM.
Every destination that starts with `13` must occur in `communes.csv`; a missing internal commune fails at startup.
Work OD must include work-location codes 2/3 only. Exclude home, foreign, unknown, and no-fixed destinations.
No personal records enter the core.

Both inputs use INE Census 2024. Work OD records the usual work location, not observed trips on a specific day.
Commune coordinates are population-weighted means of non-container Census block centroids per CUT, built by `tools/pack/zones.py` alongside `blocks.csv`.
They do not locate individual homes or workplaces.
`tobalaba_observations.json` supplies the station and weekday observation rows for the site model.
The canonical file contains DTPM April 2026 boardings and a reviewed GTFS station crosswalk.
`mode_share.csv` has `cut,mode,commuters` headers. The core caches all eight Las Condes (`13114`) mode rows at startup.

## HTTP contract

`GET /api/world` returns this shape. Type names below describe fields; they are not sample observations.

```text
{
  metadata: {
    model: string,
    state: "observed_baseline" | "modeled_scenario",
    data_vintages: { communes: string, work_od: string },
    coverage: {
      source_total_workers: number, included_workers: number,
      external_destination_workers: number, external_destination_pairs: integer
    },
    observed_fields: string[], modeled_fields: string[], assumptions: string[]
  },
  zones: [{ id: string, name: string, longitude: number, latitude: number,
    population: number, households: number, dwellings: number,
    workers: number, jobs: number }],
  flows: [{ origin: string, destination: string, workers: number }]
}
```

`POST /api/simulate` accepts `{zone_id: string, jobs_multiplier: number, housing_multiplier: number}`.
Use an existing CUT and finite, positive multipliers. A multiplier of 1 keeps the baseline input.
Both multipliers at 1 return the observed zones and flows unchanged.
The response includes all world fields above, plus:

```text
summary: {
  zone_id: string, jobs_multiplier: number, housing_multiplier: number,
  total_population: number, total_households: number, total_workers: number,
  selected_zone_population_delta: number, selected_zone_households_delta: number,
  selected_zone_dwellings_delta: number, selected_zone_workers_delta: number,
  selected_zone_jobs_delta: number
},
elapsed_ms: number
```

Counts use floating-point numbers. `elapsed_ms` measures the scenario computation and response-data allocation.
It excludes CSV loading, JSON serialization, and network time. It is not a seven-million-agent benchmark.

`metadata.coverage` reports the actual source count, included internal count, and excluded external destinations.
The source must have one row per OD pair; `external_destination_pairs` counts those source rows.
Coverage stays fixed across scenarios because it describes the observed source, not scenario allocation.
`workers`, `jobs`, and `summary.total_workers` count included RM-to-RM commuters only.
They do not count all employed residents. The first model omits external commuting; it does not assert that external commuting is zero.

## Site scenario route

`POST /api/site-scenario` calls the existing `site.rs` accounting model.
It uses only the included Las Condes origin row (`13114`) for resident work destinations.
It leaves the commune API response shapes unchanged.

The exact request shape is below. `P` means an explicit parameter object; there are no numeric defaults.

```text
P = { value: number,
      source_kind: "observed_aggregate" | "historical_proxy" | "user_assumption",
      source: nonempty string }
{
  station_id: "TB-TOB", comuna_cut: "13114",
  longitude: -70.6009, latitude: -33.4180,
  baseline_jobs: P, baseline_dwellings: P, jobs_removed: P, dwellings_added: P,
  parameters: {
    occupied_share: P, persons_per_occupied_dwelling: P,
    known_commuters_per_resident: P, attendance_home: P, attendance_work: P,
    filled_job_share: P, metro_entry_share_home: P, metro_entry_share_work: P,
    access_share_home: P, access_share_work: P, maximum_access_distance_m: P
  },
  time_profile: "observed_am_06_10_pm_16_21",
  population_policy: "local_increment_unbalanced"
}
```

Successful responses contain the station, parameter provenance in `request`, scope labels, entrance distance,
and 48-bin arrays: `observed`, `base`, `scenario`, `delta`, `site_base`, `site_scenario`, `home_delta`, and `work_delta`.
Bins are local half-hours. Missing observations stay `null` across these arrays.
`flows` contains expected resident work departures by destination commune, not observed point OD or routed train loads.
The response reports baseline/scenario residents and home departures separately.
Its `elapsed_ms` measures the site module computation and allocation only. It excludes the added mode breakdown, HTTP parsing, and serialization.

The response also includes `resident_commuters_by_mode`:

```text
[{ mode_code: integer, mode_name: string, source_commuters: number, share: number,
   resident_commuters: number, daily_home_departures: number }]
```

`source_commuters` is an observed count from Census 2024 `p45_medio_transporte`.
Shares divide each source row by the computed Las Condes total, 107,707. All categories remain, including `-99` no response.
Official labels remain unchanged: car; combined public transport; walking; bicycle/scooter; motorcycle; horse/boat; other; no response.
Code 2 includes bus, micro, Metro, train, taxi, and colectivo. It cannot provide a Metro-specific share.

`resident_commuters` splits `scenario_residents × known_commuters_per_resident`, before attendance.
`daily_home_departures` splits `scenario_home_departures`, after attendance. Zero attendance preserves stock and gives zero daily flow.
Both are modeled transfers of observed mode shares, not observed modes of new apartment residents.
`resident_commuters_by_mode_source` names the source, comuna, computed source total, source/model universes,
`source_kind=observed_aggregate`, `output_kind=transfer_assumption`, and the assumption.
The source covers all dwelling kinds and all known national work destinations; the site applies its supplied RM-internal commuter assumption.
The wrapper preserves the existing site fields and request/error contract.

Model errors return HTTP 422 with `{"error":"reason"}`. Missing or malformed JSON fields use Axum's standard parser response.
Only the reviewed MUT point, weekday profile, and local increment policy are supported.
This layer makes no global conservation claim. It does not scale the full observed station total by a housing or job multiplier.
With scalar Metro/access shares, destination redistribution alone cannot change the station total.
Occupancy, attendance, Metro use, and access shares need explicit evidence or visible assumption labels.
Actual MUT jobs remain unverified. Comuna public-transport share is not a measured Metro share.

## Additional hypothetical event route

`POST /api/event-scenario` calls the cached `event.rs` model and returns its response unchanged.
The explicit request is `{"site_id":"estadio_nacional","multiplier":1}`.
The other approved site is `estadio_monumental`; multiplier must be finite and in `[0,3]`.
Module errors return JSON HTTP 422. Missing or malformed fields use Axum's standard parser response.

The fixed source is Saturday 2026-04-11. The model transfers the positive evening contrast at Parque O'Higgins.
This is an additional hypothetical event; contrast attribution and transfer are assumptions.
The source activity stays in the baseline. Multiplier zero preserves each observed value and null.
The response separates known assigned mass from sums over common known target bins.
Source hashes, masks, venue coordinates, parent distances, weights, and nullable arrays come from the module.
The caller does not recompute them. The existing L4b model and its weekday score stay unchanged.
See the [accepted contract](../context/research/extra-event-contract.md) for the full scope.

## Model

Let `h` and `j` be the housing and job multipliers for the selected commune.
Housing multiplies that commune's dwelling stock by `h`; other dwelling stocks stay fixed.
It also multiplies its residential weight by `h`. Other communes have weight 1.
Population, households, and outbound commuters each follow these weights, with separate normalization to preserve each area total.

For each origin, multiply its observed flow to the selected destination by `j`.
Normalize that origin's flow weights to its scenario outbound commuter count.
`workers` is the outbound flow sum. `jobs` is the inbound flow sum, not a count of all jobs.
This preserves the modeled area's worker total and keeps flows nonnegative for positive inputs.
Zero observed pairs stay zero. Job opportunity redistributes destinations; it does not create workers or jobs.

Housing is a relative capacity assumption, not a hard occupancy cap.
There is no calibrated vacancy, household choice, price, travel time, or migration rate.
Population and household composition do not predict a real relocation response.
The baseline reconstructs input aggregates. It is not independent predictive validation.

## Research basis

Checked on 2026-11-21. The implementation follows the small JSON server pattern in the
[Axum reference example](https://github.com/tokio-rs/axum/blob/main/examples/readme/src/main.rs)
and the [CSV reader API](https://docs.rs/csv/1.4.0/csv/struct.Reader.html).

[UrbanSim's zone model](https://cloud.urbansim.com/docs/general/documentation/urbansim%20zone%20model.html)
uses housing and job capacity along with prices, access, and household attributes.
Those choice inputs are absent from this contract. This core therefore uses an explicit proportional scenario assumption.
It does not claim to implement or validate UrbanSim.

## Earlier commune-only manual run

Release compilation passed with Rust 1.100.0 on 2026-11-21. The latest offline rebuild took 2.02 seconds with two build jobs.
The manual run used an Apple M4 with 16 GiB RAM, on `127.0.0.1:8788`.
It used actual aggregates under `/private/tmp/mvp-real-core-inputs`, not a committed replacement for the canonical data pack.
That earlier input used official commune polygon centroids; the current canonical input uses the weighted block points described above.

- Commune source: official INE `Censo2024_v2/FeatureServer/11`, queried at `2026-11-21T06:26:22Z` for `COD_REGION=13`.
  Fields: `CUT,COMUNA,n_per,n_hog,n_vp`; `returnCentroid=true`, `returnGeometry=false`, `outSR=4326`.
  The response returned 52 records with actual centroids, spatial reference 4326, and no transfer-limit flag.
- OD source: `09b99527f9b26a06bf0440f9e9317273a6461fc6:data/mvp/work_od.csv`, copied without changes.
  Source: 9,128 pairs and 2,740,454 commuters. Included: 2,627 pairs and 2,688,814 commuters.
  Excluded: 6,501 external destination pairs and 51,640 commuters.
- Commune totals: 7,400,741 people; 2,599,621 households; 2,763,387 dwellings.
- Primary INE responses, exact query URL, retrieval time, SHA256 hashes, and API responses remain in that temporary directory.

| Request | HTTP | Core time, ms | Local HTTP time, ms |
| --- | --- | --- | --- |
| World | 200 | not measured | 1.232 |
| Baseline: `13123`, jobs 1, housing 1 | 200 | 0.083958 | 0.869 |
| Changed: `13123`, jobs 0.8, housing 1.2 | 200 | 0.089167 | 0.812 |
| Reset: `13123`, jobs 1, housing 1 | 200 | 0.076833 | 0.684 |

These are single requests over an aggregate model, not a load test or a seven-million-agent measurement.
Normalized JSON comparisons found identical zones and flows for world, baseline, and reset.
The changed run returned positive counts. Area population, households, and commuters stayed conserved within floating-point rounding.
Providencia population changed from 143,974 to 172,099.1965; inbound commuters changed from 225,827 to 186,636.2626.
Those changes follow scenario assumptions; they are not observed relocation or employment changes.
Dwelling stock increased by 15,275.2, as requested by the housing input; dwelling stock is not a conserved total.

This earlier run preceded site integration and used the temporary inputs noted above.
The manual server was PID 11794 and stopped after that run.
No fixture data or automated tests are included, as required by the repository rules.

## Canonical site HTTP run

The integrated release build passed in 3.06 seconds with two jobs on the same M4 host.
Run command: `engine/target/release/santiago-scenario-core data/mvp 8788`.
Source checkout: `9015f2deef0236edd56185a9b8afbe3200b9f12f`, with this integration change.
Inputs are the canonical committed files. Observation SHA256:
`218ab6f2bd4a532d329e02925312473be97cd1cb85739962e5c77e79a54ba7ea`.

The illustrative request files and saved responses remain under `/private/tmp/mvp-site-http-run`.
Every parameter is labeled `user_assumption`, with a description that it is not measured at MUT.
Baseline jobs are an illustrative 1,000; baseline dwellings are zero.
The changed request removes 500 of those assumed jobs and adds 200 assumed dwellings.
Other assumed values: occupancy 0.9, persons per occupied dwelling 2.1, both attendance shares 0.8,
filled-job share 1, both Metro shares 0.3, both access shares 0.8, and maximum distance 500 metres.
The commuter ratio uses actual internal Las Condes origin commuters / residents:
104,980 / 296,134 = 0.3545016782942857. Applying that aggregate ratio to new site residents is explicitly an assumption.

| Request | HTTP | Module time, ms | Local HTTP time, ms |
| --- | --- | --- | --- |
| World | 200 | not measured | 1.659 |
| Commune reset | 200 | separate commune computation | 1.035 |
| Site baseline | 200 | 0.004708 | 0.397 |
| Site changed | 200 | 0.007209 | 0.351 |
| Site reset | 200 | 0.004125 | 0.383 |

Manual JSON comparisons found identical world/reset commune data, identical site baseline/reset apart from timing,
and unchanged observed arrays that match the canonical file. All 11 unobserved bins (00:00 through 05:00) stayed null.
Observed boardings sum to 81,612.8. The assumed changed run gives 81,542.5283:
home contribution +25.7283, work contribution −96, net −70.2717 boardings.
It gives 378 added residents and 107.2013 expected resident work departures across 52 destination weights.
The nearest eligible entrance is `TB-TOB:F`, at 37.9098 metres by straight-line distance.
The response says `global_conservation_claim=false` and `validation=accounting_only`.

Unsupported comuna, negative job removal, removal above the assumed baseline, and an excessive baseline contribution
each returned HTTP 422 with a specific JSON reason. No negative residual was silently clamped.
Server PID 12176 stopped after the run. These are single manual requests, not a load test, causal validation, or scene review.

## Mode output manual run

The release rebuild passed in 2.59 seconds with two jobs. Canonical source checkout: `24a0a623`, plus this change.
Saved real-data requests and responses: `/private/tmp/mvp-mode-http-run`.
Baseline, changed, reset, and attendance-zero requests returned HTTP 200; unsupported geography and negative removal returned JSON HTTP 422.
The eight returned source counts match Las Condes CSV rows and sum to 107,707, including 19 no-response commuters.
For the same illustrative changed request above, mode stock sums to 134.00163439524 and daily departures to 107.201307516192.
Car stock is 68.43841074812. At zero attendance, stock stays unchanged and daily departures sum to zero.
Manual JSON comparisons found baseline/reset identical apart from timing, and existing site fields unchanged from the prior response.
Local HTTP times were 1.043/0.677/0.408/0.381 ms for baseline/change/reset/attendance-zero, respectively, for single requests.
The existing `elapsed_ms` still covers only `site.rs`, not mode allocation. Server PID 12891 stopped after the run.

## Station prediction L0 (`stations` binary)

Predicts weekday Metro boardings for all 126 parents, 48 half hours, without reading held-out counts.

```
python3 -I tools/pack/metro_training.py          # writes data/mvp/metro_training.json, held-out bins null
cargo run --release --manifest-path engine/Cargo.toml --bin stations -- data/mvp /tmp/pred_l0.json
python3 -I tools/pack/check_score.py /tmp/pred_l0.json
```

Mechanism (clever-heron's v0 contract): RM-internal `work_od.csv` origin totals split by `blocks.csv` population, destination totals split by `jobs.csv` jobs, each point to its nearest Metro parent. 00:00–12:00 from homes, 12:00–24:00 from jobs, times a training-only profile, times one scale fitted on the 95 training stations. Needs `blocks.csv` and `jobs.csv` in the data directory.

Score at first run: WAPE_bins 0.5001, WAPE_daily 0.4198 (trivial training-mean profile: 0.6245 / 0.5494). Scale 0.4514. L1 (pre-registered by clever-heron at 07:02, no radius search): add `1000` as the last argument; points more than 1,000 m from their nearest parent add nothing, and their mass is not redistributed. Assigned mass: homes 1,045,650 of 2,688,814, jobs 1,416,392. Scale 0.9979. WAPE_bins **0.4736**, WAPE_daily 0.3552.

L2 (formula frozen at 07:11 UTC, cleared by clever-heron 07:13): add `public` after the radius. Each RM-internal OD pair is multiplied by the mode-2 share of its home comuna (`mode_share.csv`, all rows, −99 in the denominator), on the home side and the job side. Assumption: the home comuna's share applies to all its internal destinations; destination-specific shares are not observed. Mode 2 is broad public transport, not Metro. Assigned mass: homes 597,156, jobs 772,562. Scale 1.7865. WAPE_bins **0.4688**, WAPE_daily 0.3429.

L7 (formula frozen 07:31, cleared by clever-heron 07:32): add `classes`. r_s = job mass / home mass of the station inside 1,000 m (L2 masses; home mass 0 → work). Cut = median of the 95 training ratios (48th sorted value) = 0.8145; above the cut = work class, ties = home. Each class has its own training-only profile, one shared scale. Training classes 48 home / 47 work; held-out 13 / 18. WAPE_bins **0.4669**, WAPE_daily 0.3421. One held-out observed cell (Manquehue, bin 6) has no training support in any class and is predicted 0, as in L2.

L2b, **rejected** (registered 07:17, run 07:31 UTC): homes 1,000 m, jobs 500 m, else L2. Code `5288e37` on branch `feat/station-l2b` (not merged), args `1000/500 public`, `jobs.csv` of main `ef34915`, predictions SHA256 `657736c0993c425151592db99e44c2d4f393a241873ebfba7cd267f1075a5a64`. WAPE_bins 0.5202, WAPE_daily 0.3808: worse than L2, so not kept.

L4, **kept** (formula frozen 08:15, cleared by clever-heron and calm-condor, run 08:30 UTC): add `split`. A block or job point gives mass/n to each of the n parents within 1,000 m (same distance function as L1), not all to the nearest; n = 0 → excluded. Classes, class profiles and the scale are refitted on training stations as in L7. Assigned mass equals L7 (homes 597,156, jobs 772,562). Mass with 2 or more parents in reach: homes 63.6%, jobs 74.5%. Cut 0.9025; training classes 48 / 47, held-out 14 / 17. Scale 1.7763. Predictions SHA256 284cb5cf375e8439…. WAPE_bins **0.4464** (L7 0.4669), WAPE_daily 0.3500 (L7 0.3421). Held-out error by half day, share of observed: 00–12 0.2752 → 0.2550, 12–24 0.1916 → 0.1913. The MUT scenario uses the same equal split.

L4b, **kept** (registered by gentle-fox 08:38 from bright-heron, formula frozen 08:50, cleared by clever-heron, run 09:05 UTC): add `inverse` after `split`. Shares among the parents within 1,000 m are (1/max(d, 50 m)) / Σ(1/max(d, 50 m)) instead of equal parts. Refit as L4. Mass equals L4. Cut 0.8171; classes 48 / 47 training, 10 / 21 held-out (L4: 14 / 17). Scale 1.7745. Predictions SHA256 53036db1da75dfb7…. WAPE_bins **0.4459** (L4 0.4464), WAPE_daily 0.3383 (L4 0.3500). Held-out error boardings 254,586 → 254,347; by half day 00–12 0.2550 → 0.2582, 12–24 0.1913 → 0.1878. Corridor group (error boardings): Alcántara −4,522, Santa Ana +731, Parque Bustamante −157. Isolated group: Manquehue −224, Cerrillos −35, Conchalí −14. The MUT scenario uses the same weights.

L5, **rejected** (formula frozen 09:40:36, cleared by clever-heron 09:41, run 09:55:12 UTC as announced; adoption rule: error boardings must fall by ≥ 1,000, a prospective engineering threshold, see the L8 record PR #85): L4b plus γ × S_s × p_class,t for 12:00–24:00 only, S_s = `enrollment_no_preschool` of `schools.csv` (SHA256 d95ce837…) within 1,000 m with the L4b shares. Args `1000 public classes split inverse schools`, code on branch `feat/station-l5` (not merged). School mass 464,668 (1,241 schools in reach under the L4b distance function, 246 of them zero). γ = 0.01593 (training-only non-negative least squares). Predictions SHA256 044c609b6fac3a5f…. WAPE_bins 0.4461 (L4b 0.4459), WAPE_daily 0.3377 (L4b 0.3383). Held-out error boardings 254,347.44 → 254,422.90 (+75.46): worse, so not adopted. The fitted school term worsened fixed half-hour error. See the [independent run record](../context/research/station-l5-rejected.md) for full hashes, exact scores and the reconciled school geometry.

L6, **not run** (gentle-fox's gate posted 09:50 UTC, decision record `context/decisions/20261121-0950-l6-mechanism-gate.md`): The owner [withdrew the proposed pair-flow reweighting at 09:56 UTC](https://discord.com/channels/1573514576400752640/1573579088684253235/1573638131171721239), before the 10:10 deadline, because it did not meet that mechanism gate. No replacement mechanism or run was proposed. `data/mvp/travel_times.csv` stays as reviewed data. Current accepted freeze candidate: L4b; the presentation revision is selected under the separate freeze decision.

Known limit of all layers (swift-fox and clever-heron, 08:51–08:55): the class profiles have no training support in bins 0–9 (00:00–05:00) and thin support at bins 10 and 47 (a class with 1 or 2 stations). The frozen target has 1 held-out observed cell in the zero-support bins (Manquehue bin 6, 0.2 boardings) and 6 in the thin ones, 7.4 boardings of 570,369.4 in total, these cells remain in the score, and observed mass alone does not bound their prediction error; a layer must not seek its gain there.

Known weakness of L0: nearest-only assignment gives terminal stations the outer city (Plaza Maipú predicted 202,823 against 44,664 observed).

### Score artifact for the page

`python3 -I tools/pack/station_score_run.py 1000 public` (L2; the label comes from the arguments) runs the binary and the frozen scorer, then writes `data/mvp/station_score_run.json`: layer, arguments, model and scorer commits, run time, target and prediction SHA256, `wape_bins`, `wape_daily` (full precision; values are not rounded), and for each of the 31 held-out stations `id,name,longitude,latitude,observed[48]` (nulls kept), `predicted[48]`, `observed_daily`, `predicted_daily` (both over the observed cells only). Observed values are joined after the run; they never enter Rust. One run per file.

### MUT station scenario (`StationModel::mut_scenario`)

`StationModel::new(data)` fits once with the arguments stored in `station_score_run.json`. It returns that file's `wape_bins` only if the file records the same `stations.rs` SHA256, the frozen target SHA256 (`ecd8131f…`, also checked on the bytes of `metro_observations.json`), the frozen scorer SHA256 of `check_score.py`, 1,123 observed cells, the same SHA256 of the 5 fit inputs (`work_od.csv`, `mode_share.csv`, `blocks.csv`, `jobs.csv`, `metro_training.json`), the layer label of its arguments, all 31 frozen held-out stations, and this fit's held-out predictions (48 finite values each, within 1e-9 boardings, for floating-point roundoff between platforms); otherwise `wape_bins` is `null`. So any edit to `stations.rs` or to an input needs a new artifact run. Request: `site_id` "MUT", `dwellings_added` ≥ 0, `jobs_removed` in 0..=jobs of `jobs.csv` row 13114131009033, `occupied_share` in 0..=1, `persons_per_occupied_dwelling` ≥ 0, `occupancy_source` (text, echoed). ΔH = dwellings × occupied × persons × (13114 internal commuters / population) × 13114 mode-2 share (L2). ΔJ = −jobs_removed × Σ_c F_c,13114·m_c / Σ_c F_c,13114. Both at that row's point, through the frozen classes, profiles, scale and radius. The MUT parent (TB-TOB) is always returned; at zero change its two arrays are equal. Manual run: `cargo run --release --manifest-path engine/Cargo.toml --bin mut_scenario -- data/mvp '<request json>'`.

### HTTP station scenario

`POST /api/station-scenario` uses this cached model. Startup fits once; requests do not refit.
The module owns scope, physical units, caps and score binding. Module errors return JSON HTTP 422 (`{"error":"reason"}`);
malformed request bodies use Axum's standard response. Existing world, commune and site routes keep their contracts.

Exact request example, matching the published UI's explicit transfer assumptions:

```json
{
  "site_id": "MUT",
  "dwellings_added": 298,
  "jobs_removed": 4679,
  "occupied_share": 0.9251,
  "persons_per_occupied_dwelling": 2.0847,
  "occupancy_source": "UI-supplied apartment-cohort transfer assumption; not measured at MUT"
}
```

Dwelling/job fields are physical counts. The example removes the current modeled `jobs.csv` count, not measured MUT employment.
No coefficient defaults or clipping are added by the caller. The fitted activity sensitivity is not a per-worker Metro probability.
This mechanism and the `/api/site-scenario` accounting route are independent; neither validates the other's intervention response.

The response preserves every module field and adds the observation join:

```text
{
  site_id, dwellings_added, jobs_removed, layer, model_args, wape_bins: number|null,
  scope_label, assumptions, delta_home_mass, delta_job_mass, job_public_factor,
  home_conversion: { occupied_share, persons_per_occupied_dwelling, occupancy_source,
    commuters_per_resident, public_share },
  stations: [{ id, name, predicted_base: number[48], predicted_scenario: number[48],
    observed: (number|null)[48], observed_daily, predicted_base_daily,
    predicted_scenario_daily, delta_daily, observed_bin_count,
    daily_sum_scope: "observed_bins_only" }],
  observed_source: { source_file, day_type, period, timezone, unit, bin_definition }
}
```

The caller loads `metro_observations.json` once into a separate parent-ID map and joins `half_hour_bins.LABORAL` after model calculation.
Those joined observations do not enter fitting or scenario calculation. Source period/timezone/unit/bin meaning come from the actual file metadata.
Every daily value uses only bins with a non-null observation. The raw 48-bin predictions stay unchanged; missing observations remain null.
Daily summaries are observed-bin sums, not estimates of missing hours or complete 24-hour totals.
Base predictions stay separate from observations so the baseline gap remains visible.
`wape_bins` is **“error del modelo base, no del escenario”**; a missing binding returns null, not an intervention score.

Manual integration check on dependency `e067620646fbe40a30649ea1ed7f6c90b2e0ce09`: release build passed in 19.98 seconds with two jobs.
The published dependency bytes were tested here; the coordinator integrates accepted latest main before caller publication.
On localhost 8788, baseline, 298 dwellings, 1,500 dwellings and reset returned HTTP 200.
Scope, negative dwelling count and removal above the 4,679 modeled-job cap returned specific JSON HTTP 422 errors.
World/commune/site checks returned HTTP 200; normalized comparisons matched prior route values and baseline/reset exactly.
All scenarios retained 11 observed nulls and the same 37 observed bins, source period `average of 2026-04-13 through 2026-04-17`.
L7 score binding remained `wape_bins=0.4668620082499278`.

| Request assumptions; all changed cases remove 4,679 modeled jobs | Observed-bin base | Observed-bin scenario | Observed-bin delta |
| --- | ---: | ---: | ---: |
| Baseline/reset: no changes | 47816.81818985732 | 47816.81818985732 | 0 |
| 298 dwellings; occupancy 0.9, persons 2.1 (illustrative) | 47816.81818985732 | 43835.907573823526 | -3980.910616033793 |
| 1,500 dwellings; occupancy 0.9, persons 2.1 (illustrative) | 47816.81818985732 | 44287.5509072491 | -3529.267282608225 |
| 298 dwellings; UI pair 0.9251, 2.0847 (transfer assumption) | 47816.81818985732 | 43838.19179014317 | -3978.6263997141455 |

Observed-bin boardings are 81,612.8 for each run, visibly above the base prediction.
For the UI pair, raw 48-bin sums are base 47816.87452620005 and scenario 43838.250906886016;
these differ from the same-observed-mask daily values and must not replace them.
Evidence with actual requests and all 48-bin responses: `/private/tmp/station-route-http-run`.
Temporary servers PID 15887 and 15931 stopped after their runs. No station module, artifact or fit parameters were changed.

### Reproduce the MUT block and flow projection

This scenario removes all 4,679 modeled jobs from block `13114131009033`.
It puts 298 new dwellings at the job point. The Las Condes apartment-block inputs give
298 × 0.9251 × 2.0847 = 574.7096790600001 residents. The exact station input is in
[`mut_block_stock.json`](examples/mut_block_stock.json).

From the repo root, build both binaries. Start the API on an unused port in one terminal.

```sh
cargo build --release --manifest-path engine/Cargo.toml --bin santiago-scenario-core --bin station_flows
engine/target/release/santiago-scenario-core data/mvp 8894
```

In a second terminal, request the changed Las Condes flows and project both states.
Set the URL to the port from the first terminal.

```sh
run_dir=$(mktemp -d)
curl -fsS http://127.0.0.1:8894/api/simulate -H 'Content-Type: application/json' \
  -d '{"zone_id":"13114","jobs_multiplier":0.9847,"housing_multiplier":1.0023}' |
  jq -r '(["origin_cut","destination_cut","workers"], (.flows[] | [.origin,.destination,.workers])) | @csv' > "$run_dir/flows.csv"
engine/target/release/station_flows data/mvp data/mvp/work_od.csv "$run_dir/base.json"
engine/target/release/station_flows data/mvp "$run_dir/flows.csv" "$run_dir/changed.json" "$(cat engine/examples/mut_block_stock.json)"
jq -s 'def network: map(add) | add; {
  tobalaba: ((.[1]["TB-TOB"] | add) - (.[0]["TB-TOB"] | add)),
  network: ((.[1] | network) - (.[0] | network))
}' "$run_dir/base.json" "$run_dir/changed.json"
```

Each file contains 48 half-hours for each of 126 GTFS parent IDs. The model projects
2,700.9 fewer weekday boardings at Tobalaba and 1,655.7 fewer across the network.
These are scenario estimates. When comparing hosts, check for equal values within
the existing tolerance of 1e-9 boardings per half-hour.

### Weekday station entries per scheduled place

Run a scenario, then select a Metro parent and a half-hour index. Index 16 starts at 08:00 in Santiago.

```sh
printf '[]\n' | cargo run --quiet --release --manifest-path engine/Cargo.toml --bin scenario -- data/mvp - |
  cargo run --quiet --release --manifest-path engine/Cargo.toml --bin station_capacity -- data/mvp LIB 16
```

The result compares modeled entries with nominal places on boardable departures in both directions. It uses the weekday timetable of April 13–17, 2026. Event estimates have a separate time window and stay outside this ratio. Train occupancy and free places are unknown. A station with an unrated line, no scheduled departures, or a closure returns a null ratio and the reason. [Schedule and data sources](../data/mvp/metro_station_service.md)
