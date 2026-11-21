# Construction pipeline: permits, new projects, zoning caps, and who moves in

Topic: new buildings and land change. These sources answer the demo question
"Remove a mall, put a tall residential tower there. What happens?" in 3 chains:

- **Capacity (real):** how many dwellings a tower of N floors holds (permit records).
- **Capacity (legal cap):** point -> zone polygon -> max height and density the plan allows.
- **Who moves in:** permit points joined to Census manzana people and dwellings.

Owner: clever-puma (claude-opus-4-8). I do not repeat clever-heron (housing stock,
prices) or clever-llama (building geometry, SII catastro). I go deep on permits, projects,
the completion (recepción) step, and the permit->Census join.

Verification labels (swarm rule): `VERIFIED-ROWS` = I read real records (columns, dates, place
codes). `VERIFIED-BYTES` = the page or service answered with real bytes, but I did not read topic
rows. `UNVERIFIED` = not fetched. A query that returns JSON/GeoJSON records from an ArcGIS
FeatureServer counts as ROWS. Each entry has a `Role:` line: INPUT, TARGET, or NONE.

All ArcGIS queries below use the REST query API with no key: append
`/query?where=...&outFields=*&f=json`, or `outStatistics=[...]` for aggregates.

---

### INE building permits, georeferenced — OFFICIAL live service (PE_NACIONAL)
- Link: https://services5.arcgis.com/hUyD8u3TeZLKPe4T/arcgis/rest/services/CRF_PE_v2/FeatureServer/1
- Contents: one row = one building permit, as a point (service owner `gisine1` = INE). Columns:
  COD_REGION, GLOSA_REGION, COD_COMUNA, GLOSA_COMUNA, AÑO, CANTIDAD_UNIDAD (units to build),
  NUM_PISOS (floors), GLOSA_DESTINO (free text, e.g. "Casa aislada"), USO_DESTINO (HABITACIONAL /
  MIXTO / NO HABITACIONAL), SUPERFICIE (m2), PRECISION_ (geocoding quality: ALTA/...).
- Granularity: point. Year field per record (AÑO).
- Year: 2010 to 2025 (min/max of AÑO, confirmed by query).
- Access: ArcGIS REST, no key. 163,487 points nationwide. This is INE's own org and is the
  authoritative source, newer than the UC mirror below.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. National count 163,487, span 2010-2025. Tall
  residential buildings in RM (COD_REGION=13), 2019-2025, NUM_PISOS>=15, USO_DESTINO in
  (HABITACIONAL, MIXTO): 263 permits, avg 341 units, sum 89,686 units, max 32 floors. One
  Estación Central example: a 2018 permit, 24 floors, 432 dwellings.
- Limits: CANTIDAD_UNIDAD is PERMITTED units, not built units (a permit can be dropped; see the
  recepción layer for built). For USO_DESTINO=MIXTO the unit count MIXES dwellings with commercial
  and office units (GLOSA_DESTINO shows "edificio o bloque ... + Locales comerciales + Oficinas"),
  so MIXTO units are an upper bound on dwellings, not a dwelling count. PRECISION_ is not always
  ALTA. INE removed "número de permiso" after 2020-12-30, so no permit ID to dedupe revisions.
- Role: INPUT (also a TARGET for backtesting new housing supply per comuna/year)
- Use in the simulation: INPUT. Core source for "dwellings per tower of N floors" and the spatial
  seed of new housing. Count HABITACIONAL alone for a clean dwelling figure; add MIXTO for the
  upper bound. This is the direct answer to the pitch moment.

### INE building permits — Observatorio de Ciudades UC mirror (2010-2018)
- Link: https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/Permisos_edificacion_2010_2018/FeatureServer/0
- Contents: same permit points, simpler field names: CANTIDAD_U (units), NUM_PISOS, USO_DESTIN,
  GLOSA_DEST, SUPERFICIE, GLOSA_COMU, AÑO.
- Granularity: point. Year: 2010-2018 only.
- Access: ArcGIS REST, no key. 98,825 points nationwide; RM = 25,808 (43 comunas, incl. small
  ones like Pirque and San José de Maipo — NOT limited to comunas above 50,000).
- Check: VERIFIED-ROWS 2026-11-21. RM HABITACIONAL >=15 floors: 361 permits, avg 298 dwellings
  (median 238), max 1,161 dwellings, max 38 floors. MIXTO adds 204 permits / 68,998 units (units
  include commercial, see note above). Estación Central HABITACIONAL >=20 floors: 57 permits,
  avg 477 (the "guetos verticales"). Script `tools/scrape/ine_permits.py` saves all RM points.
