# Open-data portal sweep

## Brief

- Task: list portal datasets that can feed the Santiago urban simulation.
- Scope: official and university catalogs for the Región Metropolitana. Nationwide layers stay when they contain Santiago.
- Method: `tools/scrape/portals_sweep.py` uses only the Python standard library. It waits at least one second between requests to each host.
- Output: `data/raw/portals-sweep/datasets.csv` has 871 rows. The columns are portal, title, publisher, format, year, URL, and link status.
- Good result: a source has a working access path, real rows, useful spatial detail, and a clear simulation mechanism.

## Portal results

| Portal | Catalog records | Dataset records kept for RM | Dead links | Check |
| --- | ---: | ---: | ---: | --- |
| IDE Chile Geoportal | 846 | 403 | 0 catalog links | VERIFIED 2026-11-21. All 170 public result pages returned HTTP 200. The old public CSW now redirects to `/login`. Linked services were not checked in bulk. |
| Las Condes Open Data | 108 inventory rows; 49 published layers | 49 | 0 of 49 downloads | VERIFIED 2026-11-21. The POST catalog returned 49 layers. One direct download for each layer returned HTTP below 400. |
| Gobierno de Santiago ArcGIS | 256 public items; 102 dataset items | 102 | 0 catalog links | VERIFIED 2026-11-21 through the ArcGIS search API for owner `UgitGore_01`. Two shortlisted services returned rows. Linked services were not checked in bulk. |
| Municipalidad de Santiago ArcGIS | 412 public items; 140 dataset items | 140 | 0 catalog links | VERIFIED 2026-11-21 through the ArcGIS search API for owner `IMS_Observatorio`. Four shortlisted services returned rows. Linked services were not checked in bulk. |
| Observatorio de Ciudades UC | 209 catalog items; 177 dataset items | 177 | 0 catalog links | VERIFIED 2026-11-21 through ArcGIS group `46da166267944c039ec26987be335330`. Seven shortlisted services returned rows. Linked services were not checked in bulk. |
| datos.gob.cl | Not counted | Not counted | Not counted | UNVERIFIED. `robots.txt` disallows `/api/` and sets a 10-second crawl delay. The script does not call the CKAN API. |
| Puente Alto | 3,161 resources shown by the portal | Not counted | Not counted | UNVERIFIED. The Junar catalog API requires an authentication key. The public search page is not a catalog API. |
| INE Geodatos Abiertos | Not counted | Not counted | 1 of 1 RM item tested | UNVERIFIED. The official page lists old ArcGIS Hub links. The Hub returns 403, and the tested RM item is inaccessible. |
| Maipú | Not counted | Not counted | Not counted | UNVERIFIED. The Google Sites catalog has no public catalog API. |
| Providencia | Not counted | Not counted | 1 dead portal | UNVERIFIED. `datos.providencia.cl` fails DNS. |
| Ñuñoa | Not counted | Not counted | 1 dead portal | UNVERIFIED. `datos.nunoa.cl` fails DNS. |

`Dead links` means the links that the script checked. It does not claim that every linked IDE or ArcGIS service is alive.

## The 20 most useful datasets

### 1. Santiago Census 2024 by block
- Link: https://services9.arcgis.com/zqDAjElD9y8GPDk4/arcgis/rest/services/Datos_a_nivel_de_manzana_CENSO_2024/FeatureServer
- Contents: One block polygon. Main columns include people, age, migration, work status, industry, transport mode, households, tenure, housing type, and services.
- Granularity: manzana.
- Year: 2024.
- Access: ArcGIS Feature Service. JSON, GeoJSON, and query API.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was block `13101011001011`, with 368 people and 200 households.
- Role: TARGET
- Limits: This service covers the comuna of Santiago, not all 52 RM comunas. Census counts do not give daily schedules.
- Use in the simulation: INPUT for the synthetic population. TARGET for people, household, dwelling, and transport-mode totals by block.

