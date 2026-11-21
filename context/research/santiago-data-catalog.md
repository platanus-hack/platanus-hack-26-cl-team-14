# Santiago urban data catalog

Research date: 2026-11-21. This catalog supports source selection for the urban social dynamics simulation.
These briefs cover transport, people, work, housing, and land. Companion research covers buildings and activity places.
This is not an exhaustive inventory or an imported dataset.

## Download order

This order is a research recommendation. The final pitch still determines which mechanisms to build.

1. **Census 2024 block totals and cartography:** establish population and household totals within one explicit study boundary.
2. **GTFS valid for the observation week:** establish the public transport network before comparing passenger demand.
3. **DTPM April 11–17, 2026 boardings:** candidate stop/station and half-hour comparison target. Keep fare-evasion and estimation limits explicit.
4. **Schools, health facilities, SII branches, and activity places:** add destinations. Enrolment can weight schools; branch worker counts and venue visits remain separate gaps.
5. **Housing, land, and environmental layers:** add only the inputs required by the selected social mechanism.

A public-transport score does not validate cars, walking, cycling, housing, or the whole city's social dynamics.
No complete current street-level count dataset for all those travel modes was verified in the initial catalog.
Access failures are not proof that a dataset does not exist.

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
| Geographic identifiers, coordinate systems, and time compatibility | [Source joins](santiago-source-joins.md) |
| Additional source topics, comparison targets, and independent sample reports | [Detailed source index](sources/INDEX.md) |

Independent transport evidence is in [April transport verification](santiago-transport-verified.md). Topic reports in the detailed index distinguish parsed rows, file bytes, and unverified access.

## Sampled records

| Source | Observed evidence | Remaining limit |
| --- | --- | --- |
| Census 2024 | 52,646 RM aggregate records; 52,593 match the sampled polygon layers, as recorded in the people/work verification report | 53 commune containers hold 108,997 people without unit polygons; allocation remains a model choice |
| MINEDUC 2026 | 16,831 school records parsed; 3,227 operating RM schools | Coordinates are approximate; enrolment is not daily attendance |
| March GTFS and April DTPM boardings | Active service IDs checked for April 11–17; numeric boarding rows read for all nine mode/day combinations | Frequency service is not exact departures; stop identifiers still need a full join |
| April DTPM trips and stages | Three complete records read from each archive through 64 KiB range requests | Day-code and duration-unit mismatches; full archives and expansion rules remain unresolved |
| Housing and land | Independent samples of public property transfers, 2025 permits, green areas, and settlements | Transfers are not household moves; price, use, and geography joins remain incomplete |
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
