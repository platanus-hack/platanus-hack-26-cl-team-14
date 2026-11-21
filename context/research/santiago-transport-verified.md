# April 2026 transport: bounded record verification

Checked 2026-11-21. This closes the calendar and boarding-row gaps in the transport catalog. It does not establish a complete ingestion pipeline. Role INPUT means model supply; TARGET means a candidate comparison with compatible simulated observations.

## March GTFS: VERIFIED-ROWS · INPUT

[GTFS_20260321_v3.zip](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip) returned HTTP 200, **10,701,870 bytes**. Read actual CSV records in `feed_info`, `calendar`, `calendar_dates`, `frequencies`, `trips`, `stop_times`, and `agency`.

- `feed_version=V161.20260314`; advertised dates 2026-03-21 through 2026-12-31. This advertised end date does not prove that later replacement feeds are equivalent.
- All six calendar rows have these same date bounds. All four agencies specify `America/Santiago`.
- All 28 exception records were inspected; none falls on April 11–17.
- Calendar plus exception processing gives these active service IDs:

| Dates in April 2026 | Active service IDs |
| --- | --- |
| 11, Saturday | S |
| 12, Sunday | D, F |
| 13–16, Monday–Thursday | L, LJ |
| 17, Friday | L, V |

Keep all applicable IDs. A single weekday filter would omit service. Select trips by `service_id`, rather than expanding each calendar row as if it were a duplicate copy of the whole network.

There are **14,474 frequency rows**, all with `exact_times=0`. Example `101-I-L-B03` belongs to route 101, service L, direction 0, toward Cerrillos. Its frequency interval is 06:30–08:00, with a headway of 675 seconds. Its first two stop-time rows are PB1 at 00:00:00 and PB2 at 00:01:49. These are a trip template, not midnight passenger departures.

