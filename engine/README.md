# Rust scenario core

This core serves the first real-data MVP. It reads commune aggregates and observed work flows.
It does not create a synthetic population or run individual agents.

From the repository root:

```sh
cargo run --release --manifest-path engine/Cargo.toml -- data/mvp
```

The current host has an isolated temporary toolchain. On this host use:

```sh
CARGO_HOME=/private/tmp/ph-cargo RUSTUP_HOME=/private/tmp/ph-rustup CARGO_BUILD_JOBS=2 /private/tmp/ph-cargo/bin/cargo run --release --manifest-path engine/Cargo.toml -- data/mvp
```

The server binds to `127.0.0.1:8787`. It allows cross-origin requests for the local demo.
An optional second argument changes the port for an independent run:

```sh
cargo run --release --manifest-path engine/Cargo.toml -- /path/to/real-data 8788
```

It reads the CSV files and `tobalaba_observations.json` once at startup. Each scenario starts from the observed baseline.

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

Known weakness of L0: nearest-only assignment gives terminal stations the outer city (Plaza Maipú predicted 202,823 against 44,664 observed).
