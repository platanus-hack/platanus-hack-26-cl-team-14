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

## A newer source for earnings by comuna

Sheet 1 of the ministry's 2024 annual workbook gives mean monthly taxable earnings for formal dependent workers aged 15 and older for months with contributions. All 52 names in communes.csv matched a comuna row after case and accent normalization. The workbook uses names. Use the matched census CUT for a geographic join. The reported means round to CLP 2,601,600 in Las Condes and CLP 1,452,157 in Macul. Sheet 5 uses the whole working-age population as its denominator. CASEN household income uses another unit. The ministry’s resident-base methodology assigns the most probable comuna of habitual residence using the date and agreement of administrative records. It cannot locate a worker at either venue. [Resident-base methodology, section v](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/rraa/Documento_Metodologico_Base_Residentes.pdf). [Annual workbook, sheet 1](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/rraa/2024/Indicadores_Ingreso_Anual_2024.xlsx) · [Income indicator methodology](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/rraa/Indicadores_Ingresos.pdf)

## Corrections to the data inventory

At the reviewed inventory revision `f23bb00`, four distinctions affect the proposed scenarios.

- Census P44 code 1 counts 517,018 RM residents who work at home. It includes occupations performed at home and provides no counterfactual count of office commutes. The work OD pack already excludes these residents. Apply a hypothetical change in work location only to a stated eligible group.
- The pack includes car minutes by origin and destination comuna in car_minutes.csv alongside public transport times. This is a comuna-level input.
- SII payroll by employer tax address cannot measure the wage paid at a particular venue. The 2024 resident earnings series above answers a different question.
- Match reports give attendance for individual fixtures. The local [attendance note](monumental-match-attendance.md) records 39,222 spectators at one match. A complete official open series still needs verification.
