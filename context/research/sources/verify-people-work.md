# Santiago people and work source verification

Evidence below describes the recorded samples, not a complete ingestion pipeline. No new downloads were made for the label correction.

Checked 2026-11-21 against the ten entries in [clever-heron's catalog](../santiago-people-work.md). RM means Región Metropolitana. Survey row counts are unweighted. Use the survey design and weights for estimates.

Download a direct data link with `python3 tools/scrape/verify_people_work.py '<URL>'`. The script saves files in `data/raw/verify-people-work/`. Add `--sample` for the Census person ZIP. If macOS Python lacks the system CA bundle, set `SSL_CERT_FILE=/etc/ssl/cert.pem`.

### INE Census 2024 block and rural entity table

- Link: [INE results](https://censo2024.ine.gob.cl/resultados/), [viewer](https://visualizador.ine.gob.cl/), [national CSV ZIP](https://storage.googleapis.com/bktdescargascenso2024/Datos_agregados/Base_manzana_entidad_CPV24.zip), [RM cartography ZIP](https://storage.googleapis.com/bktdescargascenso2024/Cartografia/GPKG/Cartografia_censo2024_R13.zip), [cartography manual](https://storage.googleapis.com/bktdescargascenso2024/Datos_agregados/Manual_uso_cartografia_CPV2024.pdf).
- Contents: 212 columns. `MANZENT`, `CUT`, `AREA_C`, `n_per`, `n_hog`, age groups, employment, housing, and commute mode counts. For block `13101011001008`, the file reports 188 people and 123 households.
- Granularity: Urban census block, rural village block, and rural census entity. INE groups some of these units for disclosure control.
- Year: 2024.
- Access: Direct ZIP downloads with a semicolon-delimited CSV and a separate GeoPackage.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200, 25,468,009 bytes. Parsed 197,032 rows, including 50,071 urban and 2,575 rural RM rows. The RM cartography returned HTTP 200 and 64,475,234 bytes. Normalize `MANZENT` to an integer ID. Join against both polygon layers. In RM, 50,022 urban blocks, 499 rural village blocks, and 2,072 rural entities match.
- Role: INPUT / TARGET candidate.
- Limits: The sample verifies the publisher's bulk table. Disclosure control suppresses some values. 53 RM rows are comuna containers with no polygon by design. They hold 108,997 people. Keep their counts at comuna level. See section 4.1 of the cartography manual.
- Use in the simulation: INPUT for residential population, dwelling, and household totals. TARGET candidate for published aggregates held out from calibration; suppressed values are unavailable.
- Verdict: OK. Public bulk data exist at the stated resolution.

### INE Census 2024 microdata

- Link: [INE release](https://censo2024.ine.gob.cl/ine-publica-base-de-datos-del-censo-de-poblacion-y-vivienda-2024/), [manual](https://censo2024.ine.gob.cl/wp-content/uploads/2025/12/manual_uso_microdatos_censo2024.pdf), [dwelling ZIP](https://storage.googleapis.com/bktdescargascenso2024/viviendas_censo2024.zip), [household ZIP](https://storage.googleapis.com/bktdescargascenso2024/hogares_censo2024.zip), [person ZIP](https://storage.googleapis.com/bktdescargascenso2024/personas_censo2024.zip).
- Contents: One row per dwelling, household, or person. Join on `id_vivienda` and, where applicable, `id_hogar`. The dwelling and household files include `comuna`, `comuna_bajo_umbral`, `area`, and `tipo_operativo`.
- Granularity: Person, household, or dwelling record. The finest released residential geography is comuna × urban/rural area, subject to the disclosure threshold.
- Year: 2024.
- Access: Direct ZIP downloads. INE offers CSV and Parquet.
- Check: VERIFIED-ROWS, 2026-11-21. The dwelling and household ZIPs returned HTTP 200 with 55,921,195 and 44,678,452 bytes. The person ZIP returned HTTP 206 with 1,048,576 of 355,253,274 bytes. Read rows with 25 dwelling, 18 household, and 63 person columns. The manual lists 7,664,466 dwellings, 6,622,597 households, and 18,480,432 people.
- Role: INPUT.
- Limits: Block membership is unavailable for these individual records. Use the block totals to place synthetic households.
- Use in the simulation: INPUT for joint person and household traits.
- Verdict: OK. Keep the microdata and block totals as separate inputs.

### CASEN 2024

- Link: [CASEN 2024](https://observatorio.ministeriodesarrollosocial.gob.cl/encuesta-casen-2024), [core RData](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/casen_2024.RData), [geography RData](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/casen_2024_provincia_comuna.RData), [use note](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/Nota_uso_bases_de_datos_Casen_2024.pdf).
- Contents: Core fields include `region`, `area`, `expr`, `ytot`, `ytoth`, and `n_ocupados`. The companion has `folio`, `id_persona`, `expp`, `expc`, `provincia`, and `comuna`.
- Granularity: Person records with household variables. The companion adds comuna of residence. The survey design supports national, regional, and urban/rural estimates for its design variable.
- Year: 2024 survey round.
- Access: Direct RData downloads. The publisher also lists Stata and SPSS.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200 for both files, 46,840,844 and 551,986 bytes. Both have 218,367 rows. The core has 877 columns and 40,658 RM rows.
- Role: INPUT / TARGET candidate.
- Limits: Use `folio` and `id_persona` to join. Check estimate precision. The survey design does not support direct comuna estimates.
- Use in the simulation: INPUT for household income and traits. TARGET candidate for regional distributions.
- Verdict: OK. Commune codes exist, with the stated design limit.

### Official 2024 comuna poverty estimates

- Link: [Publisher](https://observatorio.ministeriodesarrollosocial.gob.cl/pobreza-comunal-2024), [income XLSX](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/pobreza-comunal/2024/SAE_ingresos_2024.xlsx), [multidimensional XLSX](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/pobreza-comunal/2024/SAE_multidimensional_2024.xlsx).
- Contents: Comuna code, projected population, estimated poverty count and rate, interval bounds, and estimate method.
- Granularity: Comuna.
- Year: 2024.
- Access: Direct XLSX downloads.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200 for both workbooks, 60,618 and 56,485 bytes. Read Santiago `13101`: 10.17% income poverty and 17.87% multidimensional poverty.
- Role: INPUT.
- Limits: These are small-area estimates. Use the interval bounds. The denominator in both Santiago rows is 547,366 projected people.
- Use in the simulation: INPUT for local poverty calibration.
- Verdict: OK. Both published spreadsheets have comuna rows.

### INE National Employment Survey (ENE)

- Link: [ENE](https://www.ine.gob.cl/estadisticas-por-tema/mercado-laboral/ocupacion-y-desocupacion), [July–September 2026 CSV](https://www.ine.gob.cl/docs/default-source/ocupacion-y-desocupacion/bbdd/2026/csv/ene-2026-08-jas.csv), [codebook](https://www.ine.gob.cl/docs/default-source/ocupacion-y-desocupacion/bbdd/libro-de-codigos/codigos-ene-2020.pdf).
- Contents: Sampled person, `ano_trimestre`, `mes_central`, home comuna `r_p_c`, work location `b18_region` and `b18_codigo`, employment status, formality, and `fact_cal`.
- Granularity: Person record with comuna codes for home and workplace. The survey design supports national and regional estimates.
- Year: July–September 2026 sample. The publisher lists rolling quarters from 2010.
- Access: Direct semicolon-delimited CSV.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200 plus HTTP 206 to resume, 37,971,063 bytes total. Parsed 96,961 rows and 222 columns. Of 18,955 RM residents in the sample, 8,342 have RM home and work comuna codes.
- Role: TARGET candidate / INPUT.
- Limits: The work location question accepts a foreign country code and includes non-response codes. Check precision for estimates of comuna pairs. A work comuna can contain many sites.
- Use in the simulation: TARGET candidate for regional employment. INPUT candidate for comuna commuting patterns.
- Verdict: WRONG by omission. The file also has a work-comuna field. The original brief only reports the residence code.

### INE Supplementary Income Survey (ESI)

- Link: [ESI](https://www.ine.gob.cl/estadisticas-por-tema/mercado-laboral/encuesta-suplementaria-de-ingresos), [2025 person CSV](https://www.ine.gob.cl/docs/default-source/encuesta-suplementaria-de-ingresos/bbdd/csv_esi/2025/esi_2025_personas.csv?sfvrsn=7f734fe_4&download=true).
- Contents: Sampled person, residence `r_p_c`, work `b18_codigo`, labor income `ing_t_p` and `ing_t_t`, and weight `fact_cal_esi`.
- Granularity: Person record with comuna codes. Use weighted income estimates at national or regional level.
- Year: October–December 2025.
- Access: Direct comma-delimited CSV.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200, 84,878,737 bytes. Parsed 93,458 rows, including 17,830 RM residents. In 7,979 rows, both home and work codes identify RM comunas.
- Role: INPUT / TARGET candidate.
- Limits: Restrict wage calculations to the income population in the manual. ESI is a module of ENE. Treat matching samples and dates as overlapping data.
- Use in the simulation: INPUT for labor income distributions. TARGET candidate for regional income estimates.
- Verdict: OK for regional estimates. The CSV also provides home and work comuna fields.

### SII company statistics

- Link: [SII company statistics](https://www.sii.cl/sobre_el_sii/estadisticas_de_empresas.html), [bulk ZIP](https://www.sii.cl/sobre_el_sii/empresas/EMPRESAS.zip).
- Contents: `PUB_COMU.txt` has one row per year and headquarters comuna, with company counts, annual UF sales, dependent workers, net pay, and worked-month weights. Separate files cross comuna with industry or worker bands.
- Granularity: Comuna of the registered headquarters.
- Year: Commercial years 2005–2025 in the sampled table.
- Access: Direct ZIP with tab-separated text.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200, 310,389,406 bytes. Read all 7,287 rows in `PUB_COMU.txt`. Santiago `2025` reports 80,535 firms and 953,163 reported dependent worker relationships.
- Role: TARGET candidate.
- Limits: One worker can appear for multiple employers. Places of work can differ from the registered headquarters. Some cells are suppressed.
- Use in the simulation: TARGET candidate for registered firm totals by headquarters comuna.
- Verdict: OK. The geography does not support a worksite job total.

### SII legal entities and branch addresses

- Link: [SII register](https://www.sii.cl/sobre_el_sii/nominapersonasjuridicas.html), [addresses ZIP](https://www.sii.cl/estadisticas/nominas/PUB_NOM_DIRECCIONES.zip), [2020–2024 entities ZIP](https://www.sii.cl/estadisticas/nominas/PUB_EMPRESAS_PJ_2020_A_2024.zip).
- Contents: Headquarters and branch files contain entity key, `VIGENCIA`, `FECHA`, `TIPO_DIRECCION`, street, number, and comuna. The 2024 entity file includes employer-level work and industry fields.
- Granularity: Registered address for each entity and site. A point requires geocoding. The employer-level count has no branch key.
- Year: Branch record dates range from 1975-09-17 to 2026-08-02. The entity ZIP contains files for 2020–2024.
- Access: Direct ZIP downloads with tab-separated text.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200 for both ZIPs, 95,040,204 and 187,529,334 bytes. Parsed 294,897 branch rows, 266,652 marked current and 28,245 inactive. RM has 111,657 branch rows across both statuses. Read a 22-column entity row for Las Condes from commercial year 2024.
- Role: INPUT.
- Limits: A current registration can still differ from present activity at the address. There are no branch worker counts or coordinates in the branch table.
- Use in the simulation: INPUT for candidate work locations after filtering and geocoding.
- Verdict: OK for branch addresses. The row check does not support assigning workers to individual sites.

### MINEDUC school directory

- Link: [Publisher](https://datosabiertos.mineduc.cl/directorio-de-establecimientos-educacionales/), [2026 RAR](https://datosabiertos.mineduc.cl/wp-content/uploads/2026/11/Directorio-EE-Oficial-2026.rar).
- Contents: School ID `RBD`, comuna, latitude, longitude, operating status `ESTADO_ESTAB`, total enrolment `MAT_TOTAL`, and enrolment by education level.
- Granularity: School establishment point. The codebook calls the coordinates approximate.
- Year: 30 April 2026 snapshot. The publisher lists annual editions from 1992.
- Access: Direct RAR with a semicolon-delimited CSV, a PDF codebook, and an XLSX frequency table. Extract the CSV with `bsdtar`.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200, 908,273 bytes. Parsed 16,831 rows. Of 3,227 operating RM schools, 3,220 had coordinates inside a broad RM bounding box.
- Role: INPUT.
- Limits: Filter `ESTADO_ESTAB=1` for operating schools. The bounding box is a coarse check. Enrolment counts do not measure daily attendance or travel.
- Use in the simulation: INPUT for school destinations and enrolment weights.
- Verdict: WRONG. The directory already includes enrolment totals, and the 2026 archive contains point coordinates.

### MINSAL facility GIS and health-service boundaries

- Link: [Publisher](https://www.minsal.cl/ide-minsal-geoportal-descarga-de-datos/), [catalog](https://www.minsal.cl/ide-minsal-geoportal-catalogo-de-geodatos/), [facility ZIP](https://www.minsal.cl/wp-content/uploads/2026/08/ESTABLECIMIENTOS_12062026_DES.zip), [boundary ZIP](https://www.minsal.cl/wp-content/uploads/2026/08/SERVICIO_SALUD_MINSAL.zip).
- Contents: Facility points with location, identifiers, `COD_REG`, `TIPO`, `NIVEL`, `ESTADO`, and emergency service flags. Service polygons with `COD_DEIS`, `SERVICIO`, and `MACROZONA`.
- Granularity: Facility point and separate health-service area polygon.
- Year: Facility snapshot 12 June 2026. The publisher dates the boundaries 26 January 2026.
- Access: Direct ZIP downloads with ESRI shapefile components.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200 for both ZIPs, 1,068,166 and 72,483,724 bytes. Read 5,295 facility records, including 1,071 with RM code `13`, and 29 boundary records. All sampled facility status values indicate ordinary operation.
- Role: INPUT.
- Limits: The facility projection is WGS84. The boundary file declares SIRGAS-Chile 2002. Read each archive's `.prj` before spatial joins. Neither layer measures visits or capacity.
- Use in the simulation: INPUT for healthcare destinations and service areas.
- Verdict: WRONG for the blanket WGS84 claim. The polygons use a different coordinate reference system.

## Critique

1. The Census join leaves 53 comuna containers outside the block and entity layers. This audit does not allocate their 108,997 people.
2. ENE and ESI show comuna pairs. This audit does not estimate the uncertainty of any weighted flow or check worksite assignment.
3. School coordinates passed a coarse location check. Site geocoding, facility status, and health-service capacity still need work.
