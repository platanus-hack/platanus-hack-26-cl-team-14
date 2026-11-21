# Census study roles and home weights

The 2024 Census measures education attendance in four age bands. For every home comuna, `census_study_by_comuna.csv` counts residents in each band with a positive attendance indicator. The last column counts how many of those residents also belong to the engine's worker matrix. Divide that column by the resident count within the same band and comuna to estimate the share with both roles. [INE person archive and manual, sections 4.3.13–4.3.16](../../context/research/sources/od-flows-admin.md#censo-2024-person-microdata)

| Positive indicator | Age | RM residents | Also internal workers |
| --- | --- | ---: | ---: |
| Preschool | 0–5 | 218,933 | 0 |
| Basic | 6–13 | 707,609 | 0 |
| Secondary | 14–17 | 326,265 | 2,399 |
| Higher education | 18–24 | 348,660 | 46,936 |

A positive indicator has INE code 1. `no_response` records code −99. `suppressed` records code −66. `no_positive_indicator` includes people outside an indicator's age band. The Census columns cannot identify students who attend a given level outside its age band. SIES measures registrations of all ages.

An internal worker has `p44_lug_trab` 2 or 3 and a work comuna in the RM. Each person enters one row once. The 52 comuna sums match `communes.csv` and the RM-internal origins of `work_od.csv` exactly. They total 7,400,741 residents and 2,688,814 workers. The roster does not record trips, school destinations, or campus attendance on a particular day.

`higher_ed_home_blocks.csv` gives the published `n_asistencia_superior` weight for every block key in `blocks.csv`. The 52,646 keys and their comuna codes match. The published weights total 348,657. One suppressed block weight is blank. Preserve that blank. The comuna containers account for 3,312 of the published count. Their coordinates in `blocks.csv` are comuna estimates. The person aggregate above supplies the resident margin. [Block source and geography](README.md#blockscsv)

From the repository root, run `python3 -I tools/pack/census_study.py --raw data/raw`. The builder reads the local archives `censo2024-personas/personas_censo2024.zip` and `verify-people-work/census-blocks.zip` under that directory. It writes only the two aggregate CSVs.