- Limits: ends 2018 (use the official PE_NACIONAL layer above for 2019-2025). Same MIXTO caveat.
  Credit: lucky-otter flagged the MIXTO gap and the comuna-coverage error.
- Role: INPUT
- Use in the simulation: INPUT. Fallback with the simplest schema; superseded by PE_NACIONAL for
  current years.

### INE recepción final — completed dwellings (CRF_NACIONAL)
- Link: https://services5.arcgis.com/hUyD8u3TeZLKPe4T/arcgis/rest/services/CRF_PE_v2/FeatureServer/2
- Contents: one row = one certificate of final reception (obra terminada), as a polygon. Columns:
  REGION, NOM_REGION, CUT, NOM_COMUNA, N_VIVIENDA (completed dwellings), AÑO_EMISION. Layer 0 of
  the same service holds CRF individual certificates 2017-2018.
- Granularity: polygon (received work). Year by AÑO_EMISION.
- Access: ArcGIS REST, no key.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. Read fields and a record (AÑO_EMISION 2014,
  N_VIVIENDA 58). The year statistic used field AÑO_EMISION (not AÑO).
- Limits: completion, not permit. A reception lags its permit by 1-4 years, so do not match a
  permit and a reception by year alone. No units/floors breakdown like the permit layer.
- Role: TARGET (dwellings actually BUILT, to check the permit->built drop-off)
- Use in the simulation: TARGET. The built counterpart of the permit layer. Use it to learn what
  fraction of permitted dwellings get built, so the engine does not treat every permit as housing.

### Census manzana (who moves in) — INE Manzana 2017 people and dwellings
- Link: https://services5.arcgis.com/hUyD8u3TeZLKPe4T/arcgis/rest/services/Manzana_2017_2/FeatureServer/0
- Contents: one row = one census manzana polygon. Columns include MANZENT (manzana id), CUT,
  COMUNA, TOTAL_PERSONAS, TOTAL_HOMBRES/MUJERES, age brackets (0-5, 6-14, 15-64, 65+),
  PERSONAS_EMIGRANTES, PUEBLOS_INDIGENA_ORIGINARIO, TOTAL_VIVIENDAS, TOTAL_VIV_PARTICULARES,
  CANTIDAD_HOGARES, dwelling type (VIV_TIPO_CASA/DEPTO/PIEZA...), wall/roof/floor material, water.
- Granularity: manzana (city block). Year: Census 2017.
- Access: ArcGIS REST, no key. Point-intersect a permit coordinate:
  `.../query?geometry=<x>,<y>&geometryType=esriGeometryPoint&inSR=4326&spatialRel=esriSpatialRelIntersects&outFields=*&f=json`.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. The 2018 Estación Central tower above (lon -70.7018,
  lat -33.4603) falls in manzana MANZENT 13106101001026 (Pila del Ganso): TOTAL_PERSONAS 2,156,
  TOTAL_VIVIENDAS 1,019, CANTIDAD_HOGARES 878, VIV_TIPO_DEPTO 904, PERSONAS_EMIGRANTES 218. So the
  block holds ~2.1 persons per dwelling and is mostly apartments and migrants.
- Limits: a manzana mixes the new tower with older housing, so it is a neighbourhood profile, not
  the tower's own residents. Census 2017 predates most towers; use it as the "before" and the 2024
  manzana as the "after". I do NOT re-catalog the Census; clever-heron / quiet-gecko own it
  (quiet-gecko verified Census 2024 Manzanas_CPV24 polygons; 52,593/52,646 RM rows join).
- Role: INPUT (person-per-dwelling multiplier and demographic profile) and TARGET (the 2017->2024
  change in tower manzanas)
- Use in the simulation: INPUT. The "who moves in" chain: a dwelling count is not a people count;
  multiply new dwellings by persons-per-dwelling and the local age/migrant profile. The 2017->2024
  population change in tower manzanas is a backtest TARGET for the engine.

### MINVU PRC vector zones — Estación Central and Cerro Navia (ArcGIS Online)
- Link: https://services3.arcgis.com/cTnMkBRk4HWkUCRo/ArcGIS/rest/services/PRC_Estacion_Central_Cerro_Navia/FeatureServer
- Contents: one row = one zone polygon of the Plan Regulador Comunal. Columns: COMUNA, ZONA,
  DENOMINACI, Ocup_suelo (ground coverage), Alt_Edif (height), Densidad, Sup_pred_m (min plot m2).
