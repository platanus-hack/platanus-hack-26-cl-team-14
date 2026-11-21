# Score targets

A score target is an observed count that the simulation output can be compared with.
Each entry says: time step, spatial unit, date range, and the simulated quantity that it measures.

### DTPM boardings by stop and station, by half hour (April 2026)
- Link: https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2026.04_v3.xlsx (index: https://www.dtpm.cl/index.php/documentos/matrices-de-viaje)
- Contents: one row = average boardings per stop or station per half hour. Columns: Modo (BUS/Metro/Tren), Paradero, SIMIT, Comuna, Media_hora, Subidas_Promedio, by day type (LABORAL/SABADO/DOMINGO). Sheets: FICHA_DATOS, DICCIONARIO, SUBIDAS_2026_04, RESUMEN.
- Granularity: point (stop or station). Time step: 30 minutes.
- Year: week of April 11 to 17, 2026.
- Access: direct download, xlsx, 29.3 MB.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 29,342,772 bytes downloaded to `data/raw/dtpm/subidas_2026_04.xlsx`; sheet names, column names, and real stop codes (for example `E-1-4-SN-50`) read with `tools/scrape/dtpm_subidas_peek.py`. Full records not parsed yet.
- Limits: expanded from bip! validations; fare evasion is not included (DTPM estimates it near 20 to 30% on buses, so bus counts are a floor, Metro counts are near complete). Averages per day type, not raw days.
- Role: TARGET
- Use in the simulation: primary score: simulated boardings per stop per half hour vs this table.

### DTPM trip table (viajes) April 2026
- Link: https://www.dtpm.cl/descargas/modelos_y_matrices/Viajes_2026-04.zip
- Contents: one row = one expanded public transport trip. Columns (from structure file): tipodia, factor_expansion (OD matrix, zonas777), n_etapas, tviaje, distancia_eucl, distancia_ruta, tiempo_inicio_viaje, plus origin and destination zones.
- Granularity: trip level, zones (zonas777). Time step: timestamp (continuous).
- Year: April 11 to 17, 2026.
- Access: direct download, zip, 1.66 GB. Column spec: https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura%20Tabla%20de%20Viajes%202026%204.txt (downloaded).
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, Content-Length 1,658,733,451, first 1 KB has a valid ZIP signature (`PK` + internal file "Ta..."), structure txt downloaded with real column list. No data rows read; clever-koala owns full download.
- Limits: destinations are estimated (ADATRAP method), not observed. Only public transport. Same evasion bias as boardings.
- Role: TARGET
- Use in the simulation: aggregate to an OD matrix by zone and hour and compare with simulated trips.

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
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 58,028 bytes of index HTML with the file links listed. No archive downloaded.
- Limits: method changed in April 2017 (daily estimation); older cuts are weekly. Zone systems can differ between years.
- Role: TARGET
- Use in the simulation: trend check: the same score computed on a second week guards against overfitting to one week.

### DTPM GTFS feed of March 21, 2026 (in service during the trip week)
- Link: https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip
- Contents: routes, stops, timetables, shapes of the Santiago public transport network.
- Granularity: stop and timetable level.
- Year: feed dated March 21, 2026; it was the current feed during April 11 to 17 (the next feed is GTFS_20260425_final.zip, HTTP 206, ZIP signature, 10,709,254 bytes).
- Access: direct download, zip.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206 range request, first bytes have a valid ZIP signature. Credit: clever-koala found the matching feed.
- Limits: exact service validity dates inside the feed (calendar.txt) not read yet.
- Role: INPUT
- Use in the simulation: the network that the simulated agents travel on. Use this feed, not the April 25 one, so the score compares the same network as the trip week.

### INE monthly Metro ridership (press series)
- Link: https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2026/09/01/personas-transportadas-por-metro-de-santiago-alcanzaron-un-total-de-53-5-millones-en-julio-de-2026
- Contents: total persons transported by Metro de Santiago per month (for example 53.5 million in July 2026), from the INE transport bulletin.
- Granularity: whole network. Time step: month.
- Year: continuous series, at least 2025 to 2026.
- Access: press pages; the bulletin has xlsx series on the INE transport statistics page (HTTP 200, file links behind a JS portal).
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 97,042 bytes, the observed value "53,5 millones" read in the page text.
- Limits: one number per month for the whole network. No station detail.
- Role: TARGET (weak)
- Use in the simulation: sanity check of total simulated Metro demand and seasonality.

### MOP concession dipticos: urban highway transactions per toll gantry
- Link: https://concesiones.mop.gob.cl/uploads/sites/4/2025/01/Diptico_VS_2024_01.pdf (Vespucio Sur; one PDF per concession per month on concesiones.mop.gob.cl)
- Contents: monthly toll transactions per gantry (pórtico), vehicle category split, year-on-year comparison, from SICE reports.
- Granularity: toll gantry (point on the highway). Time step: month.
- Year: about 2021 to 2025, by concession.
- Access: direct download, PDF. Numbers must be extracted from charts and tables by hand or with a PDF parser.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 583,625 bytes, valid `%PDF-1.7` header. The Sistema Norte-Sur diptico URL returned 200 but Content-Length 2,518, likely an error page: UNVERIFIED for that concession.
- Limits: PDF only, no bulk data. Numbers sit in charts, not tables; extraction may be too lossy and slow for the hackathon (keen-heron critique, accepted). Transactions count vehicles with TAG, not persons.
- Role: TARGET (lead only)
- Use in the simulation: the only open observed count for private car flow (per gantry per month). Treat as a lead, not a score component, until one PDF is extracted cleanly. See also `traffic-counts.md` (bold-panda) for INE toll series.

### INE Censo 2024: persons and dwellings per manzana
- Link: https://censo2024.ine.gob.cl/resultados/
- Contents: persons and dwellings per manzana, plus manzana polygons (GeoParquet).
- Granularity: manzana. No time step (one cut).
- Year: 2024 (published December 2025).
- Access: JS portal; direct file URL not exposed in the HTML. Third-party mirrors exist (github.com/pachadotdev/censo2024-cartografia-postgresql, github.com/bastianolea/censo_2024_mapas). Also cataloged in `context/research/data-sources-buildings-activity.md`.
- Check: UNVERIFIED for data rows. Portal page HTTP 200, 168,990 bytes, but no file link in the HTML; no download executed.
- Limits: static. Counts residents, not daytime population.
- Role: TARGET and INPUT
- Use in the simulation: where simulated agents sleep must match this distribution. Also the base population input.

### MINEDUC enrollment per school (matrícula)
- Link: https://datosabiertos.mineduc.cl/matricula-por-estudiante/
- Contents: one row per enrolled student per year, with school (RBD), grade, and comuna; also per-school summaries. Schools are georeferenced in the MINEDUC directory.
- Granularity: school (point via directory join). Time step: year.
- Year: about 2004 to 2026.
- Access: portal page HTTP 200, but file links are not in the static HTML (JS). No API found.
- Check: UNVERIFIED for data rows. Page HTTP 200, 19,815 bytes; no csv/zip link found in HTML; datos.gob.cl search returned nothing.
- Limits: enrollment is not daily attendance. The attendance dataset (`asistencia-declarada-mensual`) exists on the same portal, also unverified. Note: the MINEDUC school DIRECTORY page does expose direct files (`Directorio-EE-Oficial-2026.rar`, HTTP 206, real `Rar!` signature), so the matrícula files likely exist behind a similar path.
- Role: TARGET (static)
- Use in the simulation: simulated school-bound person flows per school vs enrolled students.

### EOD Santiago 2012 (origin-destination survey)
- Link: https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/ (old URL sectra.gob.cl/encuestas_movilidad returns 404)
- Contents: trip diary of 18,000 households, all modes, 866 zones. The page links three Google Drive files: Base de Datos (drive id 193675NwL6QNDs6VJFHU0mXiX5-6SdBPJ), Zonificación (`Zonificacion_EOD-2012_Santiago.zip`, id 1SM2C5b3jhbmYCbAkmtg2yWRU2dMQkGRl), Informe (id 13_j6G1vaG1acOuL-W5okvAEJ0aaGKO7e).
- Granularity: household and trip, 866 zones.
- Year: field work 2012 to 2013.
- Access: Google Drive download, zip.
- Check: link VERIFIED 2026-11-21 (Drive view page HTTP 200 with title `Zonificacion_EOD-2012_Santiago.zip`); file bytes NOT downloaded, so UNVERIFIED for rows.
- Limits: 14 years old, before Metro Lines 3 and 6 and before the pandemic. Do not use as a current target.
- Role: INPUT only
- Use in the simulation: behavioral priors (trip rates per person, mode choice parameters). NOT a score target.

### UOCT intersection traffic counts
- Link: https://chileatiende.gob.cl/fichas/23628-solicitar-informacion-de-transito
- Contents: continuous and periodic vehicle counts per intersection, operated by UOCT.
- Granularity: intersection (point). Time step: sub-hourly.
- Year: continuous.
- Access: paid request by email (ventas@uoct.cl), 7.36 UF per intersection for continuous counts. No open download.
- Check: VERIFIED that access is paid (ChileAtiende ficha read); UNVERIFIED for any data.
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
- Use in the simulation: the only observed non-transit mobility count found. Daily simulated bicycle passages at counter sites vs this series. Candidate third score term once cycling exists in the model.

### Negative results (searched, nothing open found)
- Pedestrian counts: no open dataset found.
- Metro per-station monthly series as a separate open file: not found; the DTPM boardings xlsx is the per-station source.
- datos.gob.cl: no DTPM, UOCT, or Metro mobility datasets at all (API searched 2026-11-21).

## Proposed fixed score

From the 2 best targets (DTPM boardings; DTPM trip table as OD matrix), week of April 11 to 17, 2026, day type LABORAL, on the GTFS_20260321_v3 network:

```
SMAPE(s, o) = mean over cells of |s - o| / ((s + o) / 2), range [0, 2]; score term = 1 - SMAPE/2, range [0, 1].
Cells: only stop x half hour with obs >= 10 boardings (keeps denominators > 0 and kills near-zero blowup).
Evasion: scale observed bus boardings by 1/(1-e), e = 0.25; Metro and Tren unscaled.
S_stop on boardings per stop x half hour (Subidas xlsx); S_od on trips per zona777 OD pair x hour, obs >= 10 (Viajes zip).
SCORE = 0.6 * S_stop + 0.4 * S_od, in [0, 1]; a layer ships only if SCORE goes up.
```

Known limits of this score, accepted from critique (keen-heron, gentle-fox): it only observes public transport, about 35 to 40% of Santiago trips; cars and walking have no open observed counts (see UOCT and MOP entries), and cycling has only counter sites (MINVU Eco-Counter entry). It has no housing or work term, so it cannot judge a scenario like "remove the MUT" by itself. The first candidate extension is a static residential term against Censo 2024 manzana counts, once those rows are verified.

## Critique

1. The two biggest files (Viajes, Etapas) were verified by headers and first bytes only. Row counts and real column values are not confirmed; clever-koala is downloading them in full.
2. The only verified car-flow target is monthly PDFs per concession. Extracting gantry numbers from PDF charts may be too lossy to score against. Nothing sub-monthly exists openly for cars.
3. Census manzana, MINEDUC enrollment, and EOD 2012 have zero downloaded rows. The static targets in the score are therefore all unproven; the proposed score leans 100% on one week of DTPM data from one operator's estimation pipeline.
