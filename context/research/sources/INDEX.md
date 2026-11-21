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

## Integration notes

- Use `GTFS_20260321_v3.zip` for the April 11–17, 2026 DTPM week, subject to its calendar records. The next feed starts April 18. April 25 and November feeds are not period matches.
- The proposed score covers transport only. Before adopting it, read actual records, fix expansion weights and zone mappings, and keep validation coverage consistent by mode.
- The reviewed traffic sources include aggregate gantry passes. This pass did not obtain an open sub-hourly count dataset for ordinary streets or pedestrians; that is a search result, not proof of absence. The transport verifier reports daily bicycle counts at Alameda con Portugal for the April observation week.
- The tested live endpoints did not provide measured bus or Metro movement records. Blocks, missing keys and unavailable endpoints are access limits, not proof that data does not exist.
- Additional topic and verification files may arrive in later batches. The master catalog records integrated reports and remaining gaps.
