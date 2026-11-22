# V1 daily trip score against EOD 2012

First run of the comparison designed in [v1-daily-trip-comparison.md](v1-daily-trip-comparison.md). V1 done-when: "daily trips match the real origin-destination data for Santiago".

## Command

```sh
python3 -I tools/pack/eod_od_compare.py /path/to/EOD_STGO
```

`EOD_STGO` is the extracted [survey mirror](https://users.dcc.uchile.cl/~egraells/gds-data/eod2012.tgz) (`viajes.csv`, `personas.csv`).

## Method

- **Model.** The engine places workers and students so that comuna flows equal `mode_od.csv` (Census 2024) and `school_od.csv` (MINEDUC 2026) exactly ([population.rs](../../engine/src/population.rs)). Higher education: the travelling share of Census higher-education residents (69.6 % = in-person + hybrid over all SIES 2026 registrations, 209,954 persons) goes to SIES campuses in proportion to in-person + hybrid registrations, independent of home comuna; a campus without a point uses its reported comuna. Each of the 45 survey comunas lies in one EOD sector ([zones.csv](../../data/mvp/eod_sectors/zones.csv)), so the sector pairs of the model follow from the comuna flows.

## Exclusions by geography

| Purpose | Model: end in the 7 RM comunas outside the survey | Model: destination outside the RM | Survey: end outside the RM, sector 9 (weighted trips) | Survey: unlocated end, sector 0, no sector-9 end (weighted trips) |
| --- | ---: | ---: | ---: | ---: |
| Work | 70,085 | 51,640 | 12,569 | 107,409 |
| Study | 45,739 | 136,153 | 2,344 | 53,262 |

The survey dictionary (`Sector.csv`) labels code 9 "Exterior a RM" and gives no location for code 0. The scores below describe trips inside the comparison area only. Edwin-06 (Vicente's critics) and Lupa reproduced these counts from the source files.
- **Survey.** Weekday trips with `Proposito` 1 (al trabajo) or 3 (al estudio), both ends in sectors 1–7. Weight = `viajes.FactorLaboralNormal` × `personas.Factor_LaboralNormal`.
- **Score.** Total variation distance (TV) between the share distributions over the 49 directed sector pairs. 0 = same shares, 1 = no overlap.

## Result

| Purpose | Model | Survey | TV sector pair | TV origin | TV destination |
| --- | ---: | ---: | ---: | ---: | ---: |
| Work | 2,618,729 persons | 2,869,579 trips (12,865 records) | **0.090** | 0.054 | 0.057 |
| Study (school + higher education) | 1,441,551 persons | 1,529,722 trips (7,935 records) | **0.160** | 0.118 | 0.097 |

Work destination share by sector, model vs survey: Norte 15.8 % / 13.7 %, Poniente 14.9 / 13.4, Oriente 29.0 / 29.9, Centro 13.6 / 18.1, Sur 11.6 / 12.0, Sur-Oriente 9.8 / 9.6, Ext. Sur-Poniente 5.2 / 3.3. The largest pair gaps are trips inside Oriente (9.2 % vs 12.6 %) and inside Centro (2.9 % vs 5.0 %).

Work mode share (TV 0.113), model 2024 vs survey 2012: car 29.4 % / 30.6 %, public 53.4 / 42.1, walk 9.0 / 16.4, bicycle 3.7 / 5.0, other 4.5 / 5.9. Survey groups follow the [work_departures mapping](../../data/mvp/README.md).

## Limits

- Different years and units: Census 2024 usual workplace (one per person) vs 2012 diary trips. A survey worker can make two trips to work in one day (after lunch), which adds short trips and walking.
- Study: before higher education was added, TV was 0.179 and Centro had 5.9 % of model study destinations vs 15.2 % in the survey. With campus trips Centro is 11.3 %. Campus choice ignores distance from home, and Norte and Ext. Sur-Poniente remain over-represented (school enrollment).
- Survey trips to work can start away from home; model origins are homes.
- The score uses inputs the model was built from (Census comuna flows), so it measures agreement with an older independent survey, not prediction error. V4 owns the fixed event score.
