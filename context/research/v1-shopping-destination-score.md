# V1 shopping destination score

The MUT → Monumental replay needs a way to measure where shopping trips go. Use destination total variation (TV) to measure baseline agreement with Santiago's EOD 2012 weekday survey. Keep the existing Metro boarding score for changes to station predictions. [SECTRA survey catalog](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/) · [Santiago destination choice brief](destination-choice.md)

## Fixed comparison

- Use purpose 11 in `viajes.csv`. Count trips to shop across all modes. Use the person's home comuna as the origin key, even if the trip started elsewhere. Count each recorded shopping trip once. The pack includes trips without a matched return.
- Multiply the trip's `FactorLaboralNormal` by the person's field of the same name. Use positive weights and mapped home and destination comunas. This reproduces the [Huff fit sample](../../tools/pack/shopping_huff.py).
- In [eod_destinations.csv](../../data/mvp/eod_destinations.csv), select `purpose=shopping` and a mapped `destination_cut`. Sum `expanded_trips` by home and destination CUT. The selection has 5,782 sampled trips and 1,588,690.257732 expanded trips across 45 home comunas. Keep all 52 destination CUTs in [communes.csv](../../data/mvp/communes.csv). An eligible destination with no survey weight contributes zero to the reference share.
- Use the same home totals for the observed and modeled distributions when assessing destination choice alone. Multiply each home comuna's observed trip total by its modeled probability for each destination. Sum across home comunas. Aggregate named place counts into their destination comuna once.

For each destination CUT, divide its observed and predicted totals by the corresponding regional total. Then compute `TV = 0.5 × sum(abs(predicted_share - observed_share))` across the 52 CUTs. Lower is better. Report the two regional trip totals separately. TV can stay constant when the model multiplies every trip count by the same factor.

## Reproduce and interpret

From the repository root with the survey extracted as described in the [source inventory](sources/verify-transport.md), run `python3 -I tools/pack/shopping_huff.py` in a worktree. The script prints TV for the comuna fit and rewrites the MUT origin-share pack. I reproduced the published fit result of **0.180** at `5d50a30b0bb668d9a8cabdd79007fb6da3509750`. The rebuild left the checked-in CSV unchanged. The script fits its coefficients with these same trips and evaluates the comuna model before it adds MUT as a separate choice. The 5,745 MUT trips come from the later named-place calculation with survey home totals. [Published fit and site counts](../../data/mvp/shopping_huff.md)

For a replay with trip rates multiplied by Census 2024 residents, report output TV and trip totals separately from the conditioned fit. Differences can come from the changed home totals or the choice model. The EOD comparison provides historical fit evidence. A test of current shopping patterns needs separate observations that cover the same purpose, modes, and destination units. A test of the swap needs observations for a comparable venue change.
