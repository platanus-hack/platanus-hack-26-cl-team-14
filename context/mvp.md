# MVP for the night of 2026-11-21

- Status: Proposed. Agreed in the thread `Goal: datos reales + MVP Rust (6–8h)` by witty-llama (coordinator and engine owner), clever-heron, and gentle-fox. The contract is from witty-llama, version 1.
- Humans sleep. Agents iterate until the humans return. Do not stop after the first version.

## What a human does with the MVP

1. The human opens one page. The page shows the comunas of Santiago in 3D. The height of a comuna is its population or its jobs. Arcs show the work flows between comunas (Census 2024).
2. The human selects a comuna (the story is Providencia and the MUT) and moves 2 large controls: jobs and housing.
3. The Rust engine runs again and returns the time that it used. The scene shows the new flows.
4. The page says clearly which values are observed and which values are estimates of the model.

First loop: comuna scale. Later loops, only after review: blocks, jobs inside a comuna, buildings near Tobalaba, and observed boardings as a separate layer with its own label. No reviewed mechanism predicts station boardings from the MUT change yet. Do not show one as a result.

## Parts and owners

| Part | Output | Owner |
| --- | --- | --- |
| Comunas | `data/mvp/communes.csv`, `data/mvp/communes.geojson` | gentle-fox |
| Work matrix | `data/mvp/work_od.csv` | gentle-fox |
| Schools | `data/mvp/schools.csv`, `data/mvp/school_od.csv` | gentle-fox |
| Blocks (follow-up) | `data/mvp/blocks.csv` | gentle-fox |
| Jobs inside a comuna (follow-up) | `data/mvp/jobs.csv` | gentle-fox |
| Buildings near Tobalaba (follow-up) | `data/mvp/buildings.geojson` | gentle-fox |
| Transport | boardings and stops export | clever-heron |
| Engine | Rust core and HTTP API on port 8787 | witty-llama |
| Scene | `web/`: one page, no build step, calls the API | gentle-fox builds; bright-condor reviews in a browser |
| Pitch proposal | `context/research/pitch-proposal.md` | gentle-fox |
| Critique | Each PR gets one critic from a different session | all |

## Data contract (witty-llama, version 1)

- Plain CSV, UTF-8, comma, one header line. GeoJSON in EPSG:4326, in a separate file. One file is 10 MB or less.
- `cut` is the 5-digit INE comuna code as text (Providencia is `13123`). SII codes are different; convert them first.
- `communes.csv`: `cut,name,population,households,dwellings,longitude,latitude`
- `communes.geojson`: one polygon for each comuna, property `cut`.
- `work_od.csv`: `origin_cut,destination_cut,workers`. Census 2024, only `p44_lug_trab` 2 or 3 with a known comuna.
- `schools.csv`: `rbd,name,cut,longitude,latitude,enrollment`
- `school_od.csv`: `origin_cut,destination_cut,students`
- `blocks.csv`: `block_id,cut,longitude,latitude,population,dwellings,workers,students` (`block_id` is the INE Census 2024 manzana code as text)
- `jobs.csv`: `block_id,cut,longitude,latitude,jobs,method`. The jobs of a comuna sum to the workers that arrive there in `work_od.csv`.
- Do not invent a missing value. Leave the column out and say so.
- Each file has a builder in `tools/pack/` (Python standard library), and a section in `data/mvp/README.md`: source, year, columns, row count, limits, and the command that builds it.
- Raw files stay in `data/raw/` (git ignores it). No rows about one person. Aggregates only.

## Engine and scene contract (witty-llama, version 1)

- The Rust engine serves HTTP on `localhost:8787`.
- `GET /api/world` returns metadata, `zones[{id,name,longitude,latitude,population,households,dwellings,workers,jobs}]`, and `flows[{origin,destination,workers}]`.
- `POST /api/simulate` with `{zone_id,jobs_multiplier,housing_multiplier}` returns the same `zones` and `flows`, a summary, and the measured `elapsed_ms`. A multiplier of 1 is the base.
- The page labels observed base values and model estimates differently.
- The model conserves the totals that it must conserve and says its assumptions. A replay of the fitted matrix is not a validation.
- There is no second contract. Until the engine runs, the scene reads a file with the same shape as `GET /api/world`, built from the real files, and shows the word "placeholder".

## Rules

- Golden rule: if the demo does not show it and nothing in the demo needs it, do not build it.
- The simulation core is Rust. Data builders and the page are not the core.
- No tests, no defensive code. Run the real thing one time and post the numbers.
- Real data, not fake data. A placeholder must say "placeholder" on the page and in the file.
- Post what you take in the thread before you start. Criticize the work of other agents with evidence.
