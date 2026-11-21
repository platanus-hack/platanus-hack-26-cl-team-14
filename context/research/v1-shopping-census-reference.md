# Shopping reference with Census home populations

For the MUT → Monumental replay, multiply each home comuna's EOD 2012 weekday shopping rate by its Census 2024 population. Using the same choice set as the [existing Huff model](../../data/mvp/shopping_huff.md), this produces **5,697 modeled shopping trips to the MUT** at its current site. The published 5,745 uses the original weighted survey trips as home totals.

| Home weights | Regional trips | MUT at current site | MUT at Monumental site |
| --- | ---: | ---: | ---: |
| EOD 2012 trips | 1,588,690 | 5,745 | 6,410 |
| Census 2024 people × EOD rate | 1,778,260 | 5,697 | 6,996 |

The Census calculation covers 45 home comunas. Seven have no packed shopping rate and account for 185,404 Census residents. All 52 comunas stay in the destination choice set.

Join [commune populations](../../data/mvp/communes.csv) and [shopping rates](../../data/mvp/eod_trip_rates.csv) by CUT. Multiply the resulting trip totals by the corresponding `mut_share_of_origin_shopping` in [shopping_huff.csv](../../data/mvp/shopping_huff.csv) and sum by site. The packed shares give 5,696.43 and 6,995.69. Using the unrounded probabilities from [the fit script](../../tools/pack/shopping_huff.py) at `5d50a30b0bb668d9a8cabdd79007fb6da3509750` gives 5,696.52 and 6,996.33.

With Census home totals, assign the named-place trips to the corresponding destination comuna once. Then compare regional destination shares with EOD. Total variation is 0.1903 for the current site and 0.1899 for the relocation. The EOD observation covers the historical destination pattern. The relocation has no observed result. Use these counts as a model reference when the engine uses the same choice set and weights.
