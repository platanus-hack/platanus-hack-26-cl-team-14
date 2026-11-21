# Construction pipeline: permits, new projects, and zoning caps

Topic: new buildings and land change. These sources answer the demo question
"Remove a mall, put a tall residential tower there. What happens?" in 3 chains:

- **Capacity (real):** how many dwellings a tower of N floors holds (permit records).
- **Capacity (legal cap):** point -> zone polygon -> max height and density the plan allows.
- **Who moves in:** permit points joined to Census manzana people and dwellings.

Owner: clever-puma (claude-opus-4-8). I do not repeat clever-heron (housing stock,
prices) or clever-llama (building geometry, SII catastro). I go deep on permits and projects.

Verification labels (swarm rule): `VERIFIED-ROWS` = I read real records (columns, dates, place
codes). `VERIFIED-BYTES` = the page or service answered with real bytes, but I did not read topic
rows. `UNVERIFIED` = not fetched. A query that returns JSON/GeoJSON records from an ArcGIS
FeatureServer counts as ROWS. Each entry also has a `Role:` line: INPUT, TARGET, or NONE.

---

### INE building permits, georeferenced (Observatorio de Ciudades UC mirror)
- Link: https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/Permisos_edificacion_2010_2018/FeatureServer/0
- Contents: one row = one building permit, as a point. Columns: COD_REGION, GLOSA_REGI,
  COD_COMUNA, GLOSA_COMU, AÑO, CANTIDAD_U (dwellings/units to build), GLOSA_DEST (e.g.
  "Casa aislada"), USO_DESTIN (HABITACIONAL / non-residential), NUM_PISOS (floors), SUPERFICIE (m2).
- Granularity: point. One step per permit. Year field per record.
- Year: 2010 to 2018 (min/max of AÑO field, confirmed by query).
- Access: ArcGIS REST query API, no key, JSON/GeoJSON. 98,825 points nationwide.
  Query with `where`, `outFields`, `outStatistics`. Script: `tools/scrape/ine_permits.py`.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. National count 98,825. RM (COD_REGION=13) = 25,808 points.
  RM residential permits with NUM_PISOS>=15: n=361 towers, avg 298 dwellings (median 238),
  avg 20.4 floors, max 1,161 dwellings, max 38 floors. Estación Central residential >=20 floors: 57 permits,
  avg 477 dwellings each (the "guetos verticales").
- Limits: this is a UC mirror, not INE's live service. Ends 2018 (no 2019+ here). "Obra nueva"
  and "regularización de obra nueva" only, no extensions ("ampliaciones"). Covers comunas above
  ~50,000 people. CANTIDAD_U is permitted units, not built units; a permit can be dropped. INE
  removed "número de permiso" from its files after 2020-12-30, so no permit ID to dedupe.
- Role: INPUT
- Use in the simulation: INPUT. Core source for "dwellings per tower of N floors" and for the
  spatial seed of new housing. The RM 15+ floor stat above is the direct answer to the pitch moment.

### MINVU PRC vector zones — Estación Central and Cerro Navia (ArcGIS Online)
- Link: https://services3.arcgis.com/cTnMkBRk4HWkUCRo/ArcGIS/rest/services/PRC_Estacion_Central_Cerro_Navia/FeatureServer
- Contents: one row = one zone polygon of the Plan Regulador Comunal. Columns: COMUNA, ZONA,
  DENOMINACI, Ocup_suelo (ground coverage, e.g. "1 Piso: 70% 2-4 Pisos: 50%"), Alt_Edif
  (height, e.g. "4 pisos"), Densidad, Sup_pred_m (minimum plot m2), Superf_ha.
- Granularity: zone polygon (sub-comuna). No time step; one plan version.
- Year: layer source dated 2019-12-10 (field "Fuente"). Reflects the PRC in force then.
- Access: ArcGIS REST query API, no key. Layer 0 = Cerro Navia, layer 1 = Estación Central.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. Read fields and 3 zone rows for Estación Central,
  including the inherited "ZONA COMUNA DE MAIPÚ" (Alt_Edif "4 pisos").
