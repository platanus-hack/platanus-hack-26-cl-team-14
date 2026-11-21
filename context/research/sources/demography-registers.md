# Demography and registers: aggregate sources for Santiago

Topic: public registers of people and vehicles, as aggregate statistics. No named persons.
Author: clever-fox (claude-haiku-5-5). Checks run 2026-11-21 with `curl`.

Summary: the best checked files are the INE population projections by comuna (bytes and header only) and the SERMIG residence-permit table (bytes and header only, region level). Neither is a TARGET. Neither gives comuna-level flows or resident stock. Vehicle, electoral, and birth data by comuna are still open. The driver-licence file we found holds personal data and must not be used.

## Entries

### INE: Estimaciones y proyecciones de población 2002-2035, comunas (base 2017)
- Link: https://www.ine.gob.cl/docs/default-source/proyecciones-de-poblacion/cuadros-estadisticos/base-2017/estimaciones-y-proyecciones-2002-2035-comunas.xlsx (page: https://www.ine.gob.cl/estadisticas-por-tema/demografia-y-poblacion/estimaciones-y-proyecciones-de-poblacion)
- Contents: one sheet "Est. y Proy. de Pob. Comunal". Columns: Region, Comuna, Edad, Provincia, then the population by year as `Poblacion 2002` to `Poblacion 2035`. Each row is one comuna and one age value.
- Granularity: comuna. Time step: one year (2002 to 2035).
- Year: 2002 to 2035 (projection from 2024 onward).
- Access: direct download, xlsx, 11,057,485 bytes.
- Check: VERIFIED-BYTES. 2026-11-21, HTTP 200, xlsx content type, 11,057,485 bytes. Unzipped and read the sheet name and the header columns (shared strings). Data rows NOT parsed (sheet XML is 69 MB).
- Limits: Base 2017, so it may differ from the 2024 census. The INE page shows an estimate at 31 Dec 2023, so check which series this file holds before you use it as a control total. Age granularity: check if it is single year or 5-year groups. No geometry.
- Role: INPUT
- Use in the simulation: INPUT. Modelled population per comuna and age (not an observed count), to set the number of agents per comuna (control total for the ~7 million target). Not a TARGET.

### SERMIG: Datos abiertos, residence permits (RD) acogidas 2000 to 1st semester 2026
- Link: https://serviciomigraciones.cl/wp-content/uploads/estudios/Datos-abiertos/RD/RD_Acogidas_2000_1er_semestre_2026.xlsx (page: https://serviciomigraciones.cl/estudios-migratorios/datos-abiertos/)
- Contents: one sheet. Columns: SEXO, RANGO_ETARIO, PAÍS, ACTIVIDAD, ESTUDIOS, REGIÓN, AÑO, Total. Counts of permits by these variables. Metadata file also linked.
- Granularity: region. No comuna column in this file.
- Year: 2000 to 1st semester 2026 (from file name).
- Access: direct download, xlsx, 1,627,497 bytes.
- Check: VERIFIED-BYTES. 2026-11-21, HTTP 200, valid xlsx (zip), 1,627,497 bytes. Header read (8 columns). Row count 47,335 is from the sheet dimension tag (A1:H47336), not from parsed rows. Years not parsed. Metadata file: HTTP 200, 31,202 bytes.
- Limits: Permits, not residents. The stock is not the same as the number of people living in the city. Counts can change later, per SERMIG. No comuna, so the comuna split must come from another source.
- Role: NONE
- Use in the simulation: NONE. Permits are flows, not resident stock, and the file has no comuna. It cannot set the resident foreign-born share.

### SERMIG: Datos abiertos, residence permits (RT) and resolved permits (RD resueltas), same family
- Link: https://serviciomigraciones.cl/estudios-migratorios/datos-abiertos/ (links on that page: `Datos-abiertos/RT/RT_Acogidas_...xlsx`, `RT_Resueltas_...xlsx`, `RD_Resueltas_...xlsx`, and `Refugio-1er-semestre-2026.zip`)
- Contents: same layout as the RD file, by permit type.
- Granularity: region.
- Year: 2000 to 1st semester 2026.
- Access: direct download, xlsx and zip, sizes not checked.
- Check: UNVERIFIED for the other files. Their URLs are listed on the SERMIG page, but I did not download them. Only the RD acogidas file was checked.
- Limits: as above. Refugee zip not opened.
- Role: NONE
- Use in the simulation: NONE beyond the RD file, until a comuna split exists.

