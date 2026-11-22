# Age and household size of each synthetic person

Each `Person` in `engine/src/population.rs` now has `age_band` and `household_size`. Serves V1 "who they are" and the "close the full metro" replay: who is stranded, by age and household.

## Source

INE Census 2024 person microdata, `personas_censo2024.csv` ([release](https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2025/12/04/ine-publica-base-de-datos-del-censo-de-poblaci%C3%B3n-y-vivienda-2024), [ZIP](https://storage.googleapis.com/bktdescargascenso2024/personas_censo2024.zip)). Columns: `edad`, `tipo_operativo`, `id_vivienda`, `id_hogar`, `comuna`, `p44_lug_trab`, `p44_lug_trab_esp`, `p45_medio_transporte`, `asistencia_basica`, `asistencia_media`.

Source ZIP SHA-256: `7f79428cae551ba075f23d42a6d2b82acd0408ab49a6f24491974c1dc1a6b544`.

Pack: `python3 tools/pack/person_traits.py <zip>` writes `data/mvp/person_traits_workers.csv` and `data/mvp/person_traits_students.csv`.

## Method

- **Workers: observed joint, no sampling.** The script uses the same filters as `mode_share.py`. Each (home comuna, work comuna, mode) cell is split by age band and household size. All 23,467 cells sum exactly to `mode_od.csv` (2,740,454 commuters). The engine reads this file in place of `mode_od.csv`, in the same cell order, so every person keeps the same home and work point.
- **Students** come from MINEDUC `school_od.csv`, which has no age. Each student draws age band and household size from the Census students of the home comuna (basic or secondary attendance, 1,033,874 persons, the `blocks.csv` population). The draw uses the existing systematic sampler, so the result is deterministic.
- **Household size** counts the people in a private dwelling household (`tipo_operativo=2`) with the same `comuna`, `id_vivienda`, and `id_hogar`. The pack counts 2,599,621 private households with 2.826 people per household.
- Bands: age 0 under 15, 1 15–24, 2 25–44, 3 45–64, 4 65+. Household 1 to 5, 5 = 5 or more, 255 = unknown.
- Persons that a `residents` scenario adds get `UNKNOWN` (255). No source gives their traits.

## Check

Before the unknown-size change, at base `f623d40`, the `scenario` binary output is byte-identical to main except `elapsed_ms`, for `[]` and for a residents ×1.2 in Santiago plus a Tobalaba closure. Station score unchanged: WAPE_bins 0.4179, WAPE_daily 0.2968 (L9 weights, `check_score.py`).

The unknown-size update marks 1,891 worker records and 592 student records. Every worker age count within each origin, destination, and mode is unchanged. So is every student age count by home comuna. All 23,467 worker OD cells still match `mode_od.csv`. In an isolated release run, every baseline station row matches the prior build. The full Metro closure moves zero workers to a different job point. All 2,688,814 modeled workers count in the age summary. The known-size household summary counts 2,687,027.

## Limits

- 55,094 RM persons are outside private dwellings: 8,458 in the street operation (`tipo_operativo` 1) and 46,636 in collective dwellings (3). These records keep their age and carry unknown household size.
- Students' age and household are independent of their school comuna inside a home comuna.
- The mode retention rule uses a separate group for unknown household size within each home comuna and age band. Household-size summaries count known sizes. Age summaries count people with known ages.
- Income is not here. handy-agent owns the CASEN 2024 income research.
