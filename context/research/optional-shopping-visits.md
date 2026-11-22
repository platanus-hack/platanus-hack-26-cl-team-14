# Optional shopping visits of the same persons before and after a change

Decision #388: work stays a fixed commitment; flexible activities can move or be skipped. This connects the visit rule of #402 to the engine for one need: weekday shopping.

## Rule

- **Who:** every worker and every other resident (#475), 6,369,266 persons. Students are left out.
- **Where:** each destination comuna has the fitted Huff utility of the shopping layer (#416): α·ln(shops+1) − β·minutes, α 0.586, β 0.2047/min. Each person has a fixed taste draw per destination (Gumbel, SplitMix hash of person and destination, own seed). The person picks the highest utility + taste. These draws reproduce the Huff shares.
- **Skip:** one more option per home comuna. Its utility is calibrated at baseline so the share of persons who shop gives the shopping-layer trips: EOD 2012 weekday, 0.246 trips per resident and 15.30% of residents with a shopping trip (#351), so 1.608 trips per shopper.
- **Mode and time:** each person draws a shopping mode and a departure half-hour from the EOD 2012 shopping profile of the home comuna. Only public-transport shoppers who leave inside the change window (all day if none) see the changed minutes. Everyone sees changed shop points.
- **Same preferences:** the same taste draws are used before and after. `choose_visit` (#402) makes each decision; time and budget limits have no Santiago source and do not bind.

## Output

`optional_visits`: shoppers before and after, same destination, other destination, stopped, started, trips (shoppers × 1.608), destinations that changed, one paired worker example (job block and work mode before and after, visit before and after), and `baseline_check`.

## Real runs (private target dir)

| Request | Same | Other destination | Stopped | Started |
| --- | ---: | ---: | ---: | ---: |
| No change | 1,104,566 | 0 | 0 | 0 |
| MUT −26 shops, Monumental +26 | 1,103,907 | 266 | 393 | 798 |
| Metro closed 07:00–10:00 | 1,101,979 | 216 | 2,371 | 0 |
| Metro closed all day | 1,053,927 | 4,277 | 46,362 | 0 |

Baseline check: destination total variation between persons and the shopping layer 0.0019; trips 1,776,135 vs 1,778,260.
Cost: +1.5 s for the first run (baseline decisions, cached), +1.2 s when shop points change. 8 threads.

## Limits

- The skip response is a logit assumption: no Santiago data measures how many people skip shopping when transit gets slower.
- Huff minutes are public-transport comuna minutes; non-transit shoppers keep baseline minutes.
- A residents change in the same request turns this off (`not_modeled`), because persons must match before and after.
- Destinations are comunas. Inside a comuna the shopping layer splits trips by shop point.