- Limits: only 2 comunas in this service. "Densidad" field is often blank; height is text
  ("4 pisos"), not a clean number. MINVU hosts separate per-comuna services; coverage of RM is
  partial, not one seamless layer. Estación Central is the key demo comuna, so this is high value.
- Role: INPUT
- Use in the simulation: INPUT. The legal cap chain: point-in-polygon -> max height/coverage ->
  max dwellings a plan allows at that site. Shows WHY towers could be so tall here (weak old cap).

### MINVU PRMS — Plan Regulador Metropolitano de Santiago (ArcGIS Online)
- Link: https://services3.arcgis.com/cTnMkBRk4HWkUCRo/ArcGIS/rest/services/PRMS_AGOL_2024/FeatureServer
- Contents: 18 layers of the metropolitan plan. Layer 18 "a2_titulo_4_intensidadusosu" =
  land-use intensity; columns PROVINCIA, COMUNA, NOMBRE, PRMS (modification number), ORD_1/ORD_2/ORD_3
  (ordinance article, e.g. "Título 4° Intensidad de Utilización del Suelo Metropolitano",
  "Art. 4.3 - 1. Áreas Urbanizadas"), M2, Há, URL (link to the ordinance PDF). Other layers:
  urban limit, metropolitan road system, natural-risk areas, productive activities.
- Granularity: zone polygon, intercomunal (above comuna, below region).
- Year: "AGOL_2024" service; records cite PRMS modifications (e.g. Mod 71/2007, PRC 2010) and the
  2014 ordinance PDF.
- Access: ArcGIS REST query API, no key.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. Listed 18 layers; read fields and 3 rows of layer 18.
- Limits: the PRMS sets the intercomunal frame (urban limit, density class), not the parcel-level
  height the PRC sets. Use it with the PRC. Article text lives in the linked PDF, not in the table.
- Role: INPUT
- Use in the simulation: INPUT. Second half of the legal cap chain, and the urban/rural boundary
  that says where new housing can go at all.

### INE permisos de edificación — official page and economic database
- Link: https://www.ine.gob.cl/estadisticas/economia/edificacion-y-construccion/permisos-de-edificacion
- Contents: the primary publisher. "Bases de datos" tab holds the monthly building-permit survey
  (by comuna: COD_COMUNA, GLOSA_COMU, AÑO, CANTIDAD_U units, surface by destino). Access format
  2002-2018, Excel 2019-2021, newer years to confirm.
- Granularity: comuna, monthly. The georeferenced product (point) is a separate release.
- Year: 2002 to present (survey); georeferenced from 2010.
- Access: web page; the file list loads by script (Sitefinity doc library), not a static link.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200 on the page. I did NOT get the Excel rows: the
  "Bases de datos" file list is script-loaded and I did not reach the file URLs.
- Limits: comuna level only (the point product is the georeferenced release). Provisional figures,
  revised later. No "número de permiso" after 2020-12-30.
- Role: INPUT
- Use in the simulation: INPUT. Primary, current source to extend the 2010-2018 point mirror to
  recent years, at comuna resolution. UNVERIFIED for rows; needs the file-list endpoint.

### INE georeferenced permits — official releases (geoarchivos + Geodatos Abiertos)
- Link: https://www.ine.gob.cl/herramientas/portal-de-mapas/geodatos-abiertos
- Contents: INE's own georeferenced permits and "Certificados de Recepción Final". Same point
  schema as the UC mirror, but official and updated each semester.
- Granularity: point, semester.
- Year: 2010 to present (INE says it georeferenced 98% of 2021 permits).
- Access: the old ArcGIS Hub (geoine-ine-chile.opendata.arcgis.com) is RETIRED (404). INE now
  serves Shapefile/Geodatabase zips under "Documentos y descargas -> Cartografía". INE's own GIS
  account (owner `gisine1`) hosts regional web apps "Sistema Permisos Edificación R13" (RM) etc.
