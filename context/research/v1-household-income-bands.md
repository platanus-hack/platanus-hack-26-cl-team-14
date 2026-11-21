# Household income bands for synthetic travelers

The V1 Metro replay could report who faces a longer trip after service stops. The income band would describe a modeled household trait. The survey does not measure the travel response to the October 2019 closure. [V1 and V3 scope](../verticals.md)

## Source and unit

Use CASEN 2024 `ymonecorh`, monthly household monetary income in Chilean pesos. It includes autonomous household income and state cash transfers. Select the Metropolitan Region (`region = 13`). The survey covers residents in occupied private dwellings within its survey frame. The household income indicator excludes live-in domestic workers and their family. [CASEN 2024 source and research manual, household monetary income indicator](https://observatorio.ministeriodesarrollosocial.gob.cl/encuesta-casen-2024)

Use `expr` as the regional expansion weight. Use one row per eligible household (`pco1 = 1`) to calculate household quantile thresholds. Count eligible person rows to estimate the share of people living in each band. For a commuter distribution, restrict those person rows to a documented survey population that approximates the modeled commuters. Report the sample size and check precision before publishing subdivisions. [CASEN 2024 usage note, §§2–3](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/Nota_uso_bases_de_datos_Casen_2024.pdf)

## Regional reference thresholds

I read the ministry's core RData and selected `region = 13` and `pco1 = 1`. The selection has 14,151 sampled households, including 68 with zero monetary income. None has a missing income or weight. Weights sum to 2,940,786 households. Sort by `ymonecorh` and select the first income where cumulative `expr` reaches 25%, 50% and 75% of the sum. The thresholds are **745,000**, **1,312,767** and **2,323,474** pesos per month. These are weighted sample estimates. I did not estimate confidence intervals. [Core CASEN 2024 RData](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/casen_2024.RData)

## Use in V1

- Publish the thresholds in pesos, their survey year, the base population and any excluded rows. Keep valid zero income in the denominator and report missing values separately.
- Use regional thresholds and a documented regional sampling distribution. A comuna or block label on a synthetic traveler identifies its modeled home. It does not turn regional survey income into a local measurement. Preserve the comuna-area Census fields for age and household size. [Census microdata release](https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2025/12/04/ine-publica-base-de-datos-del-censo-de-poblaci%C3%B3n-y-vivienda-2024)
- Keep the assignment stable across baseline and closure. A change in modeled travel time can then be compared across the same groups. A difference between bands requires a sourced link between income and the travel choices that change. The group label alone gives no such link.

The packed CASEN 2015 table has historical household means for 26 of 52 RM comunas. Those means cannot supply individual income bands. The 2024 comuna poverty estimates describe a different measure. [2015 packed estimates](commune-income.md) · [2024 comuna estimates](https://observatorio.ministeriodesarrollosocial.gob.cl/pobreza-comunal-2024)
