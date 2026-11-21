# Santiago transport and mobility sources

Checked: 2026-11-21. Scope: inputs and calibration data for Santiago's urban simulation. Research only; the pitch is still empty.

This catalog separates scheduled service from recorded travel. It covers useful public sources, not every transport dataset.
All source pages below were opened or attempted in this session. Access failures are explicit.
“Header confirmed” means an HTTP 200 response and file signature were observed; it does not mean the full dataset was inspected.

## Catalog

All 9 entries are **UNVERIFIED FOR INGESTION**: no actual data rows were retrieved.
The access evidence below concerns pages, metadata, or file headers only.
INPUT and TARGET are proposed uses, not proof that the data supports the final score.


| Source | Role and concrete mechanism | Contents and granularity | Coverage and data date | Access observed | Limit for the simulation |
| --- | --- | --- | --- | --- | --- |
| [DTPM static GTFS](https://www.dtpm.cl/index.php/gtfs-vigente) | INPUT: routes, stops, and schedules constrain available public transport paths and departures. | Scheduled routes, stop points, station links, shapes, trips, frequencies, and calendars. | Santiago public transport; feed effective 2026-11-14. Includes Red, Metro, MetroTren, and airport buses per the [official format note](https://www.dtpm.cl/descargas/gtfs/Descripcion%20GTFS%20estatico%20DTPM_20260210.pdf). | Public [ZIP](https://www.dtpm.cl/descargas/gtfs/GTFS_20261114_v2.zip), about 12.8 MB; header confirmed. CSV-style TXT files. | Describes planned service, not passenger demand or actual arrivals. Inspect calendar validity before use. |
| [DTPM trips and stages](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) | INPUT or TARGET: estimate origin–destination demand, or compare simulated trip totals by zone and time. Separate fitting and evaluation samples. | Payment and bus-GPS-derived trips; estimated origin/destination stops and stages. | Red, Metro, and train; April 11–17, 2026; older cuts available. | Public [trip CSV ZIP](https://www.dtpm.cl/descargas/modelos_y_matrices/Viajes_2026-04.zip), about 1.66 GB; header confirmed. Stage ZIP and TXT dictionaries linked. | Excludes fare evasion. Destinations are estimated. Sample days do not represent every week. |
| [DTPM stop and station boardings](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) | TARGET: compare simulated boardings per stop/station and half-hour. Calibration target only; not an independent holdout of the trip table. | Aggregated demand by bus stop or rail station and half-hour. | Santiago system; April 2026 and older cuts. | Public [XLSX](https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2026.04_v3.xlsx), about 29.3 MB; header confirmed. Workbook not inspected. | Derived from the same trip system, not independent validation. Station names and bus stop codes need different joins. |
| [SECTRA Santiago EOD](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/) | INPUT: estimate initial mode shares and trip-purpose distributions with survey weights. Current-year calibration still needed. | Household travel survey; trip purposes, mode shares, and survey zones. | Gran Santiago; 2012 survey. | Official page links [database ZIP](https://drive.google.com/file/d/193675NwL6QNDs6VJFHU0mXiX5-6SdBPJ/view) and [zoning ZIP](https://drive.google.com/file/d/1SM2C5b3jhbmYCbAkmtg2yWRU2dMQkGRl/view). Drive pages opened; archive contents and unauthenticated downloads not verified. | Useful for all-mode behavior. Too old to treat as current demand without recalibration. Preserve survey weights and zone definitions. |
| [OpenStreetMap via Geofabrik](https://download.geofabrik.de/south-america/chile/index.html) | INPUT: street and path geometry defines candidate walking, cycling, and car connections. | Mapped streets and paths at segment level; other mapped objects are included. | Chile, then clip to Santiago; daily snapshot, with 2026-11-20 files listed. | Public PBF, SHP, and GeoPackage downloads listed; file contents not inspected. Page states ODbL 1.0. | Mapping completeness varies. A mapped street does not supply measured traffic, capacity, or travel speed. |
| [SECTRA cycle network](https://geoportal.cl/index.php/geoportal/catalog/36392/ciclovias) | INPUT: existing cycleway segments constrain cycling paths; planned segments can define a scenario. | Cycleway lines, commune, length, and construction/planning status. | Chile, filter Metropolitan Region; live layer named July 2026, last edited 2026-07-31. | [ArcGIS layer](https://services6.arcgis.com/feQ9HId8vmgonvvD/ArcGIS/rest/services/CICLOV_validVisor_WFL1/FeatureServer/0) metadata opened. Query supports JSON, GeoJSON, and PBF; 2,000-record limit. Features not downloaded. | Catalog says May 2024, while live layer is newer. Filter `ETAPA` to existing infrastructure for a baseline. |
| [MINVU cycle counters](https://www.minvu.gob.cl/ciclovias-minvu/) | TARGET candidate: compare simulated bicycle passages with a counter at the same location and time. | Counter points and passage totals; weekday/weekend, daily, weekly, and monthly summaries. | Selected cycleways nationwide; Santiago station coverage and individual date ranges need inspection. Page states monthly map updates. | Public map and [counter portal](https://www.eco-public.com/ParcPublic/?id=5737) linked. MINVU page opened; counter portal timed out. Bulk export not verified. | Measures passages at selected counters, not unique cyclists or citywide trips. Treat as an access lead until a Santiago series is obtained. |
| [DTPM operating statistics](https://www.dtpm.cl/index.php/noticias/estadisticas) | TARGET candidate: compare simulated operated kilometers with published totals. Old speed tables cannot validate current travel times. | Operated kilometers by operating unit; historical service-direction speed tables by time interval. | Santiago bus system. Kilometer links extend into August/September 2026; displayed speed tables are from 2010. | Public page and dated file links inspected; individual tables not downloaded. | A current page contains old sections. Do not label the speed tables as 2026 observations. |
| [CONASET crash portal](https://mapas-conaset.opendata.arcgis.com/) | NONE for the present social-dynamics model. Crash locations do not measure demand; retain only as an optional safety research lead. | Geospatial road-crash source; intended point-level safety evidence. | Chile; select Metropolitan Region. Layer years and geographic completeness not verified. | Portal opened but returned no readable layer metadata. The [2024 commune report](https://www.conaset.cl/wp-content/uploads/2025/09/Distritos-Circunscripciones-2024.pdf) appeared in official search results, but retrieval failed. No usable layer export confirmed. | Access lead, not a ready input. Obtain a dated layer or the commune report before making a safety claim. |

## First inputs to obtain

Recommendation: start with the DTPM boarding workbook, matching-period GTFS, and an OSM street extract.
These give demand totals, scheduled transport, and network geometry. They do not establish a causal response to a land-use change.
Use the larger trip archive only when the selected demo needs origin–destination patterns.

The [April 2026 trip dictionary](https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura%20Tabla%20de%20Viajes%202026%204.txt) was opened.
It includes origin/destination communes, stops, Zone777 codes, expansion factors, and estimated travel-time components.
Its `tviaje` field is deprecated in favor of improved time fields; `tviaje2` is the total of the listed components.
Zone777 is a separate geography: do not equate its identifiers with EOD survey zones or census blocks.

## Gaps and use limits

The [official historical GTFS index](https://www.dtpm.cl/index.php/programas-de-operaciones/gtfs-historicos) was opened.
It links a 2026 archive, but a feed covering April 11–17, 2026 remains unresolved here.
Clever-koala owns feed matching and actual-row sampling in a separate transport verification brief.


- No current, complete private-car origin–destination dataset was verified. EOD 2012 is the nearest verified official survey source.
- No public live bus-position feed or station crowding feed was verified. GTFS alone cannot provide either.
- Current route geometry and older observed trips need compatible stop identifiers and service dates before calibration.
- Public access does not establish reuse terms. OSM declares ODbL; the other opened pages did not establish a common catalog-wide license.
- No full archives were downloaded, no ingestion code was written, and no tests were run.
