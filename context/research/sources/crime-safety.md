# Crime and safety data for Santiago

Scope: crime data for the Región Metropolitana at the smallest public spatial unit. Date checked: 2026-11-21 (UTC-3).

## Honest verdict: is crime a real driver or decoration?

**Mixed. Comuna scale: real driver. Below comuna: decoration.**

- Comuna scale and above: public, deep, calibratable. CEAD gives casos policiales by comuna, familia, grupo and subgrupo, every year from 2005. ENUSC gives victimization and fear of crime by city. The Observatorio de Homicidios gives victim-level rows with comuna of aggression, hour, weapon and place type. That is enough to feed (a) a victimization term in the residential-choice utility (why upper-income households move east), (b) a time-space activity-aversion term (why Transantiago ridership drops after 21:00 in some comunas), and (c) a hotspot term for the tail (174 narco-controlled zones concentrate homicides).
- Below the comuna: the only sub-comuna source is STOP Carabineros, which has cuadrantes in a Vue dashboard with Google Maps tiles and no public API or rows. No cuadrante shapefile, no incident points. OpenStreetMap has police stations but Overpass public endpoints were timing out at test time. For a cuadrante-scale sim of La Pintana or Puente Alto, crime becomes decoration unless the team pays for closed data (Paz Ciudadana / BigData, or a Transparencia request to Carabineros).
- One warning: CEAD casos policiales are counts of police-known cases, not incidents. The gap (cifra negra) is what ENUSC measures, and for robo violento ENUSC 2024 finds ~60% of victims do not report. Treat CEAD by comuna as a lower bound, and reweight with the ENUSC comuna denuncia rate before use.

## Sources

### CEAD · casos policiales por unidad territorial (dashboard POST API)
- Link: https://cead.minsegpublica.gob.cl/wp-content/themes/gobcl-wp-master/data/get_estadisticas_delictuales.php
- Contents: one row per unidad territorial (País, Región, Provincia, Comuna) with casos policiales or tasa per 100k, filtered by year, trimester, month, familia, grupo and subgrupo. 7 familias, 22 grupos and 45 subgrupos delictuales from 2024 forward (older series uses DMCS: 2 groups, 13 subgroups).
- Granularity: comuna. Monthly if `mes[]` is set. Series 2005 onward.
- Year: 2005 to 2026 (Jan-Sep 2026 preliminary).
- Access: HTTP POST, form-encoded, returns an HTML `<table>`. No auth, no robots block. See `tools/scrape/cead_comuna_casos.py`.
- Check: VERIFIED-ROWS 2026-11-21. POST with `comuna[]=13101`, `anio[]=2024`, `anio[]=2025`, all 7 familias, HTTP 200, 3,898 B response. Rows read from HTML `<td>`: País 2024=158,213 / 2025=167,303; Región Metropolitana 50,456 / 54,630; Provincia Santiago 37,712 / 39,902; Comuna Santiago 5,170 / 4,950 (sum of 7 familias, casos policiales). Place code: 13101 (Santiago). Column headers: Unidad Territorial + one per year. Downloadable as XLSX with `descarga=true` (not tested).
- Role: INPUT and TARGET.
- Limits: counts, not incidents. Only police-known cases; cifra negra is ~60% for robo con violencia (ENUSC 2024). Classification changed in 2024 (7 familias replaced DMCS); older-series back-compat is partial. Dashboard is a WordPress endpoint, no versioning. No data below comuna.
- Use in the simulation: INPUT for the comuna victimization pressure that enters the residential-choice utility and the night-travel aversion. TARGET for matching monthly comuna casos of the top 5 subgrupos (robo con violencia, robo en lugar habitado, lesiones, homicidio, VIF).

### CEAD · Minuta nacional de casos policiales (PDF cuarto trimestre acumulado)
- Link: https://cead.minsegpublica.gob.cl/wp-content/uploads/download-manager-files/casos-policiales-ene-sep-2026.pdf
- Contents: PDF minuta with tables of casos policiales por familia, trimester, grupo etario, and region. Comparables Chile vs regiones.
- Granularity: region (not comuna). Monthly.
- Year: 2026 ene-sep (current). Prior PDFs cover same window each year.
- Access: direct GET, no auth.
- Check: VERIFIED-BYTES 2026-11-21. HTTP 200, 2,479,986 B, PDF 1.6. Server: nginx. Did not read tables.
- Role: NONE (narrative only).
- Limits: region-only, no sub-comuna.
- Use in the simulation: NONE at comuna scale (worse granularity than the API above). Keep for narrative in the pitch.

