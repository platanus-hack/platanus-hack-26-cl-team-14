# Bus, Metro, and car use by household car access

Santiago's EOD 2012 survey separates bus, Metro, and several mixed modes. The current [car-access pack](../../data/mvp/eod_car_mode.md) combines them into three engine groups. This table splits the survey modes to answer how often households with a car reported a bus or Metro work trip.

| EOD mode group | Mode codes | Households with a car | Households without a car |
| --- | --- | ---: | ---: |
| Car | 1 | 52.7% | 7.2% |
| Bus | 2, 3, 7 | 9.7% | 30.2% |
| Metro | 4 | 5.0% | 6.3% |
| Bus and Metro | 9, 10 | 7.6% | 17.4% |
| Car and Metro | 8 | 0.5% | 0.3% |
| Remaining modes | All other codes | 24.5% | 38.7% |

The three bus, Metro, and bus-plus-Metro groups account for 22.2% of work trips from households with a car. The remaining group includes walking, cycling, taxis and several mixed modes. Values are weighted shares of trips.

Use the same sample and weights as [the car-access builder](../../tools/pack/eod_car_mode.py). Read `ModoAgregado` on weekday purpose-1 trips with positive `viajes.FactorLaboralNormal × personas.Factor_LaboralNormal`. Join `Hogar` to vehicle types 1, 3 or 5. Sum the trip weights within each household group and mode group. The sample has 6,068 trips from households with a car and 7,372 from households without one. [EOD source and archive](sources/verify-transport.md)

Household ownership does not tell whether the traveler could drive the car on that trip. These are observed choices in 2012. Individual preference parameters and current Metro closure responses require a model and separate checks.
