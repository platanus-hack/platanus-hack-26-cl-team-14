# Daily trip comparison with the Santiago travel survey

This design supports the V1 daily travel criterion and the MUT replay. It uses the 2012 origin-destination survey as a historical reference. SECTRA checked survey trip volumes, transfers and trip distances against other records. The daily trip comparison below still needs a model run. [SECTRA survey and downloads](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/) · [SECTRA methods paper](https://estudiosdetransporte.org/sochitran/article/view/141)

## Compare trips on common terms

1. Use baseline travel on a normal weekday. Keep trips with both ends in the seven survey sectors. Map modeled endpoints through the same sector boundaries. Report excluded survey and model mass separately. [Point-to-sector tables](../../data/mvp/eod_sectors/README.md)
2. Count one record per movement from an origin to a destination. A transfer is a stage within that movement. A complete tour contributes two movements. Keep the return-home purpose on the return leg. Apply the same purpose and aggregate-mode code mapping to both sources.
3. Join survey trips to people by `Persona`. Sum the product of the positive person factor `Factor_LaboralNormal` and trip factor `FactorLaboralNormal`. For rates per resident, sum the person factor once per eligible person, including people with no trips. Label any engine rate that uses workers or students instead of residents. [Survey weight implementation](../../tools/pack/mapocho_corridor.py) · [Tour eligibility](../../data/mvp/eod_tours/README.md)
4. Compare trips per weekday by purpose first. For each purpose, compare the share in each directed sector pair using the total trips for that purpose as the denominator. Use the union of observed and modeled cells and track missing geography separately. Show mode and departure half-hour as separate distributions when the engine provides both fields. Freeze the geographic and purpose mappings before comparing model layers.

## Check the two model limits

- **Tour generation.** Reconcile the generated tours against the retained tour tables by purpose and home sector. Compare rates per eligible person before comparing expanded totals. The tables use the outbound weight once per pair. In a separate all-trip comparison, each survey movement uses its own weight. Of 12,859 complete pairs, 5,510 have different leg weights. The published tables retain 1,940,581 of 2,830,083 weighted complete tours. [Tour coverage and weights](../../data/mvp/eod_tours/README.md)
- **Whole day.** Show trips from unsupported purposes and linked stops in the survey as coverage gaps. Keep those trips in the survey total. The current worker display groups exclude students. A worker headcount from the Census and a weighted survey trip count have different units. [Current scenario output](../../engine/src/scenario.rs)

The first comparison checks whether the generator uses its inputs as intended. The second measures descriptive agreement with an older survey. If the full survey informed the fit, use new observations to measure independent travel prediction error. Keep the frozen station score protocol for station boardings. [Station score protocol](../../engine/README.md)