- Granularity: zone polygon (sub-comuna). Year: layer source dated 2019-12-10.
- Access: ArcGIS REST, no key. Layer 0 = Cerro Navia, layer 1 = Estación Central.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. Estación Central = 102 zone polygons.
- Limits: fields are NOT machine-clean: of 102 EC zones, Alt_Edif (height) is blank in 44 and
  Densidad is blank in 83; where present, height uses mixed units ("4 pisos", "8 m", "31.50 mts").
  The legal cap needs manual coding per zone. Only 2 comunas in this service; RM coverage via MINVU
  is partial, not one seamless layer. Credit: lucky-otter.
- Role: INPUT
- Use in the simulation: INPUT. The legal cap chain: point-in-polygon -> max height/coverage ->
  max dwellings a plan allows. Shows WHY towers got so tall in Estación Central (weak old cap).

### MINVU PRMS — Plan Regulador Metropolitano de Santiago (ArcGIS Online)
- Link: https://services3.arcgis.com/cTnMkBRk4HWkUCRo/ArcGIS/rest/services/PRMS_AGOL_2024/FeatureServer
- Contents: 18 layers. Layer 18 "intensidad de uso de suelo": PROVINCIA, COMUNA, NOMBRE, PRMS
  (modification), ORD_1/ORD_2/ORD_3 (ordinance article), M2, Há, URL (link to ordinance PDF).
  Other layers: urban limit, metropolitan road system, natural-risk areas, productive activities.
- Granularity: zone polygon, intercomunal. Year: "AGOL_2024"; records cite PRMS modifications.
- Access: ArcGIS REST, no key.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. 18 layers; layer 18 has 385 rows; read fields and rows.
- Limits: the PRMS sets the intercomunal frame (urban limit, density class), not the parcel-level
  height the PRC sets. Article text is in the linked PDF, not the table.
- Role: INPUT
- Use in the simulation: INPUT. Second half of the legal cap chain, and the urban/rural boundary.

### INE permisos de edificación — official page and comuna economic database
- Link: https://www.ine.gob.cl/estadisticas/economia/edificacion-y-construccion/permisos-de-edificacion
- Contents: the publisher page. "Bases de datos" tab holds the monthly building-permit survey by
  comuna (CANTIDAD_U units, surface by destino). Access format 2002-2018, Excel 2019-2021.
- Granularity: comuna, monthly. The point product is the georeferenced release (PE_NACIONAL above).
- Year: 2002 to present.
- Access: web page; the "Bases de datos" file list loads by script, not a static link.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200 on the page. I did NOT read the Excel rows (the file
  list is script-loaded). NOTE: the comuna Excel is UNVERIFIED, but the georeferenced point data is
  fully live via PE_NACIONAL — the old claim "INE permits download is broken" is only true for the
  retired ArcGIS Hub and the script-loaded Excel, not for the feature service.
- Role: INPUT
- Use in the simulation: INPUT. Comuna monthly totals, if a monthly series is needed below the
  point data. UNVERIFIED for rows.

### MINVU geoide — national PRC/PRI/PRMS MapServer
- Link: https://geoide.minvu.cl/server/rest/services/IPT
- Contents: ArcGIS MapServer services, one per planning instrument. Vectorized zones with land
  use, risk areas, roads. An alternate zoning host to the AGOL services above.
- Granularity: zone polygon. Year: varies by instrument.
- Access: ArcGIS REST, no key. `?f=pjson` per service.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200 on the /IPT folder. I did not pull zone rows here
  (I used the AGOL PRC/PRMS services). verify-housing-land.md opened PRC_Metropolitana here.
- Role: INPUT
- Use in the simulation: INPUT. Backup zoning host when a comuna is missing from AGOL.

### SEIA — large real-estate projects in environmental review
- Link: https://seia.sea.gob.cl/busqueda/buscarProyecto.php
- Contents: one row = one project in environmental review. Real-estate and urbanization projects
  above the DS 40 size thresholds (not checked) must enter. Each has a comuna, a location, a
  developer, m2, and often a dwelling count, plus a dossier with coordinates.
- Granularity: project (point/polygon in the dossier). Year: 2010 to present.
- Access: public web search (POST form). Monthly statistics are PDF. Public document API host
  pertinencia.sea.gob.cl/api/public answered HTTP 200.
- Check: VERIFIED-BYTES 2026-11-21. Search landing HTTP 200; API host HTTP 200. My GET to the
  search action returned 0 bytes (it needs POST). I did NOT pull a project row.
- Limits: ONLY projects above the SEIA threshold enter, so it is biased to big towers and misses
  mid-size permits. Coordinates sit inside PDF dossiers, not a clean column.
- Role: INPUT
- Use in the simulation: INPUT. Forward-looking pipeline of the biggest projects, with location.
  UNVERIFIED for rows.

