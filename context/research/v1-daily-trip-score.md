# V1 daily trip score against EOD 2012

First run of the comparison designed in [v1-daily-trip-comparison.md](v1-daily-trip-comparison.md). V1 done-when: "daily trips match the real origin-destination data for Santiago".

## Command

```sh
python3 -I tools/pack/eod_od_compare.py /path/to/EOD_STGO
```

`EOD_STGO` is the extracted [survey mirror](https://users.dcc.uchile.cl/~egraells/gds-data/eod2012.tgz) (`viajes.csv`, `personas.csv`).

## Method

- **Model.** The engine places workers and students so that comuna flows equal `mode_od.csv` (Census 2024) and `school_od.csv` (MINEDUC 2026) exactly ([population.rs](../../engine/src/population.rs)). Each of the 45 survey comunas lies in one EOD sector ([zones.csv](../../data/mvp/eod_sectors/zones.csv)), so the sector pairs of the model follow from the comuna flows.

## Exclusions by geography

| Purpose | Model: end in the 7 RM comunas outside the survey | Model: destination outside the RM | Survey: end outside sectors 1–7 (weighted trips) |
| --- | ---: | ---: | ---: |
| Work | 70,085 | 51,640 | 119,978 |
| Study | 41,364 | 136,153 | 55,606 |

The scores below describe trips inside the comparison area only. Edwin-06 (Vicente's critics) reproduced the four extra counts from the source files.
- **Survey.** Weekday trips with `Proposito` 1 (al trabajo) or 3 (al estudio), both ends in sectors 1–7. Weight = `viajes.FactorLaboralNormal` × `personas.Factor_LaboralNormal`.
- **Score.** Total variation distance (TV) between the share distributions over the 49 directed sector pairs. 0 = same shares, 1 = no overlap.

## Result

| Purpose | Model | Survey | TV sector pair | TV origin | TV destination |
| --- | ---: | ---: | ---: | ---: | ---: |
| Work | 2,618,729 persons | 2,869,579 trips (12,865 records) | **0.090** | 0.054 | 0.057 |
| Study | 1,235,972 persons | 1,529,722 trips (7,935 records) | **0.179** | 0.132 | 0.150 |

Work destination share by sector, model vs survey: Norte 15.8 % / 13.7 %, Poniente 14.9 / 13.4, Oriente 29.0 / 29.9, Centro 13.6 / 18.1, Sur 11.6 / 12.0, Sur-Oriente 9.8 / 9.6, Ext. Sur-Poniente 5.2 / 3.3. The largest pair gaps are trips inside Oriente (9.2 % vs 12.6 %) and inside Centro (2.9 % vs 5.0 %).

Work mode share (TV 0.113), model 2024 vs survey 2012: car 29.4 % / 30.6 %, public 53.4 / 42.1, walk 9.0 / 16.4, bicycle 3.7 / 5.0, other 4.5 / 5.9. Survey groups follow the [work_departures mapping](../../data/mvp/README.md).

## Limits

- Different years and units: Census 2024 usual workplace (one per person) vs 2012 diary trips. A survey worker can make two trips to work in one day (after lunch), which adds short trips and walking.
- Study: the model has only school enrollment. The survey also has higher education, which concentrates in Centro and Oriente. This explains most of the Centro gap (5.9 % vs 15.2 %).
- Survey trips to work can start away from home; model origins are homes.
- The score uses inputs the model was built from (Census comuna flows), so it measures agreement with an older independent survey, not prediction error. V4 owns the fixed event score.
