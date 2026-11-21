# Which Santiago tables can support the venue comparison?

At main commit `833df69`, the work tables support moving modeled jobs between two blocks. The packed data give no joint distribution of fan origin, income and travel mode for either venue. Keep the event audience fixed across the two scenarios. Treat a change in its travel mode as a model result.

| Packed table | Join and coverage | What the join measures |
| --- | --- | --- |
| work_od.csv and mode_od.csv | Census 2024. Group the latter by origin_cut and destination_cut before joining the 9,128 pairs. | Commutes from RM residents. Of 2,740,454 commuters, 2,688,814 work within the RM. Filtering destinations to RM changes the denominator. |
| jobs.csv and jobs_2026.csv | Join by block_id. Sum by workplace cut to compare with RM census destinations. | Estimated job locations. The April 2026 variant uses work arrivals at training Metro stations to adjust the distribution within each comuna. Both tables retain the same comuna totals. Neither gives venue payroll. |
| income.csv | Join cut to the **home** comuna. CASEN 2015 has 52 rows, including 26 with estimates and 26 outside the survey domain. | Mean monthly household monetary income in November 2015 pesos. Preserve missing values. Use for historical residence context. |
| vehicles.csv | All 52 CUTs in 2024. The numerator uses the permit issuer and the denominator uses household residence. | Alhué issued 18.8 permits per resident household. This ratio cannot give the probability that a fan has a car. |
| eod_destinations.csv | EOD 2012. Group by home_cut and purpose. Of 1,141 rows, 86 lack destination_cut. Retain their weight in the original denominator. | Expanded trips for shopping, health and errands. The packed file lacks an income band, a person key and a stadium-event purpose. |

For income-specific travel behavior, link trips to people and households **within the original EOD survey**. Report weighted shares and unweighted support by group. The retained weekday tour data come from diaries collected between June 2012 and November 2013. A comuna join across EOD, CASEN and Census cannot recover individual income or a match audience.

The review checked the CSV headers, row counts and missing destination keys against this checkout. It also checked the local income, vehicle and survey documentation. It did not reproduce the upstream surveys or estimate a behavioral coefficient.

[Packed data definitions](../../data/mvp/README.md) · [CASEN income](commune-income.md) · [Survey coverage and weights](../../data/mvp/eod_tours/README.md)