### Observatorio de Homicidios · Base de datos de víctimas (XLSX, individual records)
- Link: https://prevenciondehomicidios.cl/wp-content/uploads/2026/09/Base_de_Datos_VHC_2018_PS2026_VF.xlsx
- Contents: one row per victim of homicidio consumado, with SEXO, EDAD_RECOD, CONDENAS (prior convictions), NACION_RECOD (nationality), LUG_AGR_RECOD (place of aggression), ARMA_RECOD (weapon), CONTEXTO_RECOD, COMUN_AGR (comuna of aggression), NOM_REG, DIA, MES2, ID_ANO, HORA_RECOD (6h bin).
- Granularity: comuna of aggression + date + 6-hour time bin + place type (vía pública, domicilio particular, etc.).
- Year: 2018 to primer semestre 2026.
- Access: direct GET, no auth.
- Check: VERIFIED-ROWS 2026-11-21. HTTP 200, 583,002 B, valid XLSX (OOXML zip). sheet1.xml has 9,142 `<row>` entries. Columns read from `sharedStrings.xml`: SEXO, EDAD_RECOD, CONDENAS, NACION_RECOD, LUG_AGR_RECOD, ARMA_RECOD, CONTEXTO_RECOD, COMUN_AGR, NOM_REG, DIA, MES2, ID_ANO, HORA_RECOD. Value examples: comuna Recoleta, región Metropolitana de Santiago, hour bin 00:00-05:59, weapon Arma de Fuego, context "asociado a delito y/o grupo organizado". Years 2018 to 2026 (primer semestre) implied by file name and row count.
- Role: TARGET.
- Limits: inter-institutional count (validated Carabineros + PDI + Fiscalía + SML + Interior), so it lags CEAD. Place of aggression is one of ~5 broad categories, not an address. Comuna is the only geo field.
- Use in the simulation: TARGET. Comuna × month × place-type counts are a strong tail calibration target for the "fear zone" term and for scenarios like "remove the MUT and put a vertical ghetto there", where night-time violence in the receiving comuna matters.

### Observatorio de Homicidios · informes anuales (PDFs)
- Link: https://prevenciondehomicidios.cl/informe-de-homicidios-consumados/
- Contents: Informe Nacional 2018–2022, 2023, 2024 and primer semestre 2026 (PDFs) plus Informe Caracteristicas de Victimarios de Femicidio 2026.
- Granularity: region. Comuna only for highest-rate comunas (top-10 tables).
- Year: 2018 to 2026 S1.
- Access: direct GET.
- Check: VERIFIED-BYTES 2026-11-21 (link list retrieved via the parent page, 7 files; the XLSX above was downloaded, the 6 PDFs were not).
- Role: NONE (duplicates XLSX above).
- Limits: narrative text, tables only for top comunas. The XLSX above subsumes it.
- Use in the simulation: NONE as a feed (use the XLSX). Keep for pitch quotes.

### ENUSC · Encuesta Nacional Urbana de Seguridad Ciudadana (INE + SPD)
- Link: https://www.ine.gob.cl/estadisticas/sociales/seguridad-publica-y-justicia/enusc
- Contents: household and person records for 92 comunas (urban, >50k pop) with victimization by delito, denuncia, fear of crime, incivilidades, confidence in police. Expansion factors for comuna and Gran Santiago.
- Granularity: comuna of residence (92 comunas in sample). Annual. Representative for Gran Santiago.
- Year: 2003 to 2024 (ENUSC 2025 published 2026-07-10, microdata usually released with the Informe de Calidad about 60 days later).
- Access: results and PDFs direct on INE. Microdata file location not found on INE site at check time; prior editions circulate via Subsecretaría de Prevención del Delito and third-party mirrors (CDE UNODC Mexico).
- Check: UNVERIFIED 2026-11-21. HTTP 200 on results page (120,624 B) and bulletins. Did not get a .sav/.csv/.xlsx of microdata; `bbdd`, `bases-de-datos`, `microdatos` paths all 404. Press release confirms ENUSC 2025 7.9% hogar-víctima nacional.
- Role: INPUT if microdata is retrieved.
- Limits: urban only (misses rural comunas Lampa, Buin south, etc.). Sample size at comuna level gives wide CIs for small delitos. 2020 fue remoto por COVID.
- Use in the simulation: INPUT if the team can retrieve the microdata (write to SPD or INE SIAC). Feeds the fear-of-crime attribute of people-agents and the denuncia rate used to reweight CEAD counts.