### SERMIG: Minutas comunales de la Región Metropolitana (PDF)
- Link: https://serviciomigraciones.cl/wp-content/uploads/estudios/Minutas-Comuna/RM/Buin.pdf (others in the same folder: Independencia, Recoleta, Estación Central, La Florida, Quinta Normal, San Miguel, La Cisterna, San Joaquin)
- Contents: short PDF per comuna with migrant population estimates. Based on the INE census 2024 and SERMIG records, per the search snippet.
- Granularity: comuna.
- Year: 2022 estimate, some minutas from 2024 to 2025 (per search snippet).
- Access: PDF download.
- Check: UNVERIFIED. Found in search results. I did not open any minuta PDF, so the figures in the search snippet are not checked.
- Limits: PDF, not data. Only some comunas exist. Estimates, not counts.
- Role: INPUT
- Use in the simulation: INPUT, if the figures are confirmed. Share of migrants by comuna, to check the simulated migrant share per comuna. Open the PDFs to check.

### INE: Permisos de circulación (annual vehicle permits)
- Link: https://www.ine.gob.cl/estadisticas-por-tema/comercio-y-servicios/permisos-de-circulacion
- Contents: annual count of permits issued by the municipalities, by vehicle type, catalytic and non-catalytic. The page says data is national and regional. Comuna rankings appear in INE press releases (for 2023: San Gregorio 230 permits, Ollagüe 316 permits).
- Granularity: region (publication level). Comuna only in press releases, not in the file list we could read.
- Year: 2024 is the latest on the page (title: "Permisos de circulación emitidos el 2024"). Press release for 2023 also found.
- Access: download list loads through a POST to `/estadisticas-por-tema/comercio-y-servicios/permisos-de-circulacion/getArchivos/` with a folder ID. The folder IDs are not in the HTML. The file links were not reached.
- Check: UNVERIFIED FOR INGESTION. Page: HTTP 200, 116,908 bytes. The download list was not read, so no file rows were checked.
- Limits: Permits are issued where a vehicle is registered, not where it is used. A Contraloría report (June 2026) found 322 municipalities issued about 146,000 irregular permits. Those are not in the official count, so the count may be off for some comunas.
- Role: INPUT
- Use in the simulation: INPUT, weak. The issuing comuna is not the owner's residence: owners can pay the permit in any comuna (BCN guide: https://www.bcn.cl/api-leyfacil/servicio/ObtenerGuiaPublicadaHTML?uri=permiso-de-circulacion). Use only as a regional car-ownership proxy, after the file is read.

### INE: Parque de vehículos en circulación (regional bulletins)
- Link: https://regiones.ine.cl/documentos/default-source/region-xi/estadisticas/parque-de-vehiculos/boletines/documentos/parque-de-veh%C3%ADculos-en-circulaci%C3%B3n-a%C3%B1o-2024---regi%C3%B3n-de-ays%C3%A9n.pdf
- Contents: vehicle fleet by comuna, for Aysén, 2023 to 2024, from the INE Encuesta Anual de Vehículos en Circulación. This is a regional bulletin. It shows that INE publishes fleet by comuna for at least one region.
- Granularity: comuna (in that bulletin). Santiago file not found.
- Year: 2023 to 2024.
- Access: PDF.
- Check: UNVERIFIED. The URL was found in search results. Not downloaded.
- Limits: Only one region seen. The Metropolitan region file was not located.
- Role: INPUT
- Use in the simulation: INPUT, if a Metropolitan region file exists. Otherwise NONE.

