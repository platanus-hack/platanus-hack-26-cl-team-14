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
It reads both CSV files once at startup. Each scenario starts from the observed baseline.

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

Data files have not arrived yet. The response marks their vintages as unconfirmed until the owner supplies provenance.

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

## Manual check status

Release compilation passed with Rust 1.100.0 on 2026-11-21. The boundary-coverage rebuild took 1.99 seconds with two build jobs.
Real-data API execution is pending delivery of the input CSV files. No runtime result is claimed yet.
No fixture data or automated tests are included, as required by the repository rules.