### 2. Las Condes Census 2017 by block
- Link: https://datos.lascondes.cl/proxy/s3/exports/open-data/shapes/censo_2017/geojson/censo_2017.geojson
- Contents: One census block. Main columns include people by sex and age, dwellings by type and occupancy, household count, and construction material.
- Granularity: manzana.
- Year: 2017.
- Access: direct download. CSV, GeoJSON, and Shapefile.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first CSV row was block `13114141003500`, with 776 people and 182 dwellings.
- Role: TARGET
- Limits: It covers only Las Condes. The data predates the 2024 census.
- Use in the simulation: INPUT for block-level households. TARGET for the Las Condes population and housing distribution.

### 3. Georeferenced building permits in Chile
- Link: https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/Permisos_edificacion_2010_2018/FeatureServer
- Contents: One permit point. Columns include region, comuna, year, unit count, destination, use, floors, and floor area.
- Granularity: point.
- Year: 2010-2018.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row contained comuna, year, units, use, floors, and square metres.
- Role: INPUT
- Limits: The series stops in 2018. A permit does not prove that construction finished or that residents moved in.
- Use in the simulation: INPUT for the age, size, use, and capacity of housing supply.

### 4. Las Condes property master
- Link: https://datos.lascondes.cl/proxy/s3/exports/open-data/shapes/maestro_predios/geojson/maestro_predios.geojson
- Contents: One property record. Columns include address, number, SII role, and full address.
- Granularity: point or property record.
- Year: generated 2026. Source reference year is not stated.
- Access: direct download. CSV, GeoJSON, and Shapefile.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first CSV row was `VITAL APOQUINDO 1200`, SII role `2750-4`.
- Role: INPUT
- Limits: It covers only Las Condes. The sample has no dwelling count, floor area, or use.
- Use in the simulation: INPUT for property locations and address joins.

### 5. Social-housing projects in Chile
- Link: https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/Vivienda_Social_en_Chile_de_1936_a_2013/FeatureServer
- Contents: One project point. Columns include project name, comuna, units, permit year, floor area, and distance to education, health, and emergency services.
- Granularity: point.
- Year: 1936-2013.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row had 130 units, permit year 2009, and 61.1 square metres.
- Role: INPUT
- Limits: The series stops in 2013. Distances do not show service quality or current occupancy.
- Use in the simulation: INPUT for subsidized housing locations, capacity, and service access.

### 6. Santiago business-density zones
- Link: https://services9.arcgis.com/zqDAjElD9y8GPDk4/arcgis/rest/services/DENSIDADES_DIRECTORIO_NACIONAL_DE_EMPRESAS_2017_STGO_gdb/FeatureServer
- Contents: One density polygon. Columns include density level, economic activity, industry, date, and source.
- Granularity: zone.
- Year: 2017.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was a low-density manufacturing zone.
- Role: TARGET
- Limits: It gives density classes, not establishment points or worker counts. It covers only the comuna of Santiago.
- Use in the simulation: INPUT for job-attraction zones. TARGET for the spatial distribution of economic activity.

### 7. Santiago municipal-patent density zones
- Link: https://services9.arcgis.com/zqDAjElD9y8GPDk4/arcgis/rest/services/DENSIDAD_DE_PATENTES_COMERCIALES_MUNICIPALES_AÑO_2019_gdb/FeatureServer
- Contents: One density polygon. Columns include density level, patent type, date, and source.
- Granularity: zone.
- Year: 2019.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was a medium-density commercial-patent zone.
- Role: TARGET
- Limits: A patent is not a worker or visitor count. The data covers only the comuna of Santiago and is seven years old.
- Use in the simulation: INPUT for retail and service destinations. TARGET for commercial concentration.

### 8. Destination-to-work trips by EOD zone
- Link: https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/Cantidad_de_viajes_de_destino_trabajo_EOD_2006/FeatureServer
- Contents: One EOD zone. Columns include comuna and trips by purpose, including work, education, shopping, health, and home.
- Granularity: zone.
- Year: 2006.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was zone `63621` in La Florida, with 744 work trips and 7,432 total trips.
- Role: TARGET
- Limits: It is 20 years old. It predates Metro Lines 3 and 6, the RED redesign, and the pandemic.
- Use in the simulation: TARGET for destination-purpose shares only after a strong age penalty. Do not use it as the sole current behavior source.

