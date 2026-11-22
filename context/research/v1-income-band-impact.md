# Who changes, by household income band

`optional_visits.workers_by_household_income_band` counts workers in each imputed household income band, and how many of them change work mode, stop shopping or change their shopping visit. Each count comes with its share of the band, so bands of different size compare directly.

## Band draw by home comuna

The band of each worker is drawn from the CASEN 2024 workers who live in the same comuna (`data/mvp/casen_worker_income.csv`, `tools/pack/casen_worker_income.py`). The filter, thresholds and weight are those of [v1-income-burden.md](v1-income-burden.md): RM, age 15 or more, `activ` 1, `nucleo` > 0, `ymonecorh` in November 2024 CLP, `expr`. The comuna comes from the public commune file, joined on `folio` and `id_persona`. All 19,841 workers join; the 52 comunas have 50 to 1,467 sample rows. The draw uses comuna only, not comuna × age: 639 of the 832 comuna × age × band cells have fewer than 30 rows ([geographic limits](v1-income-burden.md#geographic-limits-of-the-worker-bands)). The four band totals equal the regional table (514,418 / 992,632 / 1,347,280 / 1,446,122). [Commune file](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/casen_2024_provincia_comuna.dta)

Commune is not a CASEN survey domain. The ministry warns that direct commune estimates can be unreliable or biased. The table is a model input for the draw, not a published commune estimate. [Survey manual](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/Manual_para_la_Investigacion_Casen_2024.pdf)

The earlier draw by age only (#497) gave the same response in every band: the band did not depend on where the worker lives, works or travels. Age is no longer used for the band. Workers under 15, with unknown age or unknown private household stay `unmodeled`.

## Result (release CLI, data/mvp)

| Scenario | Band 1 | Band 2 | Band 3 | Band 4 |
| --- | ---: | ---: | ---: | ---: |
| Workers | 334,498 | 638,245 | 855,372 | 858,912 |
| Metro closed 07:00–10:00: change work mode | 3.71% | 3.96% | 4.16% | 4.20% |
| All buses removed: change work mode | 3.55% | 3.32% | 3.09% | 2.48% |
| All buses removed: stop shopping | 0.59% | 0.56% | 0.54% | 0.44% |

The bus removal moves a 43% larger share of band 1 than of band 4. The Metro closure moves a slightly larger share of the upper bands. No change gives zero in every row.

The band is an imputed household trait. It does not enter any choice, and it does not measure a person's income, wage or cash for a trip.

## Extra cash per workday

Each band row also gives the change in daily round-trip cash for the workers who change work mode: `extra_cash_clp_per_workday` (sum), `extra_cash_clp_per_priced_changer` (mean) and `extra_cash_1_day_pct_of_band_median_monthly_income`. Prices, in 2026 CLP:

| Mode | Daily round trip | Source |
| --- | --- | --- |
| Public | 2 × 795 | Red bus fare with transfers, 2026 ([v1-optional-visit-fares.md](v1-optional-visit-fares.md)) |
| Car | 2 × km × 1,414 / 16.9 | Gasoline 93, Región Metropolitana, September 2026: 1,414 CLP/l ([CNE monthly report, October 2026, p. 13](https://www.cne.cl/wp-content/uploads/2026/10/RMensual_v202610.pdf)). New light vehicles 2025: 16.9 km/l ([ICCT](https://theicct.org/chile-acelera-su-transicion-hacia-un-transporte-mas-limpio-el-reto-ahora-es-no-perder-impulso-sept26/)). km = engine straight-line distance home comuna → job comuna |
| Walk, bicycle | 0 | |
| Taxi, colectivo, other | no price | counted in `work_mode_changes_unpriced` |

Car cost is a floor: road distance is longer than straight-line, the fleet is older than new vehicles, and parking and tolls are not included. Metro peak fares (+100) are not included. The percentage is for one workday, because a closure lasts one day. It converts the 2026 mean to November 2024 CLP (IPC 105.83 / 114.13) and divides it by the band's worker-weighted median monthly household income ([v1-income-burden.md](v1-income-burden.md)).

| Metro closed 07:00–10:00 | Band 1 | Band 2 | Band 3 | Band 4 |
| --- | ---: | ---: | ---: | ---: |
| Workers who change mode | 12,409 | 25,271 | 35,552 | 36,096 |
| Mean extra cash per priced changer, CLP/day | 601 | 595 | 570 | 343 |
| One day, % of band median monthly income | 0.100% | 0.054% | 0.030% | 0.009% |
| Unpriced changers | 1,870 | 3,557 | 5,042 | 4,128 |

With all buses removed, the mean cash change is negative in every band (−81 to −11 CLP/day): many changers walk, which costs no cash. The engine does not price the extra time.