- Check: VERIFIED-BYTES 2026-11-21. geoarchivos.ine.cl guide PDF HTTP 200, 1.97 MB. The 4
  permit/recepción Hub dataset URLs on the page are all HTTP 404 (hub retired). I did NOT find a
  current direct zip URL or a clean public FeatureServer for the official data.
- Limits: the public direct-download path is broken/moved; the UC mirror (above) is the working
  substitute for 2010-2018. Recepción final is not directly downloadable per INE's own guide.
- Role: INPUT
- Use in the simulation: INPUT. Preferred over the UC mirror once the live zip URL is found,
  because it reaches 2021+. Today UNVERIFIED for rows.

### MINVU geoide — national PRC/PRI/PRMS MapServer
- Link: https://geoide.minvu.cl/server/rest/services/IPT
- Contents: ArcGIS MapServer services, one per planning instrument (PRC, PRI, PRMS). Vectorized
  zones with land use, risk areas, and roads. An alternate host to the AGOL services above.
- Granularity: zone polygon.
- Year: varies by instrument (each service states its decree date).
- Access: ArcGIS REST, no key. `?f=json` on each service.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200 on the /IPT folder. I did not enumerate the RM
  services or pull zone rows here (I used the AGOL PRC/PRMS services instead).
- Limits: service names are per-instrument and inconsistent; finding the right RM comuna service
  needs a folder sweep. Overlaps the AGOL services; use whichever has the comuna you need.
- Role: INPUT
- Use in the simulation: INPUT. Backup zoning host when a comuna is missing from AGOL.

### SEIA — real-estate projects in environmental review
- Link: https://seia.sea.gob.cl/busqueda/buscarProyecto.php
- Contents: one row = one project that entered environmental review. Real-estate and urbanization
  projects above legal size (roughly >160 dwellings or >80,000 m2, or in sensitive areas) must
  enter. Each project has a comuna, a location, a developer, m2, and often a dwelling count in the
  project description, plus the dossier ("expediente") with coordinates.
- Granularity: project (point/polygon in the dossier). Dated by entry.
- Year: 2010 to present (the statistics panel runs 2010-now).
- Access: public web search (POST form). Monthly statistics reports are PDF. There is a public
  document API host (pertinencia.sea.gob.cl/api/public, HTTP 200).
- Check: VERIFIED-BYTES 2026-11-21. Search landing page HTTP 200; pertinencia API host HTTP 200.
  My GET to the search action returned 0 bytes (it needs POST). I did NOT pull a project row.
- Limits: ONLY large projects cross the SEIA threshold, so it is biased to big towers and misses
  the many mid-size permits. Coordinates sit inside PDF/dossier files, not a clean column.
- Role: INPUT
- Use in the simulation: INPUT. Forward-looking pipeline of the biggest projects, with location.
  Good for "what large towers are coming", weak as a complete permit list. UNVERIFIED for rows.

### DOM monthly permit resolutions (Ley 21.718 / DDU 525) and Transparencia Activa
- Link: https://www.minvu.gob.cl/elementos-tecnicos/circulares-division-de-desarrollo-urbano-ddu/permisos-aprobaciones-y-recepciones-edificacion/
- Contents: Ley 21.718 and DDU 525 (2025) require each municipality to publish a monthly resolution
  that lists the permits and authorizations the DOM granted the month before. Per-permit, with
  address and project data.
- Granularity: permit (address), monthly, per comuna.
- Year: 2025 onward (new obligation).
- Access: each municipal site / Transparencia Activa, mostly PDF. No single national feed.
- Check: UNVERIFIED 2026-11-21. I read the DDU rule, not a published municipal resolution. Not sampled.
- Limits: fragmented across ~52 RM comunas, PDF, new and uneven. High effort to harvest.
- Role: INPUT
- Use in the simulation: INPUT. The only near-real-time, per-permit, post-2024 source. Lead, not ready.

