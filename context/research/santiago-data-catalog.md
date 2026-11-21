# Santiago urban data catalog

Research date: 2026-11-21. This catalog supports source selection for the urban social dynamics simulation.
These briefs cover transport, people, work, housing, and land. Companion research covers buildings and activity places.
This is not an exhaustive inventory or an imported dataset.

## Scope and evidence

The pitch is still empty. This work supports problem research; it does not define demo features.
The accepted scope covers population, transport, housing, work, and leisure in Santiago.
Physical hazard simulation is outside that scope.

Each source entry records its contents, geography, year, access route, and known limits.
An accessible page does not prove that its downloads work. The briefs state what the researcher inspected.
Dates describe the data where known; an access date does not make old data current.
INPUT means a candidate model input. TARGET means a candidate comparison measure; it does not define the final score.
UNVERIFIED FOR INGESTION means actual data rows were not inspected, even when metadata or file headers responded.

## Catalog sections

| Topic | Source brief |
| --- | --- |
| Transport schedules, travel demand, roads, cycling, and crashes | [Transport](santiago-transport.md) |
| Population, households, income, work, businesses, education, and health | [People and work](santiago-people-work.md) |
| Housing, property values, land use, public space, and settlements | [Housing and land](santiago-housing-land.md) |
| Building geometry, shops, culture, sport, and other activity places | [Buildings and activity](data-sources-buildings-activity.md) |
| Weather, air quality, and optional environmental context | [Environment](santiago-environment-hazards.md) |

Actual-row sampling and matching-period transport verification continue in the #agent-chat goal thread (`1573562197387251883`).

## Sampled records

| Source | Observed evidence | Remaining limit |
| --- | --- | --- |
| SINCA, Cerrillos II | 7 daily PM2.5 records, 2026-11-12 to 2026-11-18; exact response URL and fields in the environment brief | Preliminary values; other stations and full archive not checked |
| DMC, Quinta Normal | 31 daily temperature records, January 2025; exact report URL and fields in the environment brief | Report does not expose per-row validation flags |

These samples establish access to records. They do not establish a measured effect on social activity or a complete ingestion pipeline.

## Before combining sources

- Choose the study boundary. Santiago commune, Greater Santiago, and the Metropolitan Region have different coverage.
- Keep each source's year and spatial identifiers. Record coordinate systems before joining spatial layers.
- Check geographic representativeness before using survey estimates at commune or block level.
- Separate observed trips from scheduled service, assessed values from sale prices, and business addresses from workplace counts.
- Confirm access and reuse terms before importing data. A public dashboard may not provide a public bulk dataset.

## Remaining work

Select and download the datasets needed by the eventual pitch. Check their actual fields and spatial coverage before model calibration.
The source briefs identify access gaps and the nearest usable alternatives.