### Municipal permits and licences on datos.gob.cl (CKAN records)
- Link: https://datos.gob.cl/api/3/action/package_search?q=permisos+de+circulacion (results: La Serena 2010, 2023; Angol 2009)
- Contents: records from single municipalities. Found through the CKAN search.
- Granularity: municipality.
- Year: 2009 to 2025 (varies).
- Access: CSV download per resource.
- Check: UNVERIFIED. Catalog metadata only. Resources not downloaded.
- Limits: Few municipalities. Not Santiago's comunas as a set. Likely not usable for the full region.
- Role: NONE
- Use in the simulation: NONE for the full city.

### Licencias de conducir, La Serena 2024 (CSV, municipal)
- Link: https://datos.gob.cl/dataset/71555674-750b-4fc4-a541-9109dbe59d5a/resource/42c44fdf-a74b-4995-b8c4-4c4d21555b12/download/licencias-de-conducir-2024.csv
- Contents: one row per licence issued. Columns include Emitida (date), Clases, Glosa Solicitud, Profesión, Nivel Estudio, Sexo, Fecha Nacimiento (date of birth), Nacionalidad.
- Granularity: person-level records. No comuna column.
- Year: 2024.
- Access: direct download, CSV, 6,478,587 bytes, 71,224 data rows.
- Check: VERIFIED-ROWS (sample only). 2026-11-21, HTTP 200, 6,478,587 bytes, 71,224 lines. The header and the first rows were read. The file was deleted after the check. Personal data: do not keep.
- Limits: Personal data. It has date of birth, sex, nationality, and job per person. It is one municipality only. Do not use it, and do not put it in `data/raw/`. Licence counts by comuna are not in this file.
- Role: NONE
- Use in the simulation: NONE. Privacy risk. Do not use. Do not keep this file, even for a check.

### Servel: Padrón electoral and participation (TOTEM-2 PDF, 2026)
- Link: https://www.servel.cl/wp-content/uploads/2026/03/TOTEM-2.pdf
- Contents: Presentation with voting history for the Valparaíso region (padrón and participation by election, 1988 to 2025). It is a regional table, not per comuna of the Metropolitan region.
- Granularity: region (the file we opened).
- Year: 1988 to 2025.
- Access: PDF.
- Check: VERIFIED-ROWS for the file only. HTTP 200, 1,255,941 bytes, PDF text read on pages 1 and 2 (regional padrón table). This is Valparaíso region data, not Santiago.
- Limits: Not Santiago. Comuna tables for the Metropolitan region were not found. Servel published personal data in 2022 (CIPER report, 2022-04-28, shows the risk). Use only aggregate tables.
- Role: NONE
- Use in the simulation: NONE for this file. The comuna padrón (adult count) would be an INPUT if a Metropolitan file is found.

### Servel: Padrón por comuna (Metropolitan region)
- Link: https://www.servel.cl (search for estadísticas and datos abiertos; section not found by search)
- Contents: per the news search, Servel publishes padrón counts by comuna, including foreign voters (786,466 foreign voters in the latest padrón, per news report).
- Granularity: comuna (expected, not verified).
- Year: 2021 to 2026 (per news).
- Access: unknown.
- Check: UNVERIFIED. Homepage HTTP 200, 269,256 bytes. No comuna table found.
- Limits: Adult count only. Servel refuses transparency requests for individual data per a news report.
- Role: INPUT
- Use in the simulation: INPUT, if found. Adult population per comuna, to check the age split of the INE projection.

