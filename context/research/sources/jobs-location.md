# Jobs by place in Santiago

## Best public sources

### SII company worker records, 2020–2024
- Link: [SII company and address file index](https://www.sii.cl/sobre_el_sii/nominapersonasjuridicas.html), [worker records ZIP](https://www.sii.cl/estadisticas/nominas/PUB_EMPRESAS_PJ_2020_A_2024.zip), [registered addresses ZIP](https://www.sii.cl/estadisticas/nominas/PUB_NOM_DIRECCIONES.zip)
- Contents: Join the legal-entity worker record to its registered address by RUT. The worker file gives worker counts or bands, region, activity, and sales band. The address file lists registered head offices and branches, including inactive addresses.
- Granularity: Company address point after geocoding; no coordinates. Worker counts follow the SII domicile or head office. They do not show where each worker performs the job.
- Year: Worker records cover commercial years 2020–2024 and were updated in November 2025. Address records include historical and current addresses; the latest directory was updated in August 2026.
- Access: Direct download; two ZIP files with TXT data. The current worker ZIP is 187,529,334 bytes. The address ZIP is 95,040,204 bytes.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206, 1,024 bytes from each ZIP. Both start with a ZIP signature. I did not decode their TXT rows.
- Role: INPUT
- Script: `tools/scrape/sii_job_locations.py` downloads all three SII files to `data/raw/sii-job-locations/`. I did not run it because the two ZIP files total about 283 MB.
- Limits: Counts follow company headquarters, not work sites. One company can have many branches. Address geocoding is required. The public worker file covers legal entities, not every employer type.
- Use in the simulation: Seed firm size and activity at head-office locations. Use branch addresses to improve work-site placement only when another source provides branch employment.

### SII workers by commune
- Link: [SII company statistics](https://www.sii.cl/sobre_el_sii/estadisticas_de_empresas.html), [open worker statistics file](https://www.sii.cl/sobre_el_sii/empresas/PUB_COMU_TRTRAB.xlsb)
- Contents: Counts of dependent workers reported by employers, with company, activity, worker-band, and commune breakdowns.
- Granularity: Comuna. The SII assigns workers to the employer's registered domicile or head office.
- Year: Commercial years 2005–2025; the page says it was updated in October 2026.
- Access: Direct XLSB download, 4,550,113 bytes.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206 returned a 1,024-byte sample; the full spreadsheet is 4,550,113 bytes. I did not decode workbook rows.
- Role: TARGET
- Limits: This does not locate branches or actual work sites. A worker with more than one employer can count more than once. Small cells can be suppressed for tax secrecy.
- Use in the simulation: Commune-level calibration target for total reported employment, not block allocation.

### INE labour commuting dashboard, Censuses 2002 and 2024
- Link: [INE commuting dashboard](https://conmutacion-laboral.ine.gob.cl/), [INE result note](https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2026/11/16/personas-que-se-trasladan-regularmente-entre-comunas-por-trabajo-aumentaron-77-2-en-dos-d%C3%A9cadas)
- Contents: One row per commune for resident workers and estimated jobs. It also includes selected origin-to-destination flows. The embedded 2024 Santiago data has fields `cut`, `nombre`, `ocupados_n`, `puestos_n`, `ocupados_p`, and `puestos_p`.
- Granularity: Comuna. The dashboard defines jobs as resident workers who do not commute, plus incoming workers.
- Year: Census snapshots 2002 and 2024.
- Access: Public HTML page with embedded JSON. `tools/scrape/census_commute_ine.py` saves the 2024 Santiago job totals and selected flows to `data/raw/census-commute-ine/`.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 14,550,297 bytes. Extracted 48 commune job rows and 144 flow rows. Example: Santiago has 433,378 jobs and 277,362 resident workers; its top listed destination from Santiago is Las Condes, with 28,583 commuters.
- Role: TARGET
- Limits: The 144 flows show only three destinations per each of 48 origins. They are not a complete origin-destination matrix. Job totals stop at the commune and are census estimates, not a live count of occupied positions.
- Use in the simulation: Set and score the 2024 job total by commune. Use listed flows as partial constraints only.

### INE Censo 2024 microdata and Redatam
- Link: [Censo 2024 data release](https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2025/12/04/ine-publica-base-de-datos-del-censo-de-poblaci%C3%B3n-y-vivienda-2024), [Redatam variable dictionary](https://redatam.ine.gob.cl/redbin/RpWebStats.exe/Dictionary?BASE=CENSO_2024&ITEM=DICALL&lang=esp), [Censo 2024 manual](https://censo2024.ine.gob.cl/wp-content/uploads/2025/03/Manual_Censal_CPV2024.pdf)
- Contents: Person, household, and dwelling microdata. The dictionary includes `P44_LUG_TRAB` for work commune or country, plus `P44_LUG_TRAB_ESP` for the specified place.
- Granularity: Residence records are released at comuna-area level. The work destination is a commune or a coded response. It is not a work-site point or block.
- Year: 2024.
- Access: Redatam web queries and tabulations. INE says microdata and supporting documents are public. I did not download a full microdata file.
- Check: UNVERIFIED. The 37,770-byte Redatam dictionary has P44 variable definitions, but no person or job records were read.
- Role: INPUT
- Limits: Statistical secrecy removes fine residential detail. Destination codes do not locate a specific work site. Some people report several communes or no known destination.
- Use in the simulation: Estimate current resident-to-work-commune flows and worker profiles. Do not use it to allocate jobs inside a commune.

## Zone-level movement data

### SECTRA EOD Santiago database
- Link: [SECTRA Santiago EOD page](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/), [survey database ZIP](https://drive.google.com/file/d/193675NwL6QNDs6VJFHU0mXiX5-6SdBPJ/view?usp=drive_link), [direct download](https://drive.usercontent.google.com/download?id=193675NwL6QNDs6VJFHU0mXiX5-6SdBPJ&export=download&confirm=t)
- Contents: SECTRA describes the 2012 survey and links a database. Its tables and fields were not verified.
- Granularity: Unknown at file level. The page also links an EOD zone file.
- Year: 2012.
- Access: SECTRA links to Google Drive. Its direct file host has `robots.txt` set to `Disallow: /`.
- Check: UNVERIFIED. I opened the SECTRA page but did not use a file sample after checking the file host rules.
- Role: TARGET
- Limits: The survey is old and records trips, not a direct count of jobs. The schema and expansion weights are unverified.
- Use in the simulation: NONE until SECTRA provides a permitted route and the survey tables are checked.

### SECTRA EOD Santiago zone polygons
- Link: [SECTRA Santiago EOD page](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/), [zone shapefile ZIP](https://drive.google.com/file/d/1SM2C5b3jhbmYCbAkmtg2yWRU2dMQkGRl/view?usp=drive_link), [direct download](https://drive.usercontent.google.com/download?id=1SM2C5b3jhbmYCbAkmtg2yWRU2dMQkGRl&export=download&confirm=t)
- Contents: SECTRA labels the link as the survey zoning file. File fields and geometry were not verified.
- Granularity: EOD zone, as described by SECTRA.
- Year: 2012.
- Access: SECTRA links to Google Drive. Its direct file host has `robots.txt` set to `Disallow: /`.
- Check: UNVERIFIED. I did not use a file sample after checking the file host rules.
- Role: INPUT
- Limits: The file cannot be fetched by this workflow. The geometry and match to current simulation zones are unknown.
- Use in the simulation: NONE until SECTRA provides a permitted download and the file schema is checked.

## Sources that do not locate work sites

### INE Censo 2017 microdata
- Link: [INE microdata release and access note](https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2019/09/16/ine-pone-a-disposici%C3%B3n-la-base-de-microdatos-del-censo-2017)
- Contents: Person, household, and dwelling records. INE says its DVD release includes records to zone and locality in REDATAM and CSV.
- Granularity: Residence zone or locality in the DVD release. The online service covers region, province, and commune.
- Year: 2017.
- Access: Online REDATAM tabulations; request a DVD from an INE office for zone-level records.
- Check: UNVERIFIED. I opened the access note but did not obtain the microdata or confirm a workplace-location variable.
- Role: INPUT
- Limits: The release confirms person records by residence zone, but I did not check its work-status field. It is older than the 2024 census. No work-site location was confirmed.
- Use in the simulation: NONE for job-site counts. A worker field would need review before using it for a 2017 resident baseline.

### INE establishment microdata, ENIA
- Link: [INE establishment tracking dataset page](https://www.ine.gob.cl/inicio/documentos-de-trabajo/documento/base-de-datos-de-seguimiento-de-establecimientos-de-la-encuesta-nacional-industrial-anual-%28enia%29)
- Contents: Manufacturer establishment records from the annual industry survey. The page describes an establishment ID, an employment-size band, and industry codes.
- Granularity: Establishment record. The public description does not name a commune, address, or coordinate field.
- Year: 1995 onward; industry code definitions change in 2013.
- Access: INE page describes the data product. I did not find or download the microdata file in this check.
- Check: UNVERIFIED. The product page opened, but I did not retrieve data rows or a data file.
- Role: TARGET
- Limits: It covers manufacturing only. A stable establishment ID does not locate the establishment.
- Use in the simulation: NONE until a public geographic field is confirmed.

## Critique

- SII worker totals follow headquarters. They can place a multi-site employer's workforce at the wrong address.
- SECTRA's EOD downloads point to a host that disallows automated access. The survey fields, weights, and zone geometry remain unverified.
- I found no open count of workers by current block or work-site point. I did not retrieve records from the 2024 Redatam data, 2017 Census microdata, or ENIA.
