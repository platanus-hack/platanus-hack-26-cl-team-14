# Residents who neither commute nor attend school

The engine had only internal commuters (2,688,814) and school students (1,277,336).
Santiago has 7,400,741 residents (Census 2024, `blocks.csv`). This adds the other residents as people with a home, a role, an age band and a household size.

## Source

INE Census 2024 person records, the same file and pass as `person_traits.py` (workers and students).
A resident goes to `data/mvp/person_traits_residents.csv` if they are not a basic or secondary student and not an internal commuter (work place 2 or 3 inside the RM).
Rebuild: `python3 -I tools/pack/person_traits.py <personas_censo2024.zip>`. The workers and students files do not change.

| Role | Census field | Persons |
| --- | --- | ---: |
| Under 15 | `edad` < 15 | 497,481 |
| Higher education | `asistencia_superior` = 1 | 301,724 |
| Employed without an internal commute (home, outside the RM, no fixed place) | `sit_fuerza_trabajo` = 1 | 884,233 |
| Unemployed | `sit_fuerza_trabajo` = 2 | 303,556 |
| Outside the labour force (retired, home care, other) | `sit_fuerza_trabajo` = 3 | 1,649,588 |
| Unknown | other codes | 43,870 |
| **Total** | | **3,680,452** |

## Engine

`Population.residents`: one record per person, home block sampled from `blocks.csv` population of the comuna (own sampler, as workers and students). No trip yet. The result has a `population` field with the three groups.
Check: with no changes and with the Metro window input, every other output field is identical to main. Memory +23 MB, load +0.1 s.

## Limits

- The three groups add to 7,646,602, not 7,400,741. Engine students come from MINEDUC enrollment (1,277,336), not from the Census (1,033,874), and a commuting student counts as both a worker and a student.
- These residents make no trip yet. The optional visits (#402, #416) can use them.
