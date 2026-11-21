# Score targets

A score target is an observed count that the simulation output can be compared with.
Each entry says: time step, spatial unit, date range, and the simulated quantity that it measures.

### DTPM boardings by stop and station, by half hour (April 2026)
- Link: https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2026.04_v3.xlsx (index: https://www.dtpm.cl/index.php/documentos/matrices-de-viaje)
- Contents: one row = average boardings per stop or station per half hour. Columns: Modo (BUS/Metro/Tren), Paradero, SIMIT, Comuna, Media_hora, Subidas_Promedio, by day type (LABORAL/SABADO/DOMINGO). The expected fields require confirmation against the data sheet; workbook metadata is not a boarding record.
- Granularity: point (stop or station). Time step: 30 minutes.
- Year: week of April 11 to 17, 2026.
- Access: direct download, xlsx, 29.3 MB.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 29,342,772 bytes downloaded to `data/raw/dtpm/subidas_2026_04.xlsx`, workbook sheet names and shared strings (including stop-code string `E-1-4-SN-50`) read with `tools/scrape/dtpm_subidas_peek.py`. Shared strings are not parsed boarding records; no numeric boarding row was read.
- Limits: expanded from bip! validations; fare evasion is not included; comparison needs the same validation coverage by mode, rather than an assumed universal correction factor. Averages per day type, not raw days.
- Role: TARGET
- Use in the simulation: TARGET. Primary score: simulated boardings per stop per half hour vs this table.

### DTPM trip table (viajes) April 2026
- Link: https://www.dtpm.cl/descargas/modelos_y_matrices/Viajes_2026-04.zip
- Contents: one row = one expanded public transport trip. Columns (from structure file): tipodia, factor_expansion (OD matrix, zonas777), n_etapas, tviaje, distancia_eucl, distancia_ruta, tiempo_inicio_viaje, plus origin and destination zones.
- Granularity: trip level, zones (zonas777). Time step: timestamp (continuous).
- Year: April 11 to 17, 2026.
- Access: direct download, zip, 1.66 GB. Column spec: https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura%20Tabla%20de%20Viajes%202026%204.txt (downloaded).
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, Content-Length 1,658,733,451, first 1 KB has a valid ZIP signature (`PK` + internal file "Ta..."), structure txt downloaded with real column list. No source data records established by this check.
- Limits: destinations are estimated (ADATRAP method), not observed. Only public transport. Same evasion bias as boardings.
- Role: TARGET
- Use in the simulation: TARGET. Aggregate to an OD matrix by zone and hour and compare with simulated trips.

### DTPM stage table (etapas) April 2026
- Link: https://www.dtpm.cl/descargas/modelos_y_matrices/Etapas-2026-04.zip
- Contents: one row = one stage (one vehicle boarding) of a trip, with service, stop, and time.
- Granularity: stage level, stop. Time step: timestamp.
- Year: April 11 to 17, 2026.
- Access: direct download, zip, 2.0 GB. Structure: https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura%20Tabla%20de%20Etapas%202026%204.txt
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, Content-Length 2,008,292,589, first 16 bytes have a valid ZIP signature (range request, HTTP 206).
- Limits: same as the trip table. Very large; sample before full ingestion.
- Role: TARGET
- Use in the simulation: transfers per route and loads per service segment.

### DTPM historical matrices (2012 to 2026)
- Link: https://www.dtpm.cl/index.php/documentos/matrices-de-viaje
- Contents: the same three tables for about two weeks per year back to 2012, for example Nov 2025 (`Tablas_viajes_NOV_2025.zip`), April 2025, and older cuts.
- Granularity: same as above.
- Year: 2012 to 2026, one or two weeks per year.
- Access: direct download per file.
- Check: UNVERIFIED for source data; 2026-11-21, HTTP 200, 58,028 bytes of index HTML with the file links listed. No source data records established by this check.
- Limits: method changed in April 2017 (daily estimation); older cuts are weekly. Zone systems can differ between years.
- Role: TARGET
- Use in the simulation: TARGET. Trend check: the same score computed on a second week guards against overfitting to one week.

### DTPM GTFS feed matching April 11–17, 2026
- Link: https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip (history: https://www.dtpm.cl/index.php/programas-de-operaciones/gtfs-historicos)
- Contents: routes, stops, timetables and shapes for Santiago public transport.
- Granularity: stop and timetable level.
- Year: feed starts March 21, 2026; the next feed starts April 18. This covers the April 11–17 observation week.
- Access: direct ZIP download.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206 range request, valid ZIP signature. Transport verifier clever-koala separately reported parsed feed V161 and calendar records; use that verifier's report for row evidence.
- Limits: April 25 and November feeds do not match this observation week. Frequency-based trips need frequencies.txt and stop-time offsets, not clock-time interpretation.
- Role: INPUT
- Use in the simulation: INPUT. Route the April observation week on this network, subject to the calendar and exception records.

### INE monthly Metro ridership (press series)
- Link: https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2026/09/01/personas-transportadas-por-metro-de-santiago-alcanzaron-un-total-de-53-5-millones-en-julio-de-2026
- Contents: total persons transported by Metro de Santiago per month (for example 53.5 million in July 2026), from the INE transport bulletin.
- Granularity: whole network. Time step: month.
- Year: continuous series, at least 2025 to 2026.
- Access: press pages; the bulletin has xlsx series on the INE transport statistics page (HTTP 200, file links behind a JS portal).
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 97,042 bytes, the figure "53,5 millones" is in the page text.
- Limits: one number per month for the whole network. No station detail.
- Role: TARGET
- Use in the simulation: TARGET (weak). Sanity check of total simulated Metro demand and seasonality.

### MOP concession dipticos: urban highway transactions per toll gantry
- Link: https://concesiones.mop.gob.cl/uploads/sites/4/2025/01/Diptico_VS_2024_01.pdf (Vespucio Sur; one PDF per concession per month on concesiones.mop.gob.cl)
- Contents: monthly toll transactions per gantry (pórtico), vehicle category split, year-on-year comparison, from SICE reports.
- Granularity: toll gantry (point on the highway). Time step: month.
- Year: about 2021 to 2025, by concession.
- Access: direct download, PDF. Numbers must be extracted from charts and tables by hand or with a PDF parser.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 583,625 bytes, valid `%PDF-1.7` header. The Sistema Norte-Sur diptico URL returned 200 but Content-Length 2,518, likely an error page: UNVERIFIED for that concession. No source data records established by this check.
- Limits: PDF only, no bulk data. Chart extraction may be too lossy for a score until checked against the printed values. Transactions count passages, not persons or unique trips. See also `traffic-counts.md` for INE toll series.
- Role: TARGET
- Use in the simulation: TARGET. Candidate gantry-count target after extracting actual observations; the checked PDF header alone does not establish counts.

### INE Censo 2024: persons and dwellings per manzana
- Link: https://censo2024.ine.gob.cl/resultados/
- Contents: persons and dwellings per manzana, plus manzana polygons (GeoParquet).
- Granularity: manzana. No time step (one cut).
- Year: 2024 (published December 2025).
- Access: JS portal; direct file URL not exposed in the HTML. Third-party mirrors exist (github.com/pachadotdev/censo2024-cartografia-postgresql, github.com/bastianolea/censo_2024_mapas). Also cataloged in `context/research/data-sources-buildings-activity.md`.
- Check: UNVERIFIED for data rows. Portal page HTTP 200, 168,990 bytes, but no file link in the HTML; no download executed.
- Limits: static. Counts residents, not daytime population.
- Role: INPUT and TARGET
- Use in the simulation: initialize resident totals or compare held-out totals with the model. A count used directly as input is not an independent target.

### MINEDUC enrollment per school (matrícula)
- Link: https://datosabiertos.mineduc.cl/matricula-por-estudiante/
- Contents: one row per enrolled student per year, with school (RBD), grade, and comuna; also per-school summaries. Schools are georeferenced in the MINEDUC directory.
- Granularity: school (point via directory join). Time step: year.
- Year: about 2004 to 2026.
- Access: portal page HTTP 200, but file links are not in the static HTML (JS). No API found.
- Check: UNVERIFIED for data rows. Page HTTP 200, 19,815 bytes; no csv/zip link found in HTML; datos.gob.cl search returned nothing.
- Limits: enrollment is not daily attendance. The attendance dataset (`asistencia-declarada-mensual`) exists on the same portal, also unverified here. The school directory separately returned a `Directorio-EE-Oficial-2026.rar` archive signature (HTTP 206); that does not establish matrícula rows.
- Role: TARGET
- Use in the simulation: TARGET (static). Compare assigned student totals per school with enrollment, not daily trips or attendance.

### EOD Santiago 2012 (origin-destination survey)
- Link: https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/ (old URL sectra.gob.cl/encuestas_movilidad returns 404)
- Contents: trip diary of 18,000 households, all modes, 866 zones. The page links three Google Drive files: Base de Datos (drive id 193675NwL6QNDs6VJFHU0mXiX5-6SdBPJ), Zonificación (`Zonificacion_EOD-2012_Santiago.zip`, id 1SM2C5b3jhbmYCbAkmtg2yWRU2dMQkGRl), Informe (id 13_j6G1vaG1acOuL-W5okvAEJ0aaGKO7e).
- Granularity: household and trip, 866 zones.
- Year: field work 2012 to 2013.
- Access: Google Drive download, zip.
- Check: UNVERIFIED for data rows. On 2026-11-21, the files were listed: Drive view page HTTP 200 with title `Zonificacion_EOD-2012_Santiago.zip`. Data rows not downloaded. No source data records established by this check.
- Limits: 14 years old, before Metro Lines 3 and 6 and before the pandemic. Do not use as a current target.
- Role: INPUT
- Use in the simulation: INPUT only (behavioral priors: trip rates per person, mode choice parameters). NOT a score target.

### UOCT intersection traffic counts
- Link: https://chileatiende.gob.cl/fichas/23628-solicitar-informacion-de-transito
- Contents: continuous and periodic vehicle counts per intersection, operated by UOCT.
- Granularity: intersection (point). Time step: sub-hourly.
- Year: continuous.
- Access: paid request by email (ventas@uoct.cl), 7.36 UF per intersection for continuous counts. No open download.
- Check: UNVERIFIED for traffic records. The ChileAtiende service page describes paid access.
- Limits: cost and delay make it unusable in a hackathon.
- Role: NONE
- Use in the simulation: NONE (paid; see `traffic-counts.md` for open car-flow options).

### MINVU bicycle counters (Eco-Counter public API)
- Link: https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/5737?withNull=true (inventory, 172 sites); daily series per site via the `/data/<idPdc>` endpoint. Found by keen-panda (`verify-transport.md`).
- Contents: one row = date and bicycle passages at a counter site. Inventory has `idPdc`, `nom`, `lat`, `lon`, flow IDs.
- Granularity: point (counter site). Time step: day (sub-day intervals exist via the `interval` parameter).
- Year: continuous; covers the DTPM week April 11 to 17, 2026.
- Access: open JSON API, no key.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200: 7 real date/count pairs for site 300062559 (Alameda con Portugal), Apr 11 to 17 2026 = 2330, 3326, 3022, 3136, 3620, 3277, 3223.
- Limits: passages, not persons (one person can pass many times). Site coverage across Santiago and outage behavior unverified. Request dates are day/month/year; response dates are month/day/year.
- Role: TARGET
- Use in the simulation: Daily simulated bicycle passages at counter sites vs this series. Candidate third score term once cycling exists in the model.

### Negative results (searched, nothing open found)
- Pedestrian counts: this search did not obtain an open dataset.
- Metro per-station monthly series as a separate open file: not found; the DTPM boardings xlsx is the per-station source.
- The tested datos.gob.cl searches did not locate DTPM, UOCT, or Metro mobility datasets (2026-11-21).

## Proposed transport score

This is a transport-only proposal, not an approved score for the whole urban model. It cannot assess housing or work outcomes.
First read actual boarding records and trip records. Fix the zone mapping, units, expansion weights, mode coverage and cell set before comparing layers.
Use April 11–17, 2026, day type LABORAL, on GTFS_20260321_v3. Compare average weekday quantities on both sides.

For nonnegative simulated count s and reference count o:

```
e(s,o) = 0                         if s = o = 0
         abs(s-o) / (s+o)          otherwise
S_stop = 1 - mean(e)               over the fixed stop × half-hour cells
S_od   = 1 - mean(e)               over the fixed zona777 pair × hour cells
SCORE  = 0.6*S_stop + 0.4*S_od      in [0,1], higher is better
```

Choose eligible cells from the reference coverage before running the baseline; keep them fixed for every model version.
Report errors by mode and count band alongside this score. Sparse cells can dominate an unweighted mean. Do not apply a fixed 25% bus-evasion correction without evidence for the selected period and coverage.
OD destinations are inferred by DTPM, so S_od measures agreement with an estimate, not independently observed destinations.
Do not score an input against itself: hold out dates or quantities that the model did not directly consume.

## Critique

1. Workbook metadata, ZIP signatures and schema files do not establish actual DTPM data rows. Use the transport verification report before ingestion.
2. Gantry counts are not unique car trips and do not cover ordinary streets. The searched sources do not establish an open sub-hourly street-count target.
3. Census, school and EOD sample status must be reconciled with the separate verification reports. Enrollment measures students, not daily school trips.