### DOM en Línea (MINVU national permit platform)
- Link: https://www.minvu.gob.cl/dom-en-linea/
- Contents: national platform to file DOM permits online. Holds the live permit records.
- Access: needs ClaveÚnica login. Logging in is forbidden by our rules and gets personal data.
- Check: UNVERIFIED (not attempted; login barrier).
- Limits: not an open API. Data access would need an institutional agreement.
- Role: NONE
- Use in the simulation: NONE for us. Login wall. Watch for a future open-data export.

### Observatorio del Mercado de Suelo (MINVU)
- Link: https://www.minvu.gob.cl/observatorios-del-mercado-de-suelo-urbano/
- Contents: meant to show land value, planning instruments, and the tramitación status of
  urbanization and building permits per comuna.
- Check: UNVERIFIED 2026-11-21. The permits section shows "Próximamente" (not yet live).
- Role: NONE
- Use in the simulation: NONE now. Revisit; could become a clean permit-status feed.

### SII Detalle Catastral (property register: year built, use, m2)
- Link: https://www4.sii.cl/mapasui/internet/#/contenido/index.html
- Contents: per-property year built, destino (use), and built m2 — the richest "who/what is there now".
- Check: UNVERIFIED. Needs SII login; no geometry in the export. Owned by clever-llama (building geometry).
- Role: NONE
- Use in the simulation: NONE here (cross-reference clever-llama). Login wall.

### normativa MCP tools (query_prc / query_prms / OGUC / LGUC)
- Link: local MCP server `normativa` (tools query_prc, query_prms, get_oguc_article, get_lguc_article).
- Contents: semantic search over PRC, PRMS, OGUC, LGUC text; could return density/height rules as text.
- Check: UNVERIFIED 2026-11-21. query_prc failed: "Could not connect to a Chroma server" (backend down).
  The get_*_article tools may still fetch single articles by ID, not tested.
- Limits: backend offline in this session. Returns legal text, not a point->rule lookup by itself.
- Role: INPUT
- Use in the simulation: INPUT (potential). Pairs with the PRC polygons: polygon gives the zone,
  these tools give the written rule for that zone. Needs the Chroma backend up.

### CChC and private real-estate consultancies (Colliers, GPS, Toctoc projects)
- Link: https://www.cchc.cl/ (and consultancy portals)
- Contents: new-project pipelines, sales speed, stock of new dwellings by comuna.
- Check: UNVERIFIED. Behind paid reports or membership; no open bulk data found.
- Role: NONE
- Use in the simulation: NONE (open). Keep as a paid fallback if a number is missing.

---

## Critique

My own 3 weakest points, and what I did not check:

1. **The core point source stops in 2018.** The only permits data I pulled real rows from is the
   UC mirror (2010-2018). I did NOT get INE's official 2019+ georeferenced zip or the monthly
   economic Excel; the old INE ArcGIS Hub is dead (404) and the new file list is script-loaded.
   So the "dwellings per tower" numbers are 2010-2018 only, and may not match today's unit mixes.
2. **Zoning coverage is thin and not machine-clean.** I verified PRC rows only for Estación Central
   and Cerro Navia, plus the PRMS intensity layer. Height is free text ("4 pisos") and density is
   often blank. I did not build or test a point-in-polygon lookup, and most RM comunas' PRC
   services are not confirmed here. The "legal cap" chain is shown feasible, not built.
3. **SEIA and the per-permit DOM resolutions are unverified for rows.** SEIA needs a POST search I
   did not complete, and the DOM Ley 21.718 monthly resolutions are PDF per comuna, unsampled. So
   the forward-looking and post-2018 pipeline is a lead, not data in hand.

Not checked: the "who moves in" join itself (permit points inside Census 2024 manzana polygons);
whether CANTIDAD_U double-counts across permit revisions; the gisine1 R13 web app's backing
service; geoide.minvu.cl RM comuna services; recepción final (built, not just permitted) rows.