### Fiscalía · Boletín Estadístico semestral y anual (PDF)
- Link: https://www.fiscaliadechile.cl/sites/default/files/documentos/Boletin_enero_junio_2025.pdf and https://www.fiscaliadechile.cl/sites/default/files/documentos/Bolet%C3%ADn_Anual_2024_0.pdf
- Contents: delitos ingresados (not police-known), términos aplicados, duración, flagrancia. Breakdown by region and categoría delictual; annual by comuna in appendix.
- Granularity: region (main tables). Comuna in appendix tables of the annual boletín.
- Year: 2012 to 2025 S1 (semestrales) and anuales up to 2024.
- Access: direct GET PDF.
- Check: UNVERIFIED 2026-11-21 (link alive via WebSearch result; PDF not opened; Notas Metodológicas 2026-02 mention a DESCARGA DE DATOS section but not tested).
- Role: NONE.
- Limits: PDF only. Covers delitos ingresados al Ministerio Público (includes all delitos; different universe than CEAD casos policiales).
- Use in the simulation: NONE at mass level (overlaps CEAD and worse for geography). Keep as a side source for cifra negra comparison.

### Fiscalía · Estadísticas Interactivas (Power BI)
- Link: https://www.fiscaliadechile.cl/estadisticas-interactivas/
- Contents: daily-updated Power BI dashboard on delitos ingresados, SAF data.
- Granularity: region, categoría, year. The dashboard shows a "DESCARGA DE DATOS" section per its notas metodológicas (not verified to download tabular data headlessly).
- Year: daily updates, series back to 2006.
- Access: embedded Power BI view, no documented API.
- Check: UNVERIFIED 2026-11-21. Page HTTP 200, 26,028 B; iframe to `app.powerbi.com/view?r=...`. Power BI `view` URLs do not expose a public dataset endpoint.
- Role: NONE.
- Limits: scraping requires a headless browser and Power BI session, out of scope for this hackathon.
- Use in the simulation: NONE.

### STOP · Sistema Táctico de Operación Policial (Carabineros)
- Link: https://stop.carabineros.cl/
- Contents: public map of denuncias y detenciones de delitos de mayor connotación social por comuna y cuadrante. Interactive heat map and bar charts.
- Granularity: cuadrante (sub-comuna) on the map; comuna in the bars.
- Year: rolling 24 months.
- Access: Vue SPA with Google Maps; served via Laravel. No documented REST API. JS bundle `app-DYhmfL-K.js` is minified; endpoints under the SPA router, no `/api/*` discovered. Mobile blocked.
- Check: UNVERIFIED 2026-11-21. HTTP 200 on root, but all rows are rendered client-side after a login-free but JS-required flow. `/api/v1/consulta-general`, `/api/comunas`, `/api/zonas` all 404.
- Role: NONE.
- Limits: no bulk download, no shapefile of cuadrantes. Terms of use restrict scraping.
- Use in the simulation: NONE as ingestion (but a great pitch screenshot for the cuadrante heat map).

### Paz Ciudadana · Índice (series históricas por comuna, PDF)
- Link: https://pazciudadana.cl/download/5886/
- Contents: historical series of the Índice Paz Ciudadana – Adimark/GfK by the 41 comunas of the Gran Santiago sample, plus national bulletin PDFs 2010–2026.
- Granularity: comuna (41 of Gran Santiago). Semestral.
- Year: 2000 to 2013 for the comunal series PDF; national index continues through 2026-10.
- Access: direct GET.
- Check: VERIFIED-BYTES 2026-11-21. HTTP 200, 867,225 B, valid PDF 1.6. Downloaded under session cookie; no auth. Did not read the tables.
- Role: TARGET.
- Limits: the comunal series freezes at 2013 in the public PDF. 2014 onward is in annual slide decks with aggregate figures only.
- Use in the simulation: TARGET for the fear-of-crime term by comuna until 2013 (long series lets us calibrate the fear→migration link across a decade). Pair with ENUSC for current periods.

