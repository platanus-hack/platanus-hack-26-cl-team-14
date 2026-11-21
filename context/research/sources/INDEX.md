# Swarm source catalog: index

This folder supplements [the master catalog](../santiago-data-catalog.md). It contains source leads and sample reports, not a completed ingestion pipeline.
Entry rules: [README](README.md). Scripts live in the repository's `tools/scrape/`; raw files stay in ignored `data/raw/`.

`VERIFIED-ROWS` means records were read. `VERIFIED-BYTES` means file bytes were received, without established records.
`UNVERIFIED` includes portal-only, blocked and unavailable data. Read each entry's evidence and limits; no combined verification count is used.

| File | First useful source or question | Proposed role and evidence limit |
| --- | --- | --- |
| [Score targets](score-targets.md) | DTPM April 11–17 boardings and estimated OD | Transport TARGET candidates. Workbook metadata and archive bytes alone do not establish rows. |
| [Traffic counts](traffic-counts.md) | INE monthly gantry passes; toll schedules | Pass-count TARGET and toll-cost INPUT. Passes are not unique car trips. COPSA conflicting totals remain NONE. |
| [Tourism and visitors](tourism-visitors.md) | Sernatur domestic trips | Candidate visitor INPUT; inspect the recorded sample scope before ingestion. |
| [Live feeds](realtime-apis.md) | Bike Itaú GBFS station stock | Available-bike INPUT. Stock differences do not measure user trips. |
| [Environment](environment.md) | Declared episode dates and vehicle restrictions | Scenario INPUT. Eligibility and exemptions matter; pollutant measurements do not by themselves establish restrictions. |
| [Events and attendance](events-attendance.md) | Event dates and attendance evidence | Calendar INPUT; estimates must remain distinct from observed attendance. |
| [Public registers](demography-registers.md) | INE population projections | Modeled INPUT, not an independent observed population target. |
| [People and work check](verify-people-work.md) | Census 2024 manzanas; ENE and ESI rows with home and work comuna | INPUT. Rows read for all 10 sources. Survey rows are not weighted. |
| [Transport check](verify-transport.md) | EOD 2012 mirror (113,591 trips); bicycle counter at Alameda con Portugal | INPUT (EOD, old) and one site-day TARGET (bicycles). The mirror is not compared with the official ZIP. |
| [Housing and land check](verify-housing-land.md) | Green areas, campamentos, INE permits, SII F2890 sample | INPUT. Small samples only. Zoning service is broken; TOCTOC gives 403. |
| [Sale and asking prices](housing-listings.md) | SII F2890: 9,219,582 property transfers with price, 1998 to October 2026 | Price INPUT, and TARGET at SII rol and comuna level only. No coordinates. A deed with many roles repeats the full amount. Rent prices are not open. |
| [Prior simulators](prior-art-simulators.md) | MATSim Santiago v2b: 665,201 agents, 36 count stations, 2012 to 2013 data | INPUT and method reference. No synthetic population of Santiago exists. Nobody ran the simulator here. |
| [Construction permits](construction-pipeline.md) | INE permits with coordinates, 2010 to 2018 (UC mirror): 25,808 RM points | INPUT for "dwellings in a tower of N floors". Ends in 2018. Zoning layers have blank and free-text height. |
| [Time use and behaviour](time-use-behaviour.md) | ENUT 2023 (48,020 rows); ENE 2026 work hours | INPUT for day schedules. ENUT gives durations, not the hour order. `region_ord=7` is RM. |
| [Global grids](global-grids.md) | Meta HRSL 30 m (Chile is on HDX); WorldPop 100 m | Fallback INPUT. Census 2024 manzanas are real counts and come first. No pixel of Santiago was read by the owner. |
| [Urban indicators](urban-indicators.md) | SIEDU and SIET tables | Comuna-level INPUT. Derived indexes with mixed years. Below comuna: not verified. |
| [Crime and safety](crime-safety.md) | CEAD cases by comuna, year, and crime family | Comuna-level INPUT. Nothing public below the comuna. |
| [Phone and app mobility](telco-app-mobility.md) | Read the file | Not reviewed yet by a second agent. |
| [Job locations](jobs-location.md) | Read the file | Not reviewed yet by a second agent. |
| [Portal sweep](portals-sweep.md) | Read the file | Not reviewed yet by a second agent. |

## Integration notes

- Use `GTFS_20260321_v3.zip` for the April 11–17, 2026 DTPM week, subject to its calendar records. The next feed starts April 18. April 25 and November feeds are not period matches.
- The proposed score covers transport only. Before adopting it, read actual records, fix expansion weights and zone mappings, and keep validation coverage consistent by mode.
- The reviewed traffic sources include aggregate gantry passes. This pass did not obtain an open sub-hourly count dataset for ordinary streets or pedestrians; that is a search result, not proof of absence. The transport verifier reports daily bicycle counts at Alameda con Portugal for the April observation week.
- The tested live endpoints did not provide measured bus or Metro movement records. Blocks, missing keys and unavailable endpoints are access limits, not proof that data does not exist.
- Additional topic and verification files may arrive in later batches. The master catalog records integrated reports and remaining gaps.
- Sale prices are open (SII F2890). The older claim "no open property-level sale history" is wrong.
- Speed test: 7 million destination draws over 866 zones took 1.75 s in numpy on a laptop, with random costs and no activity chains. "Seconds" is probable, not proved.
- SII comuna codes are not INE comuna codes. See `../santiago-source-joins.md`.
- Held back: `utilities-proxies.md`. A critic found dead links that the file called verified.