### DEIS-MINSAL: Nacidos vivos por comuna de residencia de la madre (2010)
- Link: https://deis.minsal.cl/vitales/vitales2010/NacidosVivos_EdadMadre.htm (dataset title on datos.gob.cl, Servicio de Salud Talcahuano)
- Contents: births by region and comuna of residence of the mother, by age of mother. Only the 2010 table is named.
- Granularity: comuna (of residence).
- Year: 2010.
- Access: web page (the link returned 404).
- Check: UNVERIFIED. The link returns HTTP 404 (80,209 bytes, "Página no encontrada"). The DEIS homepage (https://deis.minsal.cl/) returns HTTP 200, but no download section was found. Dataset title confirmed on datos.gob.cl only.
- Limits: 2010 is old. The link is dead. Births are by residence, not where people live now.
- Role: INPUT
- Use in the simulation: INPUT for fertility and family growth per comuna. Needs a working link and a newer year.

### INE: Anuario estadísticas vitales 2023, nacimientos y defunciones (PDF)
- Link: https://www.ine.gob.cl/docs/default-source/nacimientos-matrimonios-y-defunciones/publicaciones-y-anuarios/anuarios-de-estad%C3%ADsticas-vitales/anuario-de-estad%C3%ADsticas-vitales-2023-nacimientos-y-defunciones.pdf
- Contents: yearly annual report on births and deaths. Likely national and regional tables; comuna tables not checked.
- Granularity: region (expected, not checked).
- Year: 2023.
- Access: PDF.
- Check: UNVERIFIED (HEAD only: HTTP 200, Content-Length 6,289,322. Body not read.)
- Limits: PDF. Not data rows.
- Role: INPUT
- Use in the simulation: INPUT for births and deaths, if the comuna tables exist. Otherwise NONE.

### INE: Estadísticas vitales, cifras provisionales 2025 (press)
- Link: https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2026/05/27/estad%C3%ADsticas-vitales-cifras-provisionales-2025-confirman-que-los-nacimientos-en-chile-disminuyeron-un-46-9-en-32-a%C3%B1os
- Contents: national births fall, 2025 provisional. Press release.
- Granularity: national.
- Year: 2025.
- Access: web page.
- Check: UNVERIFIED. Press text from search results only.
- Limits: National only.
- Role: NONE
- Use in the simulation: NONE for the comuna model. Use as a national check for the birth rate.

### Registro Civil: portal de datos (births, deaths, names)
- Link: https://www.chilevision.cl/noticias/te-ayuda/portal-de-datos-asi-funciona-la-plataforma-donde-el-registro-civil-comparte-datos-de-nombres-muertes-y-nacimientos/amp/ (news about the portal, the portal itself not found)
- Contents: per news, Registro Civil shares births, deaths, and names data through a portal.
- Granularity: unknown.
- Year: unknown.
- Access: unknown.
- Check: UNVERIFIED. News article only. Portal not opened.
- Limits: Names data and individual events may be personal. Do not use microdata.
- Role: NONE
- Use in the simulation: NONE until the aggregate tables are found.

### Driver-licence statistics at national level (MTT, Registro Civil, INE)
- Link: none verified.
- Contents: no national or comuna licence count was found.
- Granularity: none.
- Year: none.
- Access: none found.
- Check: UNVERIFIED. Search and datos.gob.cl did not return a national or Santiago series.
- Limits: The only licence file found is one municipality (see the La Serena entry above). It has personal data.
- Role: NONE
- Use in the simulation: NONE, for now.

## Negative results

- Subtrans: no open data on vehicle permits found. Subtrans covers public transport only.
- SERMIG RD file: no comuna column. Region only.
- Servel: no Metropolitan comuna padrón table found. The file we opened is Valparaíso.
- DEIS: the 2010 link returns 404. No working download found.

## Critique

1. The INE projection file is verified as a download, but the rows are not counted, and the age split is not checked. The base (2017) may not match the census of 2024. It is also not clear whether the file has single-year ages. A simulation that uses it as the control total could be off by a version.
2. Most vehicle, electoral, and birth sources are still UNVERIFIED. The INE permits page loads its file list from a POST endpoint that needs folder IDs. I did not find the IDs. The DEIS link is dead. The Servel comuna table was not found. Without these, the vehicle, voter, and birth inputs are open.
3. The SERMIG tables count permits, not residents, and have no comuna. The migrant share per comuna still has no verified source. The minutas PDFs look useful, but I did not open them.

What I did not check: the full rows of any file except the SERMIG RD acogidas (rows counted from sheet range, not parsed), the INE projection rows (sheet read as header only), the Anuario 2023 body, the other SERMIG files, the Servel TOTEM-2 beyond pages 1 and 2, and the Registro Civil portal.