### datos.gob.cl · Carabineros DMCS + VIF + Ley de Drogas por comuna 2010-2012 (XLSX)
- Link: https://datos.gob.cl/uploads/recursos/Estadistico%20Delitos%20de%20Mayor%20Connotacion%20Social%20-%20VIF%20-%20Ley%20de%20Drogas.xlsx
- Contents: comuna × month × DMCS group (hurto, robo, homicidio, lesiones, violación, VIF, drogas) counts.
- Granularity: comuna, monthly.
- Year: 2010, 2011, 2012.
- Access: direct GET.
- Check: VERIFIED-BYTES 2026-11-21. HTTP 200, 3,647,360 B, content-type XLSX. Did not open the sheets.
- Role: INPUT (historical back-series only).
- Limits: ends in 2012. CEAD above already supersedes it. Useful only as a long back-series for testing model stability (did our sim reproduce Santiago-centro 2010 patterns?).
- Use in the simulation: NONE for forward runs; INPUT only for historical calibration back-series.

### DEIS · Defunciones por causa (CIE-10 X85–Y09 = homicidio)
- Link: https://deis.minsal.cl/#datosabiertos (hub); current zip under https://repositoriodeis.minsal.cl/DatosAbiertos/VITALES/
- Contents: all deaths with CIE-10 cause, comuna of residence and of occurrence, sex, age, date. Agresiones (X85–Y09) are the vital-statistics definition of homicide.
- Granularity: comuna. Daily date.
- Year: 1990 to ~2023 cifras oficiales; 2024-2025 cifras preliminares weekly.
- Access: direct GET from `repositoriodeis.minsal.cl`, but the folder index is 403 and the historical URL patterns I tried all 404 at check time.
- Check: UNVERIFIED 2026-11-21. DEIS root HTTP 200 (278,899 B), repository root 403, four guessed file names 404. Prior CPLT ruling confirms the path pattern exists.
- Role: INPUT if downloaded.
- Limits: weekly extracts rename their file. The public interface is SAS Visual Analytics (`informesdeis.minsal.cl`) which does not export rows. A Transparencia request may be the fastest way.
- Use in the simulation: INPUT if downloaded. Independent cross-check against the Observatorio xlsx (same homicides, different system, usually ~10% gap).

### SINIM · Datos municipales (seguridad: guardias, cámaras, casetas, programas)
- Link: https://sinim.gov.cl/
- Contents: per-comuna indicators including security program coverage, number of cameras, guards, SOS buttons.
- Granularity: comuna. Annual.
- Year: 2012–2024.
- Access: web portal. Direct HTTP returned 403 from our IP at check time.
- Check: UNVERIFIED 2026-11-21. HTTP 403, 239 B.
- Role: INPUT if retrieved.
- Limits: self-reported by municipios; counts of cameras are known to disagree with municipal press releases (Santiago reports "472 operativas" in Subdere and "800+ operativas" in a 2025 Transparencia answer). Treat as an order of magnitude.
- Use in the simulation: INPUT for a comuna-level surveillance intensity feature in the perceived-safety term, with a ±2× uncertainty band.

### SernamEG / Ministerio de la Mujer · Femicidios consumados y frustrados
- Link: https://www.minmujeryeg.gob.cl/ (femicidio page)
- Contents: list of femicide cases with comuna, date, age, nationality, context.
- Granularity: comuna. Case-level.
- Year: 2010 onward.
- Access: no documented bulk download. Published as PDFs and press releases; Red Chilena contra la Violencia maintains an independent monthly count that disagrees by ~15–30%.
- Check: UNVERIFIED 2026-11-21. Not retrieved as rows.
- Role: NONE (covered by Observatorio XLSX).
- Limits: two parallel counts (SernamEG vs. Red Chilena) differ because of definitional scope (only íntimo for SernamEG historically). The Observatorio xlsx above covers the subset that reaches homicide prosecution.
- Use in the simulation: NONE beyond the Observatorio xlsx, unless gender-disaggregated residential risk is a demo moment.