### 9. Vehicle flow on Santiago road segments
- Link: https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/Calculo_de_flujo_vehicular_en_AMS/FeatureServer
- Contents: One road segment. Columns include street, comuna, EOD zone, traffic, and vehicles per hour.
- Granularity: line.
- Year: not stated in the service.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was Alonso de Córdova in Vitacura, with 1,111 vehicles per hour.
- Role: TARGET
- Limits: The service does not state the observation year or time window. Modeled flow must not be treated as a direct count without its method.
- Use in the simulation: TARGET for private-vehicle flow by road segment.

### 10. GORE floating-population and stop demand
- Link: https://services7.arcgis.com/ijNx96DNqvwR7rbt/arcgis/rest/services/Población_flotante/FeatureServer
- Contents: One public-transport stop. Columns include stop, comuna, frequencies, capacity, boardings, alightings, waiting time, and crowding.
- Granularity: point. One stated sample period is a weekday at 08:30.
- Year: not stated in the service.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was stop `PA476`, with 120 boardings, 656 alightings, and 776 combined movements.
- Role: TARGET
- Limits: The service does not state the source date. The title `Población flotante` hides that the rows are stop-demand records.
- Use in the simulation: TARGET for boardings, alightings, waiting time, and stop crowding.

### 11. Las Condes RED stops
- Link: https://datos.lascondes.cl/proxy/s3/exports/open-data/shapes/paraderos_red/geojson/paraderos_red.geojson
- Contents: One stop. Columns include stop code, street segment, services, frequency, capacity, and coordinates.
- Granularity: point.
- Year: generated 2026. Network reference date is not stated.
- Access: direct download. CSV, GeoJSON, and Shapefile.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first CSV row was stop `PC510`, served by five routes.
- Role: INPUT
- Limits: It covers only Las Condes. The portal does not state the network feed date.
- Use in the simulation: INPUT for bus access, service choice, and local capacity.

### 12. Las Condes RED route geometry and capacity
- Link: https://datos.lascondes.cl/proxy/s3/exports/open-data/shapes/ruta_buses_red/geojson/ruta_buses_red.geojson
- Contents: One route-direction geometry. Columns include route, operator, distance, direction, frequency, and peak capacity.
- Granularity: line and route direction.
- Year: generated 2026. Network reference date is not stated.
- Access: direct download. CSV, GeoJSON, and Shapefile.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first CSV row was route `507I`, with peak frequencies and capacities.
- Role: INPUT
- Limits: It covers routes that pass through Las Condes. It is not a complete current GTFS feed.
- Use in the simulation: INPUT for bus paths, frequency, and capacity.

### 13. Metro network, March 2025
- Link: https://services9.arcgis.com/zqDAjElD9y8GPDk4/arcgis/rest/services/RED_DE_METRO_DE_SANTIAGO_MARZO_2025_gdb/FeatureServer
- Contents: One station or network feature. Columns include station, line, source, and date.
- Granularity: point and line layers.
- Year: 2025.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was La Cisterna on Line 4A.
- Role: INPUT
- Limits: The service is a March 2025 snapshot. It does not include schedules, capacity, or observed passengers.
- Use in the simulation: INPUT for Metro access and network paths.

### 14. RM road infrastructure from Census 2017
- Link: https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/Calles_RM_Censo_2017/FeatureServer
- Contents: One street segment. Columns include region, comuna, street name, and road class.
- Granularity: line.
- Year: 2017.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was Avenida Sergio Vieira de Mello in Macul.
- Role: INPUT
- Limits: It is a 2017 network. It has no speed, lane, congestion, or access attributes.
- Use in the simulation: INPUT for the base street graph when a newer road source is unavailable.

### 15. GORE metropolitan green areas
- Link: https://services7.arcgis.com/ijNx96DNqvwR7rbt/arcgis/rest/services/Areas_Verdes_PULSO/FeatureServer
- Contents: One green-area polygon. Columns include comuna, type, implementation state, cover, origin, area, and hectares.
- Granularity: polygon.
- Year: not stated in the service.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was a 0.372-hectare unconsolidated area in Recoleta.
- Role: TARGET
- Limits: The service does not state the observation year. Planned and implemented spaces appear together.
- Use in the simulation: INPUT for recreation destinations. TARGET for implemented green area by comuna.

