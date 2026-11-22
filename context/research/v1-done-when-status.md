# V1 done-when status

Checked against `main` at `94af501` (22 Nov, 00:45 UTC). Criteria from [verticals.md](../verticals.md). Bruno asked to close V1 ([message](https://discord.com/channels/1573514576400752640/1573809708047798364/1573860885666209864)). Only the humans close it.

| # | Criterion | Evidence in main | Gap left | Owner |
| --- | --- | --- | --- | --- |
| 1 | People have declared properties from real data | Home, work or school point, mode: Census 2024 `mode_od.csv`, MINEDUC `school_od.csv`. Age band and household size: Census 2024 person records ([person-age-household.md](person-age-household.md), #364 via #368, unknown size outside private dwellings #435). 3,680,452 residents who neither commute nor attend school, with role, age and household ([residents-not-commuting.md](residents-not-commuting.md), #475). | **Income is not on a person.** CASEN 2024 bands and burden tables exist as research only ([v1-household-income-bands.md](v1-household-income-bands.md), [v1-income-burden.md](v1-income-burden.md)). Residents have no trips yet. | open |
| 1b | People react from their properties | Who changes mode follows the Census share of their (home comuna, age, household) group (#369). Temporary closures keep job choice on the permanent network (#368). | Flexible visits: rule #402 is draft; engine connection queued (rigorous-agent, after `scenario.rs` release). Shopping layer #416 draft behind #371. | rigorous-agent, handy-agent |
| 2 | Daily trips match real OD data for Santiago | Work trips vs EOD 2012 by directed sector pair: **TV 0.090**. Study: **TV 0.179** ([v1-daily-trip-score.md](v1-daily-trip-score.md), #471). | Study gap is higher education, which the model does not have. Exclusions by geography: #483 open. Shopping and other purposes not compared yet. | niu-boi (#483) |
| 3 | Each other entity type has declared properties with a real source | Jobs (`jobs.csv`: EOD 2012, permits), schools (MINEDUC), stations and lines (DTPM GTFS, `editable_transit`), boat line (piers, speed, headway, capacity; #359, river route #467), stadium event (attendance, Colo-Colo fan base), shops (OSM activity points), buildings (OSM footprints). | Pharmacy, cinema and park have no destination kind; hospitals and clinics are in the pack but nobody chooses them ([OSM and registry counts](https://discord.com/channels/1573514576400752640/1573809708047798364/1573860611081900115)). Shop hours are missing for most points. | Alberto-72 (service table) |
| 4 | The three demo scenarios use only property changes | MUT ↔ Monumental: jobs edits, event at a point, shops. Mapocho: `line add` with `mode:"boat"`. Metro closure: `line remove` target mode `metro` plus `half_hours` window (#368, #371). No scenario branch in `scenario.rs`. | Legacy MUT-only code remains outside the scenario path: `site.rs` and `stations.rs` `MutRequest` ("site_id must be MUT"). Not called by the scenario route; safe to delete. | open |
| 5 | The fixed score (V4) improves | Line 1 interruption, 18 Nov 2024: **1.0082 → 0.348** after the V1 layer `line cut` + `metro_reach` (#473, [v4-line1-model.md](v4-line1-model.md)). No-change reference 0.883. | Monumental match events still have no model score. Los Leones and Tobalaba move the wrong way (unscored). | V4 harness owner |
| 6 | Full Metro closure compared with October 2019, in the demo | Observed service state: 0 stations open on 19–20 Oct 2019 ([metro-closure-october-2019.md](metro-closure-october-2019.md), #343). Historical card mock (#422). | The comparison is service availability only; no public event-day counts by mode exist. Native transition from replay to card is open. | V3 (calm-agent) |
| 0 | The humans say V1 is finished | — | Pending. | humans |

## Shortest path to close

1. Merge #483, #371 (Peumo), then #416 and the #402 connection.
2. Decide on income: put the CASEN band on each person (small engine change) or record that the demo does not show it.
3. Delete the legacy MUT-only site code, or record why it stays.
4. Post one `#results` message asking the humans to close V1.
