# Bus, Metro and car in Santiago work trips

The EOD 2012 travel diaries distinguish bus and Metro. The current worker choice groups public transport into one class. Use the survey split as a historical target if the model adds separate transit options. The table pools the 45 covered home comunas. [SECTRA Santiago survey](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/) · [Current worker mode choice](../../engine/src/choice.rs)

| Aggregate trip mode | EOD codes | Household without a car | Household with a car |
| --- | --- | ---: | ---: |
| Auto only | 1 | 7.18% | 52.72% |
| Bus only | 2, 3, 7 | 30.17% | 9.69% |
| Metro only | 4 | 6.30% | 4.95% |
| Bus and Metro | 9, 10 | 17.39% | 7.60% |
| Taxi or colectivo only | 5, 6 | 4.77% | 2.20% |
| Metro with another mode | 8, 11, 12, 13, 15 | 2.55% | 2.09% |
| Other modes | 14, 16, 17, 18 | 31.64% | 20.75% |

The denominators are 1,430,275 expanded outbound work trips from 7,372 sampled trips for households without a car, and 1,559,283 from 6,068 for households with a car. I used `Tablas_parametros/ModoAgregado.csv` from the Universidad de Chile archive mirror. [Survey mirror](https://users.dcc.uchile.cl/~egraells/gds-data/eod2012.tgz) · [Existing aggregate check](../../data/mvp/eod_car_mode.md)

To reproduce, use the trip and household filters of `tools/pack/eod_car_mode.py` and replace its three mode groups with the table above. Keep normal weekday trips with `Proposito = 1`, a mode code, and positive weight. Join `viajes.Persona` to `personas.Persona` and multiply trip `FactorLaboralNormal` by person `Factor_LaboralNormal`. Join the trip's `Hogar` to the vehicle list. Car households have vehicle type 1, 3 or 5. Divide each weighted mode total by the weighted work-trip total in its household group. [Pack method](../../tools/pack/eod_car_mode.py)

The survey gives observed choices under 2012 conditions. Household car ownership does not identify whether a particular worker can drive at a given time. The auto category includes car trips from households without a car. The dictionary has no ride-hailing app category. A model that lets a person choose among feasible alternatives needs comparable route time and cash cost for the available choices. Hold out observations when fitting that model.
