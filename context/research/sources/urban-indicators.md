# Urban indicator sets for Santiago

Checked on 2026-11-21. “Below comuna” means that the published set has values for a smaller spatial unit, not only a comuna score.

### SIEDU, Sistema de Indicadores y Estándares de Desarrollo Urbano (INE + CNDU)
- Link: https://www.ine.gob.cl/herramientas/portal-de-mapas/siedu/ and https://geodatos.ine.cl/Geografia_APPS/Publico/matriz_siedu_publicacion.xlsm
- Contents: A workbook with the SIEDU indicator matrix. Indicators cover services, sustainable mobility, environment, housing and neighbourhoods, planning, economy, heritage, and participation.
- Granularity: comuna, city, urban area, manzana, and other indicator-specific geographies. It goes below comuna for several spatial indicators, but not for every series.
- Year: Indicator-specific historical years. The page says the downloadable files were updated after methodological validation in 2025.
- Access: direct download, XLSM, 516,275 bytes.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 516,275 bytes; `file` identifies Microsoft Excel 2007+. VERIFIED-ROWS: a Santiago CUT 13101 row was parsed during independent review.
- Role: INPUT for comuna and indicator calibration. Below-comuna layers require a separate SIEDU cartographic download.
- Limits: Different indicators use different denominators and geographies. The workbook is a publication matrix, not one harmonised time series. Raw sources include INE Census, MINVU, transport data, SII, SEC, CONAF and other public records, depending on the indicator.
- Use in the simulation: access to services, travel time and transport coverage, green space, housing deficit, internet, land use, municipal capacity, and civic participation.

### SIET-Chile, Sistema de Indicadores y Estándares Territoriales (INE, CNDT, MINVU, ODEPA)
- Link: https://www.siet-chile.cl/descargables/ and https://www.siet-chile.cl/wp-content/uploads/2026/02/Resultados_SIET-Chile_04Dic2025-1.xlsx?ver=1790088400
- Contents: Published results and metadata for a national territorial indicator catalogue. The initial release has 27 indicators, including access to schools, primary health, police stations, green areas, and exposure to hazards.
- Granularity: The platform states that it works at several geographic scales. For Santiago, use the downloadable geodatabase and workbook to determine the indicator-specific unit. It can go below comuna where a layer uses census microdata or a spatial grid.
- Year: Initial release dated 2025-12-04; 75% of the first 27 indicators use Census 2024 microdata.
- Access: direct download, XLSX. The page also lists a GDB ZIP and metadata PDFs.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 152,606 bytes. VERIFIED-ROWS: independent review parsed 345 comuna rows and 56 province rows from the XLSX.
- Role: INPUT for territorial access and exposure indicators. Below-comuna values require the separate geodatabase or spatial layers.
- Limits: The site is new and still expanding to 92 indicators. The first release does not equal a complete historical series. Do not mix SIET values with older SIEDU values without reading the metadata.
- Use in the simulation: baseline access, social and environmental exposure, and comuna-to-locality comparisons.

### Índice de Prioridad Social (IPS), Seremi de Desarrollo Social y Familia Metropolitana
- Link: https://www.desarrollosocialyfamilia.gob.cl/storage/docs/INDICE-DE-PRIORIDAD-SOCIAL-2022_V2.pdf
- Contents: Composite social-priority score plus component indicators for income, education, and health. The report includes the comuna ranking and the component tables.
- Granularity: comuna only. It does not go below comuna.
- Year: IPS 2022; components include RSH October 2021, AFC income 2020, SIMCE 8th grade 2019, and other indicator-specific years.
- Access: direct download, PDF. The web source exposes the PDF; a direct curl request returned an HTML wrapper in this pass.
- Check: UNVERIFIED 2026-11-21 for file bytes. The official web source opened as a PDF and returned the methodology and tables, but direct curl returned HTTP 200 HTML, not a PDF.
- Role: INPUT for comuna-level social stratification. Not a below-comuna target.
- Limits: It is a relative ranking across 52 RMS comunas. It omits variables when periodic and reliable comuna data are not available. The income component uses RSH and AFC coverage, not all residents and workers.
- Use in the simulation: social stratification, targeting, and calibration of housing, education, and health disadvantage.