### BCN Mapoteca · planos urbanos de cabeceras comunales (shapefile)
- Link: https://www.bcn.cl/siit/mapas_vectoriales/index_html
- Contents: vector layers for each cabecera comunal including points for comisarías de Carabineros, bomberos, salud, educación, cementerios.
- Granularity: point.
- Year: layers dated 2018–2021 depending on comuna.
- Access: direct GET (ZIP per comuna).
- Check: UNVERIFIED 2026-11-21. BCN root HTTP 200, catalog listed; did not download a specific Santiago ZIP to confirm it contains comisaría points.
- Role: INPUT if downloaded.
- Limits: varies by comuna; some RM comunas lack a recent layer. Points are static locations, not response-time polygons.
- Use in the simulation: INPUT for the distance-to-police feature inside the perceived-safety term, once sampled.

### OpenStreetMap · Overpass (comisarías, amenity=police)
- Link: https://overpass-api.de/api/interpreter (API) and https://www.openstreetmap.org/
- Contents: nodes/ways tagged `amenity=police` with name, operator, address.
- Granularity: point.
- Year: continuously updated, 2004–present.
- Access: Overpass QL query. Mirror `overpass.kumi.systems` as a backup.
- Check: UNVERIFIED 2026-11-21. Both public Overpass endpoints timed out from the current network during the session; retry from CI or from a different IP.
- Role: INPUT when retrieved.
- Limits: completeness varies. Carabineros vs. PDI vs. municipal guard posts are mixed under `amenity=police`; filter by `operator=Carabineros de Chile`.
- Use in the simulation: INPUT, as the sub-comuna substitute for the BCN shapefile. Combine both.

### CIPER · Zonas Ocupadas por el narcotráfico en la RM (174 polígonos, 2021)
- Link: https://www.ciperchile.cl/2021/04/20/estas-son-las-174-zonas-ocupadas-por-el-narco-en-la-region-metropolitana/
- Contents: 174 named zones (poblaciones, villas) under narco control in the RM, with population per zone and access-to-services profile.
- Granularity: polygon per población (sub-comuna). 1,012,000 people live in the zones.
- Year: 2021 (prior editions 2009, 2012).
- Access: interactive web map, no documented data download. Pablo Arriagada analyzed the dataset.
- Check: UNVERIFIED 2026-11-21. Only article text; no shapefile. The team can digitize from the map or request from CIPER.
- Role: INPUT if digitized.
- Limits: journalistic definition (≥2 of 5 criteria), not an official geography. 2021 snapshot; the border has shifted after Covid and the rise of Tren de Aragua (post-2022).
- Use in the simulation: INPUT if digitized; it is the one sub-comuna geography that materially changes agent behaviour (routing avoidance, residential rejection, wage discount for workers from those zones).

## Critique

Three weakest points of this catalog, and what I did not check:

1. **Nothing verified below the comuna.** STOP and CIPER both operate below the comuna, but neither releases rows. The catalog says "verdict: comuna real driver, below-comuna decoration", and that honesty line is doing the heavy lifting; without a sub-comuna source we are hand-waving at the 174-zone scale. I did not try a Transparencia request to Carabineros for the cuadrante shapefile and the per-cuadrante denuncias table.
2. **CEAD was tested with one comuna and two years.** The POST payload worked for comuna 13101 and Jan 2024 / Jan 2025 totals of 7 familias, but I did not run it across all 52 RM comunas, nor did I test month-level POSTs, nor the `descarga=true` XLSX branch. There may be rate-limit or field-encoding surprises at scale.
3. **ENUSC microdata is a hole.** I could not find a direct `.sav` or `.csv` link on ine.gob.cl, and I did not write to SIAC. All the "fear of crime by comuna" claims above assume we will get it; if we cannot, the residential-sorting mechanism loses its calibrator.

What I did not check:
- Overpass police stations (public endpoint timing out on this network).
- BCN Mapoteca per-comuna ZIP contents (only the index page).
- DEIS defunciones zip (current file name unknown; folder 403).
- Fiscalía Power BI backing API.
- Any Transparencia request to Carabineros, SPD or SernamEG.
