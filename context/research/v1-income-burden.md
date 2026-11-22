# Extra travel cost by household income

CASEN 2024 can show how the same extra cash cost weighs differently on workers' household budgets. The example uses **CLP 1,000 per affected workday**. This assumed price difference and the incomes are in November 2024 pesos. The cost columns show one affected day and 20 affected days. Convert any cost observed in a later month to November 2024 pesos with the INE consumer price index before dividing by income. [CASEN income results, page 14](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/Resultados_ingresos_Casen_2024.pdf#page=14) · [INE CPI calculator](https://calculadoraipc.ine.cl/)

| Household income band | Weighted median monthly household income among workers | One day's CLP 1,000 | CLP 20,000 over 20 days |
| --- | ---: | ---: | ---: |
| Up to CLP 745,000 | CLP 560,000 | 0.179% | 3.57% |
| Over CLP 745,000 to 1,312,767 | CLP 1,030,000 | 0.097% | 1.94% |
| Over CLP 1,312,767 to 2,323,474 | CLP 1,745,000 | 0.057% | 1.15% |
| Over CLP 2,323,474 | CLP 3,584,702 | 0.028% | 0.56% |

The table shows the assumed outlay as a share of the monthly household income median in each band. Use the existing [household quartile thresholds](v1-household-income-bands.md). They divide households. The median within each band weights workers.

In the [core RData](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/casen_2024.RData), select `region = 13`, `edad >= 15`, `activ = 1`, and `nucleo > 0`. The last filter excludes live-in domestic workers and their families, as in the ministry's household worker definition. Use `expr` for each employed person and `ymonecorh` for their household's monthly monetary income. For each positive-income band, sort by income and take the first value where cumulative weight reaches half. The source SHA-256 is `638c6aef38a930ccfbf3153fe9374fe97b9d30338aa492d12f8d19b0d0cb37e4`. [Ministry manual, EMP05 and monetary household income](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/Manual_para_la_Investigacion_Casen_2024.pdf)

The sample has 19,841 workers (4,300,452 expanded). Fourteen report zero household income (2,613 expanded). A cost-to-income ratio for these workers is undefined. No selected row has missing income or weight. This is a regional worker reference. It includes workers outside the engine's fixed workplace commute scope. Lost wages require an individual earnings and contract assumption. For a one-day closure, use only the one-day cost.

## Income bands for a synthetic worker

The same CASEN worker rows give the following weighted distribution. Group the eligible rows by age and the household income thresholds above. Include zero incomes in band 1 for this distribution. Divide each cell's `expr` sum by the sum for its age group.

| Age | Survey rows | Expanded workers | Band 1 | Band 2 | Band 3 | Band 4 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 15–24 | 1,458 | 276,594 | 31,802 | 68,578 | 108,229 | 67,985 |
| 25–44 | 9,521 | 2,265,281 | 215,587 | 497,633 | 716,654 | 835,407 |
| 45–64 | 7,371 | 1,499,286 | 228,157 | 360,538 | 440,634 | 469,957 |
| 65 or older | 1,491 | 259,291 | 38,872 | 65,883 | 81,763 | 72,773 |

A synthetic worker can draw one band from these regional shares for their age. Keep that band fixed across the paired replay. The band is an imputed household trait. The source supports a regional reference for employed people aged 15 or older. It does not give a distribution for children, students who do not work, individual comunas, or the exact engine commuters.

Pass the money available for the visit as a separate input in pesos. For example, keep one stated weekly shopping budget for the same household and compare the same basket at two reachable stores. The budget is a scenario assumption. The imputed income band can describe the burden of the price change after converting both figures to the same price month.
