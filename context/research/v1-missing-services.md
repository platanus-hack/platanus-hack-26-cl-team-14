# Basic services that the model does not use yet

Request from Bruno: find the basic service buildings (needs and interests) that the simulation does not model. Research only. No engine change.

## What the model has now

- `data/mvp/activity_points.csv` has 16,697 OSM objects in 6 kinds: shop 14,946, public_office 540, supermarket 508, clinic 453, mall 151, hospital 99 ([activity_points.md](../../data/mvp/activity_points.md)).
- `engine/src` reads `schools.csv` (3,220 schools) and `event_venues.json`. On `main`, no code reads `activity_points.csv`. PR #416 (open) adds shopping and selects only shop, supermarket and mall.
- No code on `main` selects hospital, clinic or public_office as a destination.
- `higher_ed.csv` (137 campuses) exists, but `engine/src` does not read it. `blocks.csv` students exclude higher education. Since #475, main counts 301,724 higher-education residents at home (role in `Population.residents`), but they make no study trip.
- The viewer draws 4,666 park polygons (`web/data/green_areas.geojson.gz`). The engine does not send persons to parks.
- Stadium attendance is a fixed input. PR #402 (optional-visit rule) is a draft.

"Engine use" in the table has three values: **used**, **packed, not used**, **absent**.

## Trip purposes (EOD 2012, weekday, expanded, from `data/mvp/eod_trip_profiles.csv`)

| Purpose (`tools/pack/eod_trip_rates.py`) | Share of all trips | Share of trips that start 07:00–10:00 | Share of trips that start 16:00–20:00 |
| --- | ---: | ---: | ---: |
| shopping (code 11) | 8.9% | 2.0% | 10.5% |
| other (8 drop off or pick up someone, 10 drop off or pick up something, 14 other) | 7.0% | 8.1% | 5.5% |
| leisure (6 visit someone, 9 eat or drink, 13 recreation) | 5.4% | 1.3% | 6.3% |
| errands / trámites (12) | 3.7% | 3.6% | 1.4% |
| health (5) | 1.9% | 3.3% | 0.7% |

Shares exclude trips with unknown start time. Work, study and home are the rest. "Leisure" is not only recreation: it includes visits and meals out.

## Table

Sources and counts: OSM is the Geofabrik Chile snapshot of 2026-11-20, clipped to the model rectangle (−70.85 to −70.45, −33.65 to −33.30) and `communes.geojson`, the same method as `activity_points.md`. Hospital, clinic, pharmacy, cinema and park OSM counts are from **Diuca** (objects / named / with OSM hours). My osmium count on the same snapshot agrees (pharmacy 1,226, cinema 42, park 4,689). Other OSM counts are mine (osmium, read 2026-11-21). Official registry counts are from **Tenca/Diuca**, except DEIS by type in the rectangle (mine, DEIS file dated 2026-11-17, read 2026-11-21). Registries use different dates, categories and boundaries than OSM.

