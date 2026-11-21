# MVP for the night of 2026-11-21

- Status: Proposed by gentle-fox (Emerson's session). Other agents can change it in the thread `Goal: datos reales + MVP Rust (6–8h)`.
- Humans sleep. Agents iterate until the humans return. Do not stop after the first version.

## What a human does with the MVP

1. The human opens one page. The page shows Providencia and the Tobalaba area in 3D, with real buildings.
2. Columns on the Metro stations and bus stops show the boardings for each half hour of a work day. The numbers are real (DTPM, April 2026).
3. The human presses one large control: "Sacar el MUT y poner una torre de N viviendas".
4. The Rust engine runs again in seconds. The scene shows where the new residents go to work and which stations get more or fewer boardings.
5. One number shows the score of the base run against the real boardings.

## Parts and owners

| Part | Output | Owner |
| --- | --- | --- |
| Zones | `data/pack/zones.csv`: Census 2024 manzanas of the RM | gentle-fox |
| Schools | `data/pack/schools.csv` | gentle-fox |
| Work matrix | `data/pack/od_work_comuna.csv` (Census 2024, real commutes only) | gentle-fox (done in PR #20) |
| Jobs | `data/pack/jobs.csv`: jobs for each zone | gentle-fox |
| Buildings | `data/pack/buildings.geojson`: footprints and heights near Tobalaba | gentle-fox |
| Transport | `data/pack/stops.csv`, `data/pack/boardings.csv` | clever-heron |
| Engine | `sim/` Rust crate: reads `data/pack/`, writes `web/data/run.json` | open: witty-llama's session, or gentle-fox if nobody takes it |
| Scene | `web/`: one page, no build step, reads `web/data/` | gentle-fox |
| Pitch proposal | `context/research/pitch-proposal.md` | gentle-fox |
| Critique | Each PR gets one critic from a different session | all |

## Contract for the data pack

- Plain CSV, UTF-8, comma, one header line. One file is 10 MB or less. Rust reads it with no extra step.
- Coordinates: `lon`, `lat` in WGS84 (EPSG:4326).
- Comuna key: `comuna_cut`, the 5-digit INE code (Providencia is `13123`). SII codes are different; convert them before you write the file.
- Manzana key: `manzana_id`, the INE Census 2024 code, as text.
- Stop key: `stop_id`, equal to the GTFS `stop_id` of `GTFS_20260321_v3.zip`.
- Each file has a builder script in `tools/pack/` (Python standard library) and a section in `data/pack/README.md`: columns, units, year, row count, known limits.
- Raw files stay in `data/raw/` (git ignores it). No download above 100 MB without a range request.
- No rows about one person. Aggregates only.

## Contract between the engine and the scene

`web/data/run.json`:

```json
{
  "scenario": {"remove_jobs_at": [-70.6009, -33.4180], "jobs_removed": 0, "dwellings_added": 0},
  "score": 0.0,
  "stops": [{"stop_id": "", "lon": 0, "lat": 0, "observed": [48 numbers], "base": [48 numbers], "scenario": [48 numbers]}],
  "flows": [{"from": [lon, lat], "to": [lon, lat], "persons": 0}]
}
```

`observed`, `base`, and `scenario` are boardings for each half hour of a work day. The scene shows `observed` when the engine output is not there, and says so on the page.

## Rules

- Golden rule: if the demo does not show it and nothing in the demo needs it, do not build it.
- The simulation core is Rust. Data builders and the page are not the core.
- No tests, no defensive code. Run the real thing one time and post the numbers.
- Real data, not fake data. A placeholder must say "placeholder" on the page and in the file.
- Post what you take in the thread before you start. Criticize the work of other agents with evidence.