The opened [GTFS reference](https://gtfs.org/documentation/schedule/reference/#frequenciestxt) defines `exact_times=0` as frequency service, `headway_secs` in seconds, and `exact_times=1` as fixed schedule service. Use stop-time differences with the frequency interval. Do not claim exact departures for this sample. GTFS describes scheduled supply, not actual vehicle positions, speeds, occupancy, or cancellations.

Spatial scope is the Red/Metro/EFE network, not the administrative boundary of Santiago comuna. This check does not establish full stop geometry joins or equivalence between DTPM and GTFS identifiers.

## Boarding workbook: VERIFIED-ROWS · TARGET

[Subidas_Paradero_Estacion_2026.04_v3.xlsx](https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2026.04_v3.xlsx) returned HTTP 200, **29,342,772 bytes**. Read `FICHA_DATOS`, `DICCIONARIO`, actual numeric cells in `SUBIDAS_2026_04`, and the cached `RESUMEN` values. This is stronger evidence than reading shared strings alone.

The source sheet defines the period as the **average of April 13–17 weekdays**, plus **Saturday April 11** and **Sunday April 12** individually. The source combines payment transactions and bus GPS to estimate boarding locations. It explicitly excludes estimated fare evasion. These are validation-derived boardings, not unique riders or all passenger demand.

Actual data headers are `Tipo_dia, Modo, Paradero, Paradero Usuario, Comuna, Media_hora, Subidas_Promedio`. The dictionary calls the fourth field `SIMIT`; the worksheet names it `Paradero Usuario`. `Paradero` is DTPM's internal stop code; the user code is a different identifier. Metro and train samples have `-` for the user code.

`Media_hora` is an Excel day fraction. For example, 0.20833333333333329 is 05:00, not 0.208 hours. The dictionary identifies a half-hour transaction bin. `Subidas_Promedio` averages by day type, mode, stop, and half hour.

The reader streamed 761,222 data rows until it found a positive example for each of the nine mode/day combinations. This is **not a claim about the full worksheet row count**. The following are individual worksheet records:

| Worksheet row | Day type | Mode | Internal stop / user code | Comuna | Half hour | Boardings |
| ---: | --- | --- | --- | --- | --- | ---: |
| 2 | DOMINGO | BUS | E-1-4-SN-50 / PB1559 | INDEPENDENCIA | 05:00 | 1 |
| 174621 | DOMINGO | Metro | ALCANTARA / - | LAS CONDES | 07:00 | 14 |
| 178704 | DOMINGO | Tren | ESTACION ALAMEDA / - | ESTACIÓN CENTRAL | 07:30 | 120 |
| 179017 | LABORAL | BUS | E-1-4-SN-50 / PB1559 | INDEPENDENCIA | 05:00 | 1.2 |
| 533284 | LABORAL | Metro | ALCANTARA / - | LAS CONDES | 05:30 | 5.2 |
| 537837 | LABORAL | Tren | ESTACION ALAMEDA / - | ESTACIÓN CENTRAL | 05:30 | 41.4 |
| 538195 | SABADO | BUS | E-1-4-SN-50 / PB1559 | INDEPENDENCIA | 00:30 | 1 |
| 756844 | SABADO | Metro | ALCANTARA / - | LAS CONDES | 06:00 | 1 |
| 761223 | SABADO | Tren | ESTACION ALAMEDA / - | ESTACIÓN CENTRAL | 06:30 | 121 |

The cached summary reports weekday averages of 2,512,170.4 BUS boardings, 2,469,425.4 Metro boardings, and 82,233.6 Tren boardings. These are publisher summary cells, **not independently recomputed sums**. They count each boarding, so transfers can produce multiple observations per person.

Use these records as a stop/station × half-hour × day-type target after resolving identifier joins. Keep mode-specific validation coverage. Missing rows must not automatically become zero demand. No complete coverage, duplicate, zero-cell, or total-reconciliation audit was performed.

## Trip and stage archives: VERIFIED-ROWS, first-member samples only

Only bytes 0–65535 were requested from each archive. Both requests returned HTTP 206 and **65,536 bytes**. No full archive was downloaded.

| Source | Advertised total bytes | First member |
| --- | ---: | --- |
| [Viajes_2026-04.zip](https://www.dtpm.cl/descargas/modelos_y_matrices/Viajes_2026-04.zip) | 1,658,733,451 | Tabla de Viajes/2026-04-11.viajes.csv.gz |
| [Etapas-2026-04.zip](https://www.dtpm.cl/descargas/modelos_y_matrices/Etapas-2026-04.zip) | 2,008,292,589 | Tabla de Etapas/2026-04-11.etapas.csv.gz |

The ZIP members use storage method 0 and contain gzip streams. Incremental gzip decompression of the first member yielded headers and three complete pipe-delimited records from each file. The rest of the archive, other dates, full row counts, and complete checksums remain unverified.

The opened [trip dictionary](https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura%20Tabla%20de%20Viajes%202026%204.txt) returned HTTP 200, 7,420 bytes. The [stage dictionary](https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura%20Tabla%20de%20Etapas%202026%204.txt) returned HTTP 200, 2,715 bytes.

### Trips: conditional TARGET

The header has 100 named fields and a trailing empty field. Fields include day type, expansion factor, stage count, timestamps, internal stops, comunas, zones, transport types, and improved travel time `tviaje2`. Three sampled expansion factors are **1.5572, 1.7232, 1.8241**, with stage counts **2, 1, 1**. The dictionary defines expansion against the OD matrix and zones as `zonas777`. This zone namespace is not Census blocks or the EOD 866-zone system.

Two mismatches require resolution before aggregation:

- `tipodia` is **1** in these Saturday records, while the dictionary describes LABORAL/SABADO/DOMINGO text. Do not assign a universal meaning to numeric code 1 from this sample.
- The dictionary labels `tviaje2` in minutes. One sampled value, **1258**, equals the **1,258 seconds** between that record's two timestamps. The other two values are 2851 and 1451. Do not ingest these values as minutes. The dictionary's component sum includes waits and other travel components, so it also need not equal the endpoint difference for every row.

`tviaje` is `-` in all three sampled records; the dictionary says it was replaced. Treat it as missing. These data can support a transit OD target only after day codes, units, expansion, and inferred destinations are reconciled. They do not measure all-mode city travel.

### Stages: conditional TARGET

The header has 35 fields. It includes mode, service/stop/period expansion, boarding/alighting times, UTM coordinates, stops, comunas, zones, and an alighting-estimation flag.

Two sampled Metro rows have `tipo_dia=SABADO`, factor **1.0000**, and `tiene_bajada=0`. Their destination and duration are `-`. A BUS row has factor **1.2226**, `tiene_bajada=1`, and duration **985**, equal to a **16-minute 25-second** timestamp difference. Thus this sampled duration is in seconds. Missing alighting data are not zero-length trips.

The dictionary describes alighting as estimated and the service/stop/period expansion separately from a zone/period expansion. Do not apply both factors without the publisher's method. Coordinates are described only as “UTM WSG84”; this dictionary does not specify the zone. No coordinate transformation was established here.

Use stages as candidate boarding/transfer targets after matching the expansion definition. Raw card-related identifiers were excluded from the report and derived evidence. Do not use them to identify or link people outside the transport analysis.

## Method and remaining limits

DTPM robots.txt returned HTTP 200 and did not exclude these download paths. Requests were spaced by at least one second. All complete downloads were below 30 MB. Python standard-library ZIP, CSV, XML, and gzip readers were used; no tests or model runs were performed.

Local evidence is under `data/raw/transport-verified/` in the verifier's worktree: response headers, full GTFS/workbook, two 64 KiB archive ranges, dictionaries, and identifier-free sample JSON. Raw data are not part of the catalog commit. XLSX reading followed workbook relationships and resolved string cells before reading numeric worksheet cells. GTFS service selection used calendar weekday flags plus exception additions/removals.

The matching calendar and numeric boarding observations are verified. Full trip/stage ingestion is not ready: sample schema mismatches, missing destinations, expansion rules, and spatial joins remain open. These limits do not prevent using the boarding workbook as a bounded validation-derived target after stop joins.