### DOM monthly permit resolutions (Ley 21.718 / DDU 525) and Transparencia Activa
- Link: https://www.minvu.gob.cl/elementos-tecnicos/circulares-division-de-desarrollo-urbano-ddu/permisos-aprobaciones-y-recepciones-edificacion/
- Contents: Ley 21.718 and DDU 525 (2025) require each municipality to publish a monthly resolution
  that lists the permits the DOM granted the month before. Per-permit, with address.
- Granularity: permit (address), monthly, per comuna. Year: 2025 onward.
- Access: each municipal site / Transparencia Activa, mostly PDF. No single national feed.
- Check: UNVERIFIED 2026-11-21. I read the DDU rule, not a published municipal resolution.
- Limits: fragmented across ~52 RM comunas, PDF, new and uneven.
- Role: INPUT
- Use in the simulation: INPUT. The only near-real-time, per-permit, post-2024 source. Lead, not ready.

### DOM en Línea (MINVU national permit platform)
- Link: https://www.minvu.gob.cl/dom-en-linea/
- Contents: national platform to file DOM permits online. Holds the live permit records.
- Access: needs ClaveÚnica login. Logging in is forbidden by our rules and gets personal data.
- Check: UNVERIFIED (not attempted; login barrier).
- Role: NONE
- Use in the simulation: NONE for us. Login wall.

### Observatorio del Mercado de Suelo (MINVU)
- Link: https://www.minvu.gob.cl/observatorios-del-mercado-de-suelo-urbano/
- Contents: meant to show land value and the tramitación status of permits per comuna.
- Check: UNVERIFIED 2026-11-21. The permits section shows "Próximamente" (not yet live).
- Role: NONE
- Use in the simulation: NONE now. Revisit.

### SII Detalle Catastral (property register: year built, use, m2)
- Link: https://www4.sii.cl/mapasui/internet/#/contenido/index.html
- Contents: per-property year built, destino (use), and built m2.
- Check: UNVERIFIED. Needs SII login; no geometry. Owned by clever-llama (building geometry).
- Role: NONE
- Use in the simulation: NONE here (cross-reference clever-llama). Login wall.

### normativa MCP tools (query_prc / query_prms / OGUC / LGUC)
- Link: local MCP server `normativa` (tools query_prc, query_prms, get_oguc_article, get_lguc_article).
- Contents: semantic search over PRC, PRMS, OGUC, LGUC text; could return density/height rules.
- Check: UNVERIFIED 2026-11-21. query_prc failed: "Could not connect to a Chroma server" (backend
  down). The get_*_article tools may fetch single articles by ID; not tested.
- Role: INPUT (potential)
- Use in the simulation: INPUT (potential). Pairs with the PRC polygons: polygon gives the zone,
  these tools give the written rule. Needs the Chroma backend up.

### CChC and private real-estate consultancies (Colliers, GPS, Toctoc projects)
- Link: https://www.cchc.cl/
- Contents: new-project pipelines, sales speed, stock of new dwellings by comuna.
- Check: UNVERIFIED. Behind paid reports or membership; no open bulk data found.
- Role: NONE
- Use in the simulation: NONE (open). Paid fallback if a number is missing.

---

## Critique

My own 3 weakest points, and what I did not check:

1. **The legal-cap layer is not machine-clean.** Zoning rows exist for Estación Central, Cerro
   Navia, and the PRMS intensity layer, but of 102 EC zones height is blank in 44 and density in
   83, with mixed units. I did not build a point-in-polygon lookup, and most RM comunas' PRC
   services are not confirmed. The legal-cap chain is shown feasible, not built.
2. **MIXTO unit counts are dirty.** For MIXTO permits CANTIDAD_UNIDAD mixes dwellings with
   commercial and office units, so the "+89,686 tall-building units" figure is an upper bound on
   dwellings, not a dwelling count. A clean dwelling count needs either HABITACIONAL only or a
   split of the MIXTO GLOSA_DESTINO text, which I did not do.
3. **The permit->reception->resident chain is checked only at single points.** I confirmed the
   permit layer, the reception layer, and one permit->manzana join each with one or a few records.
   I did not measure the permit-to-built drop-off rate, the 2017->2024 manzana change, or match a
   reception to its permit. The three chains are proven to connect, not calibrated.

Not checked: the comuna monthly Excel rows; whether CANTIDAD_UNIDAD double-counts permit revisions;
geoide.minvu.cl RM comuna zone rows; SEIA project rows (needs POST); the DOM Ley 21.718 monthly
resolutions.
