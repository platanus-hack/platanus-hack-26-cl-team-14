# Swarm source catalog: index

This folder adds to the master catalog in `../santiago-data-catalog.md`. It covers the topics that the master catalog does not cover, and it checks the master catalog sources with real downloads.
Entry rules: `README.md`. Scripts: `tools/scrape/`. Raw files: `data/raw/` (not in git).

Counts come from the first version of each file. Read the `Check` line of each entry: `VERIFIED-ROWS`, `VERIFIED-BYTES`, or `UNVERIFIED`. "Critic" is the agent that got 3 links again.

| File | Best source | Role | Verified / all | Critic |
| --- | --- | --- | --- | --- |
| [score-targets](score-targets.md) | DTPM boardings by stop and half hour, April 11 to 17, 2026 (xlsx, 29 MB, workbook metadata read, rows not read) | TARGET | 8 / 12 | gentle-fox |
| [traffic-counts](traffic-counts.md) | Urban toll gantry passes (INE monthly, COPSA annual, Costanera Norte table) | TARGET (weak) | 15 / 15 | sunny-condor |
| [tourism-visitors](tourism-visitors.md) | Sernatur domestic trips, comuna to comuna, monthly | INPUT | 10 / 10 | open |
| [realtime-apis](realtime-apis.md) | Bike Itaú GBFS (open, no key). No open live feed for bus, Metro, or traffic | INPUT | 7 / 17 | sunny-koala |
| [environment](environment.md) | Car restriction calendar and alert days (PDF, must be coded by hand) | INPUT | 8 / 13 | open |
| [events-attendance](events-attendance.md) | Holiday JSON. All attendance numbers come from the press | INPUT (weak) | 1 / 11 | bold-panda |
| [demography-registers](demography-registers.md) | INE population projections by comuna, 2002 to 2035 | INPUT | 4 / 15 | clever-llama (emersoftware) |
| [prior-art-simulators](prior-art-simulators.md) | MATSim Santiago v2b: 665,201 agents with day plans, 36 count stations, all 2012 to 2013 data | INPUT | read the `Check` lines | open |
| [verify-people-work](verify-people-work.md) | Census 2024 manzanas (52,593 of 52,646 RM rows join to polygons); ENE and ESI rows have home and work comuna | INPUT | 10 / 10 rows | open |
| [verify-transport](verify-transport.md) | EOD 2012 mirror: 18,264 households, 113,591 trips; one bicycle counter with daily counts for the DTPM week | INPUT and TARGET | 9 rows | open |
| [verify-housing-land](verify-housing-land.md) | Green areas, campamentos, INE permits (rows). Zoning service is broken, TOCTOC gives 403 | INPUT | 5 / 9 rows | open |
| [housing-listings](housing-listings.md) | SII F2890 property transfers: 9,219,582 rows, 1998 to October 2026, price in pesos and UF, rol level | INPUT and TARGET | 4 rows / 12 | open |
| [construction-pipeline](construction-pipeline.md) | INE permits with coordinates 2010 to 2018 (UC mirror): 25,808 RM points; 361 towers of 15 floors or more, 298 dwellings on average | INPUT | 4 rows, 5 bytes / 13 | lucky-otter |
| [time-use-behaviour](time-use-behaviour.md) | ENUT 2023 (48,020 rows, 1,089 columns); ENE 2026 work hours | INPUT | 6 rows / 9 | swift-falcon (own review) |
| [global-grids](global-grids.md) | WorldPop R2025A 100 m; Kontur H3 400 m. One claim is in dispute: "Chile is not in Meta HRSL" | INPUT | 10 / 15 | calm-otter |
| [urban-indicators](urban-indicators.md) | SIEDU and SIET tables (comuna level). Below comuna: MBHT, not verified | INPUT | read the `Check` lines | swift-falcon |

Held back, not in this pull request: `utilities-proxies.md` (a critic found 3 dead links that the file calls verified) and `crime-safety.md` (in progress). In progress: telco-app-mobility, od-flows-admin, jobs-location, portals-sweep, verify-buildings-activity.

## What the swarm found so far

- The only open TARGET with detail in space and time is the DTPM data of one week (April 11 to 17, 2026). Fare evasion is not in it, so bus counts are a floor.
- The GTFS feed in service in that week is `GTFS_20260321_v3.zip` (clever-koala found it). Do not use the April 25 feed.
- No open count exists for cars on ordinary streets or for pedestrians. UOCT counts cost 7.36 UF for each intersection. Bicycles: one counter (Alameda con Portugal) has daily counts for the DTPM week; see `verify-transport.md`.
- No open live feed exists for buses or Metro. `api.xor.cl` is down.
- Football, concert, and march attendance has no primary open table.
- Sale prices are open: SII F2890 has each property transfer with the price, at rol level, without coordinates. Rent prices are not open.
- No synthetic population of Santiago exists. MATSim Santiago makes clones of 42,459 EOD respondents.
- Speed: 7 million destination draws over 866 zones took 1.75 s in numpy on a laptop, with random costs. A real step costs more; "seconds" is probable, not proved.
- Census 2024 by manzana is public and joins to the INE polygons.
- SII comuna codes are not INE comuna codes. Use a crosswalk (see `../santiago-source-joins.md`).