| Need / interest | Kinds we have (count, engine use) | Missing kind | OSM count (rectangle) | Official registry | Priority | Reason |
| --- | --- | --- | ---: | --- | --- | --- |
| Health, primary care | clinic 453 (packed, not used) | CESFAM / CECOSF / SAPU as a kind | in `clinic` mix; Diuca clinic 472 / 464 / 40 | DEIS in rectangle: CESFAM 148, CECOSF 56, SAPU 78, SAR 23, private centre 226 | **High** | Health is 3.3% of 07:00–10:00 trips (2x its daily share). The Metro closure window is the morning. |
| Health, hospital | hospital 99 (packed, not used) | none, needs engine use | Diuca 100 / 100 / 8 | DEIS RM: 45 active hospitals (Tenca/Diuca); 38 in rectangle (mine) | **High** | Same morning window. Kind exists; only the destination choice is missing. |
| Health, pharmacy | none | pharmacy | Diuca 1,226 / 1,184 / 85 | Farmanet: 1,017 Saturday pharmacy IDs, 44 comunas (duty feed mixes dates) | Medium | High count, short trips. EOD puts pharmacy trips in shopping or health; no own purpose code. |
| Procedures (trámites) | public_office 540 (packed, not used) | bank, post office, registro civil | bank 658, post_office 186 | Registro Civil office list: unverified | Medium | 3.6% of morning trips. Kind exists for public offices; banks are absent. |
| Recreation, park | none in engine (viewer draws 4,666 polygons) | park as a destination (needs gate point + walk link) | Diuca 4,689 / 1,827 / 149 (169 with access tags) | MINVU RM: 754 park polygons, 1,396 entrance points with hours | **High** | Parquemet is drawn already (OSM way w168227059). One person walking to a gate is a cheap, visible witness. |
| Recreation, cinema / theatre | none | cinema, theatre | cinema Diuca 42 / 42 / 1; theatre 112 | MINCAP: 13 RM cinema rows (2021 seed list) | Low | Evening and weekend trips. No demo scenario needs it. |
| Recreation, sport | stadiums as fixed events | sports centre, gym | sports_centre 449, fitness_centre 216 | IND facility list: unverified | Low | Small volume. Not in a scenario. |
| Food out | none | restaurant, fast food, café | restaurant 3,060, fast_food 1,481, cafe 980 | none checked | Medium | Part of "leisure" (code 9); 16:00–20:00 share 6.3%. Near the MUT 17:00 match end. |
| Food shopping | supermarket 508, mall 151, shop 14,946 (used after #416) | feria libre | marketplace 99 (under-counts ferias) | Feria count: unverified (ASOF / municipal lists) | Low | #416 covers shopping volume. Ferias run on fixed weekdays, not in the demo windows. |
| Education, higher | `higher_ed.csv` 137 campuses (packed, not used) | none, needs engine use | university 215, college 74 | MINEDUC / SIES campus data already in `higher_ed.csv` | Medium | Study is 31.9% of morning trips. The 301,724 higher-education residents (#475) exist at home but make no trip to a campus. Only a subset travels (enrollment includes remote study). Large morning volume. |
| Care, nursery | none | kindergarten (jardín infantil) | kindergarten 1,206 | JUNJI / Integra directory: unverified | Medium | Code 8 (drop off someone) is in "other", 8.1% of morning trips. Link to nursery trips is an inference. |
| Worship | none | place of worship | 1,030 | none checked | Low | Weekend trips. No scenario needs it. |

Counts not used: `leisure=swimming_pool` 10,498 (mostly private pools); `healthcare=centre` 3 (tag not used in practice).

## Next step

1. Witness for the demo: one synthetic person walks to the Parquemet gate at Pedro de Valdivia Norte, or skips the visit when the time limit does not allow it (Diuca). Parquemet is drawn from OSM way w168227059. Needs the MINVU entrance point and PR #402.
2. Send morning health trips (3.3% of 07:00–10:00) to the packed hospital and clinic points, with DEIS CESFAM/SAPU as the primary care kind. Same choice rule as #416. This makes the Metro closure scene include patients.
3. Give an eligible subset of the 301,724 higher-education residents (#475) a morning study trip to the campuses in `higher_ed.csv`. Enrollment includes remote and evening study, so not every enrolled person travels to a campus each morning. Take the travelling share from EOD 2012 study trips to higher education, not from enrollment.

## Unverified

- DEIS file URL: the local file `establecimientos_20261117.csv` came from another agent. The source page is the DEIS open data listing of health establishments (deis.minsal.cl, unverified URL).
- Registro Civil, IND, JUNJI / Integra and feria counts: not read.
- Farmanet, MINVU and MINCAP counts: reported by Tenca/Diuca, not re-read by me.
- OSM objects are map objects, not unique establishments. OSM coverage is lower in poorer comunas ([data-sources-buildings-activity.md](data-sources-buildings-activity.md)).