### Índice de Calidad de Vida Urbana (ICVU 2024), IEUT-UC + CChC
- Link: https://estudiosurbanos.uc.cl/categoria/icvu/ and https://estudiosurbanos.uc.cl/wp-content/uploads/2025/06/Informe-ICVU-2024-1.pdf
- Contents: A multidimensional index for urban quality of life. The 2024 report covers connectivity and mobility, housing and environment, labour conditions, business environment, health and environment, and sociocultural conditions.
- Granularity: comuna and city. It does not publish a below-comuna score in the report.
- Year: 2024 edition, published 2025; values are indicator-specific and may use earlier source years.
- Access: direct download, PDF, 3,028,236 bytes.
- Check: VERIFIED 2026-11-21, HTTP 200; `file` identifies PDF 1.3.
- Role: INPUT for comuna-level quality-of-life calibration. Not a below-comuna target.
- Limits: The index covers 99 Chilean comunas with more than 50,000 inhabitants. It is not a complete Santiago population surface and its dimensions combine data with different years.
- Use in the simulation: comuna-level calibration for mobility, housing, work, health, and social opportunity.

### Indicadores de Sustentabilidad Urbana (ISU), CEDEUS
- Link: https://observatorio.cedeus.cl/ and https://www.cedeus.cl/indicadores-de-sustentabilidad-urbana-cedeus/
- Contents: A 30-indicator set in five groups: environment, health, equity, accessibility, and governance. It includes water, air, waste, energy, green areas, internet, housing, travel modes, travel time, elections, and emergency response.
- Granularity: city and metropolitan area in the 16-city report. The related Plataforma Municipal covers 95 comunas, but the report itself does not provide a below-comuna Santiago surface.
- Year: report values use the latest available data at December 2020 and describe the pre-pandemic baseline.
- Access: public web platform and report page; PDF download link is exposed by the page, but the historical PDF URL returned an HTML wrapper in this pass.
- Check: VERIFIED 2026-11-21 for the official pages, HTTP 200; the page returned real HTML and the source describes Santiago coverage. Download file: UNVERIFIED as PDF because the server returned HTML for the discovered URL.
- Role: INPUT for city/metropolitan sustainability calibration. Not an hourly or person-level target.
- Limits: The 30 indicators are not one synchronous year. City-level aggregation hides comuna and neighbourhood differences. Some measures are modelled or depend on administrative reporting.
- Use in the simulation: sustainability score, travel behaviour, access to facilities, and environmental constraints.

### Plataforma Municipal de Datos CEDEUS
- Link: https://plataformamunicipal.cedeus.cl/
- Contents: 33 indicators for the sustainability of 95 Chilean comunas, with methods, sources, temporal and spatial availability, and standards.
- Granularity: comuna. It does not go below comuna in the platform description.
- Year: indicator-specific; the platform describes a pilot with values for the 95 comunas, not one common reference year.
- Access: public web platform. No static download URL was found in the page during this pass.
- Check: UNVERIFIED 2026-11-21. The page returned HTTP 200 and 11,940 bytes, but independent review identified it as a browser challenge, not data. Data export endpoint is also UNVERIFIED.
- Role: LEAD only until the platform returns indicator rows.
- Limits: The 95-comuna coverage is partial nationally. A platform value can be a derived indicator rather than a raw observation. Check the method and source for each indicator before using it as a model input.
- Use in the simulation: comuna-level sustainability calibration and comparison of Santiago comunas.