### 16. PRMS metropolitan green areas
- Link: https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/ÁREAS_VERDES_PRMS_2013/FeatureServer
- Contents: One planning polygon. Columns include area, comuna, zone, permitted uses, prohibited uses, and PRMS rule.
- Granularity: polygon.
- Year: 2013 publication; the sampled rule refers to PRMS 1994.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was part of Cerro Los Piques in Las Condes.
- Role: INPUT
- Limits: This is planning regulation, not current vegetation or public access. Some text has broken character encoding.
- Use in the simulation: INPUT for land-use constraints and future green-space capacity.

### 17. Cultural facilities in the RM
- Link: https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/Equipamiento_Cultural_RM/FeatureServer
- Contents: One facility point. Columns include name, address, comuna, and facility type.
- Granularity: point.
- Year: not stated in the service.
- Access: ArcGIS Feature Service.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was Goethe Institut in Santiago, classified as a theatre or concert hall.
- Role: INPUT
- Limits: It has no capacity, opening hours, events, or visitor counts. The current status is unknown.
- Use in the simulation: INPUT for culture and entertainment destinations.

### 18. Las Condes municipal events
- Link: https://datos.lascondes.cl/proxy/s3/exports/open-data/shapes/eventos_municipales/geojson/eventos_municipales.geojson
- Contents: One event occurrence. Columns include event name, address, coordinates, start, end, category, and target audience.
- Granularity: point and event time range.
- Year: sampled row 2023. Export generated 2026.
- Access: direct download. CSV, GeoJSON, and Shapefile.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first CSV row was a cultural course or workshop on 2023-04-01.
- Role: INPUT
- Limits: The export can contain old events. It has no attendance or venue capacity.
- Use in the simulation: INPUT for time-bound leisure destinations and event travel.

### 19. Las Condes address master
- Link: https://datos.lascondes.cl/proxy/s3/exports/open-data/maestro_direcciones_prod/maestro_direcciones_prod.csv
- Contents: One address. Columns include street, number, full address, coordinates, territorial code, and neighborhood unit.
- Granularity: point.
- Year: updated 2026.
- Access: direct CSV download.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first row was `ABADIA 8` with coordinates and neighborhood unit `C-19`.
- Role: INPUT
- Limits: It covers only Las Condes. It does not identify households, residents, or building capacity.
- Use in the simulation: INPUT for geocoding properties, stops, services, and events.

### 20. Las Condes supermarkets
- Link: https://datos.lascondes.cl/proxy/s3/exports/open-data/shapes/supermercados/geojson/supermercados.geojson
- Contents: One supermarket point. Columns include name, brand, address, coordinates, and neighborhood unit.
- Granularity: point.
- Year: generated 2026. Source reference date is not stated.
- Access: direct download. CSV, GeoJSON, and Shapefile.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200. The first CSV row was Express Lider at Alexander Fleming 7750.
- Role: INPUT
- Limits: It covers only Las Condes. It has no store size, opening hours, sales, or visitor counts.
- Use in the simulation: INPUT for grocery destinations and local trip attraction.

## Negative results

- The official datos.gob.cl publisher guide documents CKAN `package_search`. The current `robots.txt` forbids `/api/`, so this sweep did not call it.
- The IDE Chile page still documents CSW. The current CSW URL redirects to a login page. The public HTML catalog remains available.
- The old Providencia and Ñuñoa data hostnames do not resolve.
- Puente Alto exposes Junar dataset endpoints, but the API needs an authentication key.
- Maipú publishes a Google Sites list, not a machine-readable catalog API.
- The INE official page links to its old ArcGIS Hub. The Hub now returns 403, and the tested RM dataset item is inaccessible.

## Critique

1. The RM filter uses titles, descriptions, publishers, and comuna names. It can miss a national dataset whose metadata does not say Chile or a local place.
2. The dead-link count is complete only for Las Condes downloads. IDE and ArcGIS counts cover live catalog links, not every downstream service.
3. Several useful services omit the source year or method. This catalog does not resolve those dates, licences, or model provenance.
