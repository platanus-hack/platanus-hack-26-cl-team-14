# Age and household size of each synthetic person

Each `Person` in `engine/src/population.rs` now has `age_band` and `household_size`. Serves V1 "who they are" and the "close the full metro" replay: who is stranded, by age and household.

## Source

INE Census 2024 person microdata, `personas_censo2024.csv` ([release](https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2025/12/04/ine-publica-base-de-datos-del-censo-de-poblaci%C3%B3n-y-vivienda-2024), [ZIP](https://storage.googleapis.com/bktdescargascenso2024/personas_censo2024.zip)). Columns: `edad`, `id_vivienda`, `id_hogar`, `comuna`, `p44_lug_trab`, `p44_lug_trab_esp`, `p45_medio_transporte`, `asistencia_basica`, `asistencia_media`.

Pack: `python3 tools/pack/person_traits.py <zip>` writes `data/mvp/person_traits_workers.csv` and `data/mvp/person_traits_students.csv`.

## Method

- **Workers: observed joint, no sampling.** The script uses the same filters as `mode_share.py`. Each (home comuna, work comuna, mode) cell is split by age band and household size. All 23,467 cells sum exactly to `mode_od.csv` (2,740,454 commuters). The engine reads this file in place of `mode_od.csv`, in the same cell order, so every person keeps the same home and work point.
- **Students** come from MINEDUC `school_od.csv`, which has no age. Each student draws age band and household size from the Census students of the home comuna (basic or secondary attendance, 1,033,874 persons, the `blocks.csv` population). The draw uses the existing systematic sampler, so the result is deterministic.
- **Household size** counts the persons with the same `comuna`, `id_vivienda`, `id_hogar`. Each household is on consecutive rows: 2,609,441 keys, 2,609,441 runs. RM mean: 2.836 persons per household. `blocks.csv` gives 2.847.
- Bands: age 0 under 15, 1 15–24, 2 25–44, 3 45–64, 4 65+. Household 1 to 5, 5 = 5 or more.
- Persons that a `residents` scenario adds get `UNKNOWN` (255). No source gives their traits.

## Check

At base `f623d40`, the `scenario` binary output is byte-identical to main except `elapsed_ms`, for `[]` and for a residents ×1.2 in Santiago plus a Tobalaba closure. Station score unchanged: WAPE_bins 0.4179, WAPE_daily 0.2968 (L9 weights, `check_score.py`).

## Limits

- 55,094 RM persons are outside private dwellings: 8,458 in the street operation (`tipo_operativo` 1) and 46,636 in collective dwellings (3). Their `id_hogar` groups are kept as households.
- Students' age and household are independent of their school comuna inside a home comuna.
- No engine rule reads these fields yet. A reaction that uses them is a separate task.
- Income is not here. handy-agent owns the CASEN 2024 income research.