### Matriz de Bienestar Humano Territorial (MBHT), CIT-UAI + SUBDERE
- Link: https://plataformabht.subdere.gov.cl/visualizations and https://plataformabht.subdere.gov.cl/assets/files/Manual-Plataforma-MBHT-Publico.pdf
- Contents: 18 normalised indicators in accessibility, environmental, socioeconomic, and security dimensions. Examples include access to green areas, cultural, sports, health and education facilities, population, and crime-related measures.
- Granularity: manzana and territorial zones, as well as comuna and region views. It goes below comuna.
- Year: The platform was built progressively from 2021 to 2022. Component years vary by source.
- Access: public visualisation with a “Descargar datos” action; methodology manual is a direct PDF download, 1,998,405 bytes. The data export endpoint was not extracted.
- Check: VERIFIED 2026-11-21, HTTP 200 for the visualisation page, 400,213 bytes; HTTP 200 for the official manual, PDF 1.7.
- Role: INPUT for below-comuna opportunity and access surfaces. Export rows still require a separate download check.
- Limits: Values are normalised from 0 to 1 within the analysed territory. They are not directly comparable across releases or territories without the method. Inputs include Census 2017, DEIS, MINEDUC, MINAGRI, crime records, OSM, satellite interpretation, and municipal or regional inputs.
- Use in the simulation: fine-grained access and opportunity surfaces, especially for neighbourhood-level housing, services, safety, and amenity scenarios.

### Atlas Comunal de Áreas Prioritarias de Acción Social (APAS), Ministerio de Desarrollo Social
- Link: https://www.comicivyt.cl/wp-content/uploads/2025/03/MIDESO-2017-Atlas_de_Accion_Social.pdf
- Contents: Maps and territorial prioritisation based on converging social disadvantage indicators. The method uses aggregated RSH information and identifies areas for social action.
- Granularity: comuna and smaller mapped territorial units, depending on the map and indicator. The PDF is not a machine-readable point or person dataset.
- Year: 2017 atlas and source-period-specific indicators.
- Access: direct download, PDF, 31,751,664 bytes.
- Check: VERIFIED 2026-11-21, HTTP 200; `file` identifies PDF 1.7.
- Role: HISTORICAL INPUT and spatial-priority validation. Not a current person-level target.
- Limits: The atlas is a static publication. RSH data are aggregated and protected. The 2017 snapshot is old for current migration, housing, and employment patterns. Map extraction requires georeferencing or a separate source file.
- Use in the simulation: historical social disadvantage priors and validation of spatial targeting. Do not use as current person-level data.

### Centro de Datos OCUC, Observatorio de Ciudades UC
- Link: https://observatoriodeciudades.com/centro-de-datos-ocuc/
- Contents: Open geospatial layers from multiple sources, including a collection of territorial indicators and indicator systems. Each dataset has provenance, reference period, and use conditions.
- Granularity: Dataset-specific. The catalogue can include points, polygons, comuna, and smaller spatial units. The catalogue page does not guarantee below-comuna data for every indicator.
- Year: Dataset-specific. Read each layer's metadata.
- Access: public catalogue with dataset-level downloads. Direct file URL and format depend on the selected layer.
- Check: VERIFIED 2026-11-21 at the web-source level: the official page opened and described 209 layers, including 10 indicator layers. Direct curl timed out after 20 seconds, so file bytes are UNVERIFIED.
- Role: CATALOGUE LEAD for finding finer-grained geospatial inputs. Each selected layer needs its own row or byte verification.
- Limits: OCUC is a catalogue, not one harmonised indicator series. Licences, update dates, spatial units, and raw sources vary by layer. Do not treat the catalogue update date as the data year.
- Use in the simulation: locate fine-grained urban layers and raw geospatial inputs for access, land use, housing, and mobility.

## Critique

1. Most sets publish derived indicators, not raw event-level data. The simulation still needs the underlying population, trips, housing, and facility records.
2. Years are not aligned. A 2024 index can combine 2020, 2021, and older administrative sources.
3. SIEDU and SIET now have bounded row evidence, but below-comuna coverage still needs separate cartographic layers. CEDEUS Municipal returned a browser challenge, and MBHT and OCUC exports remain unverified.
