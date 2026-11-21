# Compare an extra trip cost with household income

A one-time cost of CLP 1,000 represents **0.200%** of the weighted median monthly income in the lowest positive-income band and **0.028%** in the highest band. This example uses November 2024 pesos for both amounts. It shows the scale of the same expense across households. [CASEN income results](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/Resultados_ingresos_Casen_2024.pdf)

| RM monthly household income band, CLP | Positive-income sample households | Weighted median, CLP per month | One-time CLP 1,000 as % of median |
| --- | ---: | ---: | ---: |
| Up to 745,000 | 4,034 | 500,000 | 0.200% |
| 745,001–1,312,767 | 3,640 | 1,004,167 | 0.100% |
| 1,312,768–2,323,474 | 3,424 | 1,718,133 | 0.058% |
| Above 2,323,474 | 2,985 | 3,600,000 | 0.028% |

I selected CASEN 2024 records with `region = 13` and `pco1 = 1`. Use the existing band boundaries and sort positive `ymonecorh` within each band. Take the first value where cumulative `expr` reaches half of that band's positive-income weight. The 68 sample households with zero income represent 14,007 households after expansion. Keep them in the lowest band's household count and report their percentage as unavailable. The table gives household medians. A traveler distribution needs eligible person records and their weights. [CASEN input and band definitions](v1-household-income-bands.md)

For the same simulated person, report the extra fare and basket cost in pesos. If the cost is from another month, convert it into November 2024 pesos with the INE CPI before dividing by monthly household income. State how many trips or visits the calculation covers. Keep any lost wage separate, with its contract and absence assumptions. Household monetary income includes sources other than labor income and does not measure the person's cash budget. [INE CPI calculator](https://calculadoraipc.ine.gob.cl/)
