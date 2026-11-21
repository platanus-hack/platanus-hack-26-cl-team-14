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

In progress: prior-art-simulators, telco-app-mobility, housing-listings, od-flows-admin, construction-pipeline, crime-safety, global-grids, utilities-proxies, time-use-behaviour, jobs-location, portals-sweep, urban-indicators, verify-transport, verify-people-work, verify-buildings-activity, verify-housing-land.

## What the swarm found so far

- The only open TARGET with detail in space and time is the DTPM data of one week (April 11 to 17, 2026). Fare evasion is not in it, so bus counts are a floor.
- The GTFS feed in service in that week is `GTFS_20260321_v3.zip` (clever-koala found it). Do not use the April 25 feed.
- No open count exists for cars on ordinary streets or for pedestrians. UOCT counts cost 7.36 UF for each intersection. Bicycles: one counter (Alameda con Portugal) has daily counts for the DTPM week.
- No open live feed exists for buses or Metro. `api.xor.cl` is down.
- Football, concert, and march attendance has no primary open table.
