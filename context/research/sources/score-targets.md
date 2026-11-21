# Score targets

A score target is an observed count that the simulation output can be compared with.
Each entry says: time step, spatial unit, date range, and the simulated quantity that it measures.

### DTPM boardings by stop and station, by half hour (April 2026)
- Link: https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2026.04_v3.xlsx (index: https://www.dtpm.cl/index.php/documentos/matrices-de-viaje)
- Contents: one row = average boardings per stop or station per half hour. Columns: Modo (BUS/Metro/Tren), Paradero, SIMIT, Comuna, Media_hora, Subidas_Promedio, by day type (LABORAL/SABADO/DOMINGO). Sheets: FICHA_DATOS, DICCIONARIO, SUBIDAS_2026_04, RESUMEN.
- Granularity: point (stop or station). Time step: 30 minutes.
- Year: week of April 11 to 17, 2026.
- Access: direct download, xlsx, 29.3 MB.
- Check: VERIFIED 2026-11-21, HTTP 200, 29,342,772 bytes downloaded to `data/raw/dtpm/subidas_2026_04.xlsx`, sheet names and columns read with `tools/scrape/dtpm_subidas_peek.py`.
- Limits: expanded from bip! validations; fare evasion is not included (DTPM estimates it near 20 to 30% on buses, so bus counts are a floor, Metro counts are near complete). Averages per day type, not raw days.
- Use in the simulation: TARGET. Primary score: simulated boardings per stop per half hour vs this table.

### DTPM trip table (viajes) April 2026
- Link: https://www.dtpm.cl/descargas/modelos_y_matrices/Viajes_2026-04.zip
- Contents: one row = one expanded public transport trip. Columns (from structure file): tipodia, factor_expansion (OD matrix, zonas777), n_etapas, tviaje, distancia_eucl, distancia_ruta, tiempo_inicio_viaje, plus origin and destination zones.
- Granularity: trip level, zones (zonas777). Time step: timestamp (continuous).
- Year: April 11 to 17, 2026.
- Access: direct download, zip, 1.66 GB. Column spec: https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura%20Tabla%20de%20Viajes%202026%204.txt (downloaded).
- Check: VERIFIED 2026-11-21, HTTP 200, Content-Length 1,658,733,451, first 1 KB has a valid ZIP signature (`PK` + internal file "Ta..."), structure txt downloaded with real column list.
- Limits: destinations are estimated (ADATRAP method), not observed. Only public transport. Same evasion bias as boardings.
- Use in the simulation: TARGET. Aggregate to an OD matrix by zone and hour and compare with simulated trips.

### DTPM stage table (etapas) April 2026
- Link: https://www.dtpm.cl/descargas/modelos_y_matrices/Etapas-2026-04.zip
- Contents: one row = one stage (one vehicle boarding) of a trip, with service, stop, and time.
- Granularity: stage level, stop. Time step: timestamp.
- Year: April 11 to 17, 2026.
- Access: direct download, zip, 2.0 GB. Structure: https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura%20Tabla%20de%20Etapas%202026%204.txt
- Check: VERIFIED 2026-11-21, HTTP 200, Content-Length 2,008,292,589, content-type application/zip. Full file not downloaded.
- Limits: same as the trip table. Very large; sample before full ingestion.
- Use in the simulation: TARGET. Transfers per route and loads per service segment.

### DTPM historical matrices (2012 to 2026)
- Link: https://www.dtpm.cl/index.php/documentos/matrices-de-viaje
- Contents: the same three tables for about two weeks per year back to 2012, for example Nov 2025 (`Tablas_viajes_NOV_2025.zip`), April 2025, and older cuts.
- Granularity: same as above.
- Year: 2012 to 2026, one or two weeks per year.
- Access: direct download per file.
- Check: VERIFIED 2026-11-21, HTTP 200, 58,028 bytes of index HTML with the file links listed.
- Limits: method changed in April 2017 (daily estimation); older cuts are weekly. Zone systems can differ between years.
- Use in the simulation: TARGET. Trend check: the same score computed on a second week guards against overfitting to one week.

### DTPM GTFS feed of April 25, 2026
- Link: https://www.dtpm.cl/descargas/gtfs/GTFS_20260425_final.zip
- Contents: routes, stops, timetables, shapes of the Santiago public transport network.
- Granularity: stop and timetable level.
- Year: valid April 25 to December 2026.
- Access: direct download, zip, 10.7 MB.
- Check: VERIFIED 2026-11-21, HTTP 200, Content-Length 10,709,254.
- Limits: starts 8 days AFTER the April 11 to 17 trip week. It is the nearest public feed; no feed dated inside the trip week was found on the index. Network changes in those 8 days are possible but small.
- Use in the simulation: INPUT. The network that the simulated agents travel on. Needed so the score compares the same network.

### INE monthly Metro ridership (press series)
- Link: https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2026/09/01/personas-transportadas-por-metro-de-santiago-alcanzaron-un-total-de-53-5-millones-en-julio-de-2026
- Contents: total persons transported by Metro de Santiago per month (for example 53.5 million in July 2026), from the INE transport bulletin.
- Granularity: whole network. Time step: month.
- Year: continuous series, at least 2025 to 2026.
- Access: press pages; the bulletin has xlsx series on the INE transport statistics page (HTTP 200, file links behind a JS portal).
- Check: VERIFIED 2026-11-21, HTTP 200, 97,042 bytes, the figure "53,5 millones" is in the page text.
- Limits: one number per month for the whole network. No station detail.
- Use in the simulation: TARGET (weak). Sanity check of total simulated Metro demand and seasonality.

### MOP concession dipticos: urban highway transactions per toll gantry
- Link: https://concesiones.mop.gob.cl/uploads/sites/4/2025/01/Diptico_VS_2024_01.pdf (Vespucio Sur; one PDF per concession per month on concesiones.mop.gob.cl)
- Contents: monthly toll transactions per gantry (pórtico), vehicle category split, year-on-year comparison, from SICE reports.
- Granularity: toll gantry (point on the highway). Time step: month.
- Year: about 2021 to 2025, by concession.
- Access: direct download, PDF. Numbers must be extracted from charts and tables by hand or with a PDF parser.
- Check: VERIFIED 2026-11-21, HTTP 200, 583,625 bytes, valid `%PDF-1.7` header. The Sistema Norte-Sur diptico URL returned 200 but Content-Length 2,518, likely an error page: UNVERIFIED for that concession.
- Limits: PDF only, no bulk data. Transactions count vehicles with TAG, not persons.
- Use in the simulation: TARGET. The only verified observed count for private car flow, per gantry per month.

### INE Censo 2024: persons and dwellings per manzana
- Link: https://censo2024.ine.gob.cl/resultados/
- Contents: persons and dwellings per manzana, plus manzana polygons (GeoParquet).
- Granularity: manzana. No time step (one cut).
- Year: 2024 (published December 2025).
- Access: JS portal; direct file URL not exposed in the HTML. Third-party mirrors exist (github.com/pachadotdev/censo2024-cartografia-postgresql, github.com/bastianolea/censo_2024_mapas). Also cataloged in `context/research/data-sources-buildings-activity.md`.
- Check: UNVERIFIED for data rows. Portal page HTTP 200, 168,990 bytes, but no file link in the HTML; no download executed.
- Limits: static. Counts residents, not daytime population.
- Use in the simulation: TARGET. Where simulated agents sleep must match this distribution. Also the base population INPUT.

### MINEDUC enrollment per school (matrícula)
- Link: https://datosabiertos.mineduc.cl/matricula-por-estudiante/
- Contents: one row per enrolled student per year, with school (RBD), grade, and comuna; also per-school summaries. Schools are georeferenced in the MINEDUC directory.
- Granularity: school (point via directory join). Time step: year.
- Year: about 2004 to 2026.
- Access: portal page HTTP 200, but file links are not in the static HTML (JS). No API found.
- Check: UNVERIFIED for data rows. Page HTTP 200, 19,815 bytes; no csv/zip link found in HTML; datos.gob.cl search returned nothing.
- Limits: enrollment is not daily attendance. The attendance dataset (`asistencia-declarada-mensual`) exists on the same portal, also unverified.
- Use in the simulation: TARGET (static). Simulated school-bound person flows per school vs enrolled students.

### EOD Santiago 2012 (origin-destination survey)
- Link: https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/ (old URL sectra.gob.cl/encuestas_movilidad returns 404)
- Contents: trip diary of 18,000 households, all modes, 866 zones. The page links three Google Drive files: Base de Datos (drive id 193675NwL6QNDs6VJFHU0mXiX5-6SdBPJ), Zonificación (`Zonificacion_EOD-2012_Santiago.zip`, id 1SM2C5b3jhbmYCbAkmtg2yWRU2dMQkGRl), Informe (id 13_j6G1vaG1acOuL-W5okvAEJ0aaGKO7e).
- Granularity: household and trip, 866 zones.
- Year: field work 2012 to 2013.
- Access: Google Drive download, zip.
- Check: VERIFIED 2026-11-21 that the files exist: Drive view page HTTP 200 with title `Zonificacion_EOD-2012_Santiago.zip`. Data rows not downloaded.
- Limits: 14 years old, before Metro Lines 3 and 6 and before the pandemic. Do not use as a current target.
- Use in the simulation: INPUT only (behavioral priors: trip rates per person, mode choice parameters). NOT a score target.

### UOCT intersection traffic counts
- Link: https://chileatiende.gob.cl/fichas/23628-solicitar-informacion-de-transito
- Contents: continuous and periodic vehicle counts per intersection, operated by UOCT.
- Granularity: intersection (point). Time step: sub-hourly.
- Year: continuous.
- Access: paid request by email (ventas@uoct.cl), 7.36 UF per intersection for continuous counts. No open download.
- Check: VERIFIED that access is paid (ChileAtiende ficha read); UNVERIFIED for any data.
- Limits: cost and delay make it unusable in a hackathon.
- Use in the simulation: NONE (paid; MOP gantry transactions cover car flow).

### Negative results (searched, nothing open found)
- Bicycle counters for Santiago: no open dataset (datos.gob.cl: 0 results for "contador bicicletas").
- Pedestrian counts: no open dataset found.
- Metro per-station monthly series as a separate open file: not found; the DTPM boardings xlsx is the per-station source.
- datos.gob.cl: no DTPM, UOCT, or Metro mobility datasets at all (API searched 2026-11-21).

## Proposed fixed score

From the 2 best targets (DTPM boardings; DTPM trip table as OD matrix), week of April 11 to 17, 2026, day type LABORAL:

```
S_stop = 1 - SMAPE(sim boardings, obs boardings)   per stop x half hour (Subidas xlsx)
S_od   = 1 - SMAPE(sim OD trips, obs OD trips)     per zona777 pair x hour (Viajes zip)
SCORE  = 0.6 * S_stop + 0.4 * S_od                 in [0, 1], higher is better
```

Run on the same GTFS_20260425 network. A layer ships only if SCORE goes up.

## Critique

1. The two biggest files (Viajes, Etapas) were verified by headers and first bytes only. Row counts and real column values are not confirmed; clever-koala is downloading them in full.
2. The only verified car-flow target is monthly PDFs per concession. Extracting gantry numbers from PDF charts may be too lossy to score against. Nothing sub-monthly exists openly for cars.
3. Census manzana, MINEDUC enrollment, and EOD 2012 have zero downloaded rows. The static targets in the score are therefore all unproven; the proposed score leans 100% on one week of DTPM data from one operator's estimation pipeline.
