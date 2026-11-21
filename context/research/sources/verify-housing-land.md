# Housing and land: independent sample audit

Checked on 2026-11-21. Owner: clever-heron. Reviewed file: `origin/main:context/research/santiago-housing-land.md`.
Ten-minute owner recheck at 05:21 UTC: after fetch, the latest file commit remains `5722e3b`; no newer owner version appeared on main.
All eight source families received a request. Four returned actual observations or spatial records. Four did not.
An additional SII sale-transfer source found by lucky-otter also returned real rows; see the supplemental entry.
`OK` means the owner's bounded claim agrees with this check. It does not mean full ingestion is ready.

## Evidence and method

- Run: `/usr/bin/python3 tools/scrape/verify_housing_land.py` from the repo root.
- Local evidence: `data/raw/verify-housing-land/manifest.json`, response bodies, and layer metadata. Raw data are git-ignored.
- Requests use the standard library, check robots.txt, and run at most once per 1.1 seconds per host. No login or personal-owner fields.
- Curl also saved the MINVU robots response and a green-area query response with headers. Other downloads used urllib.
- GIS queries request five records. Settlement queries intersect a rough Santiago envelope: longitude -70.85 to -70.45, latitude -33.65 to -33.30.
- This envelope is a sample window, not a city boundary. Permit samples use region code 13 and therefore include Pirque.
- Native GIS coordinates use Web Mercator, EPSG:3857. Sample output uses EPSG:4326. Source service IDs and column names are not interchangeable.
- Research brief: reuse publisher query services rather than scrape map pixels. Check both HTTP status and body content. Separate edition labels from observation dates.

## Sources

### SII property assessment and cartography
- Link: https://www.sii.cl/preguntas_frecuentes/aval_contrib_bbrr/001_165_8108.htm
- Contents: documentation of fiscal reassessment and public address-based consultation. No property records downloaded; no data columns established.
- Granularity: property lookup is documented; the downloaded artifact is a national FAQ.
- Year: FAQ updated 2026-04-09. No assessment reference year established from property rows.
- Access: HTML documentation, 4,611 bytes. No bulk endpoint confirmed.
- Check: UNVERIFIED for property rows; HTTP 200 on 2026-11-21. Saved `sii.html`.
- Verdict: OK. The owner correctly calls this documentation only. The FAQ explicitly mentions public assessed value and use by address.
- Role: INPUT lead.
- Limits: fiscal value is not a sale price. This check does not establish parcel geometry, dwelling counts, or bulk availability.
- Use in the simulation: land-use allocation only after an authorized public record export is found. The FAQ itself supplies no spatial input.

### BCCh housing price index, RM Centro
- Link: https://si3.bcentral.cl/siete/ES/Siete/Cuadro/CAP_ESTADIST_EXPERIM/MN_EXPERIM01/IS_PRECIOS_GEO_2008?idSerie=F034.IPVZ41.FLU.BCCH.2008.0.T
- Contents: one observed quarterly index value in the HTML. Fields read: series code, zone name, period, value, unit, update date.
- Granularity: BCCh RM Centro zone; quarterly. It is not Santiago comuna or a parcel.
- Year: 2026 Q2 observation; updated 2026-10-30. Series identifier includes base 2008.
- Access: public HTML, 80,850 bytes. Excel export exists in the page but was not executed.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200, one observation read. `ipv.html` contains `II.2026: 227,2`, `RM Centro`, and the claimed series identifier.
- Verdict: OK. The owner's 227.2 observation agrees with the saved response.
- Role: TARGET candidate for price changes after multiple comparable quarters are retrieved.
- Limits: one level cannot score a price change. No historical series export or individual transactions downloaded. Zone membership was not checked.
- Use in the simulation: compare aggregate price trends for exactly the same zone and periods.

### BCCh market value of housing, VMV
- Link: https://www.bcentral.cl/areas/estadisticas/estadisticas-experimentales/vmv
- Additional opened link: https://www.bcentral.cl/documents/33528/3662096/Nota-tecnica-VMV.pdf
- Contents: both responses contain an Incapsula JavaScript challenge. No housing observations, PDF pages, or data columns.
- Granularity: unverified from records. The owner describes national and published geographic aggregates.
- Year: unverified. The owner's start year 2012 and 2023 method date were not independently reproduced.
- Access: HTTP 200 challenge HTML: page 212 bytes initially, 845 bytes on the final run; method PDF URL 212 bytes. No PDF bytes.
- Check: UNVERIFIED for data, 2026-11-21. Saved `vmv.html` and `vmv-note.pdf`; inspect content before using the extension.
- Verdict: OK as an unverified lead. The owner's earlier page access did not reproduce in this client.
- Role: INPUT lead; modeled stock value is not an independent transaction target.
- Limits: response status and filename do not prove access. No challenge bypass attempted.
- Use in the simulation: NONE from the downloaded bodies. Retain the proposed aggregate stock constraint only as a lead.

### TOCTOC listings
- Link: https://www.toctoc.com/
- Contents: forbidden response. No listing, asking price, property type, or location columns downloaded.
- Granularity: unverified in this request; the owner describes listing-level offers.
- Year: no listing observation date established.
- Access: HTTP 403, 118 bytes of HTML.
- Check: UNVERIFIED for listings, 2026-11-21. Saved `toctoc.html`.
- Verdict: OK as a lead with bulk access unverified. The owner's visible listing claim was not independently reproduced.
- Role: INPUT lead.
- Limits: no API, bulk licence, duplicate handling, history, or location quality established. Asking prices would not equal closing prices.
- Use in the simulation: NONE from this response. BCCh IPV is the verified aggregate price observation in this audit.

### INE building permits
- Link: https://www.ine.gob.cl/estadisticas-por-tema/industria-energia-y-construccion/permisos-de-edificacion
- Data link opened: https://services5.arcgis.com/hUyD8u3TeZLKPe4T/arcgis/rest/services/CRF_PE_v2/FeatureServer/1
- Discovery: official page links app `2c09e6be8f2f49af922fcb7184ea0ecf`; app and webmap `34fdbaa793e643eb9f87c4c4751ff4a7` expose this layer.
- Contents: point records with `COD_REGION`, `COD_COMUNA`, `GLOSA_COMUNA`, `AÑO`, `CANTIDAD_UNIDAD`, `NUM_PISOS`, `GLOSA_DESTINO`, `USO_DESTINO`, `SUPERFICIE`, `PRECISION_`, and `OBJECTID`.
- Granularity: permit destination point; annual data. RM filter is broader than Santiago.
- Year: queried RM minimum and maximum are 2010 and 2025. Five sampled rows are 2025. Bounds do not prove every intervening year is complete.
- Access: public ArcGIS JSON. Official page: HTTP 200, 120,508 bytes. Layer metadata: 15,278 bytes.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200; five records in `ine-permits-sample.json`, 3,756 bytes. Year statistics: HTTP 200, 561 bytes in `ine-permits-years.json`.
- Example: Quinta Normal, CUT 13126, 2025, one unit, two floors, `CASA AISLADA`, residential, surface 90, precision `ALTA`; point (-70.703629, -33.412837).
- Verdict: OK; the owner's portal-only status is now superseded by this sample. Precise point access is confirmed for sampled records.
- Role: INPUT.
- Limits: permit destination records are not completed buildings or occupied homes. Do not sum units across destinations without checking the record structure. Precision labels vary. No final-reception layer rows checked.
- Use in the simulation: locate permitted housing supply. Use a separate completion/occupancy process before adding residents.

### MINVU communal zoning, Metropolitan Region
- Link: https://geoportal.cl/geoportal/catalog/36394/zonificacion-planes-reguladores-comunales-region-metropolitana
- Service opened: https://geoide.minvu.cl/server/rest/services/IPT/PRC_Metropolitana/MapServer?f=pjson
- Contents: catalog metadata and an error response. No zone polygon, regulation column, or plan date retrieved.
- Granularity: zone polygons claimed by metadata; no features checked.
- Year: actual plan dates unverified. A catalog publication date is not a plan reference year.
- Access: catalog HTTP 200, 35,844 bytes. Linked service HTTP 200, 116 bytes, with JSON `error.code=404` and `Service IPT/PRC_Metropolitana/MapServer not found`.
- Check: UNVERIFIED for zoning rows, 2026-11-21. Saved `zoning.html` and `zoning-service.json`.
- Verdict: OK. The owner correctly reports the broken service. USELESS for immediate ingestion through this URL.
- Role: INPUT lead.
- Limits: transport zones and actual land cover cannot replace legal zoning. No alternative zoning export checked in this bounded audit.
- Use in the simulation: NONE until a working, dated zoning export is obtained.

### MINVU–INE green areas
- Link: https://geoide.minvu.cl/server/rest/services/Catastros/%C3%81reas_Verdes_MINVU_INE_2023/FeatureServer/0
- Contents: polygon records. Sample columns: `CUT`, `COMUNA`, `NOMBRE_EP`, `SUP_TOTAL_`, `SUP_VEGETA`, `USO`, `TIPO_EP`, `ID_AV`.
- Granularity: park/plaza polygon. Query selects `COMUNA = 'SANTIAGO'` and returns geometry in EPSG:4326.
- Year: edition label 2023. Sample rows have no observation-year field; individual survey dates remain unverified.
- Access: public ArcGIS query API. Service maximum is 2,000 records per response.
- Check: VERIFIED-ROWS, 2026-11-21, HTTP 200, five polygons, 26,627 bytes in `green-sample.json`.
- Example: CUT 13101, park `DE LOS REYES`, `ID_AV=1310110001`, `SUP_TOTAL_=209910.240686`, use `PÚBLICO`.
- Verdict: OK; metadata-only status is now superseded by real polygons.
- Role: INPUT.
- Limits: all five sample values of `SUP_VEGETA` are blank strings; do not treat blanks as zero vegetation. Two park/place names are blank. `exceededTransferLimit=true`: this is not the complete comuna inventory. Units of area attributes were not independently checked.
- Use in the simulation: recreation destinations and network accessibility. Polygon size is not observed attendance or usable capacity.

### MINVU campamentos, 2024 and 2026 editions
- Link: https://geoide.minvu.cl/server/rest/services/Catastros/Catastros_campamentos/FeatureServer
- Contents: settlement polygons. 2024 columns: `FOLIO,NOMBRE,CUT_R,CUT,COMUNA,AN_CAT,N_HOG,HECTAREAS`. 2026 columns: `FOLIO,NOM,REG,COM,INGRESO,EST,HOGARESCAT,HOGARESCEN,SUPERFIC_1`.
- Granularity: settlement polygon and household counts. Rough Santiago envelope selects Puente Alto, Huechuraba, and La Florida in these samples.
- Year: live service labels 2024 and 2026. All five sampled records in each have `CATASTRO_2011` in `AN_CAT` or `INGRESO`. These fields must not be assumed to mean count observation year.
- Access: public ArcGIS JSON. Live service: HTTP 200, 4,141 bytes. Layer 1 is 2026; layer 2 is 2024 in this check.
- Check: VERIFIED-ROWS, 2026-11-21. 2024 urban sample: HTTP 200, five polygons, 6,988 bytes in `settlements-2024-urban-sample.json`. 2026 sample: HTTP 200, five polygons, 7,267 bytes in `settlements-2026-urban-sample.json`.
- Example: Los Areneros, Puente Alto, `FOLIO=1320106`: 2024 `N_HOG=97`; 2026 `HOGARESCAT=97`, `HOGARESCEN='70'`. These are different columns, not a verified 97-to-70 household trend.
- Verdict: OK on live layer IDs and query availability. Add a warning about incompatible schemas and count definitions before using these as dated household totals.
- Role: INPUT.
- Limits: resolve each layer by name at download time. Search indexes show different IDs. 2026 household columns have different types and disagree. Survey reference dates and definitions remain unverified. No private property-owner fields requested.
- Use in the simulation: settlement locations. Use household counts only after their definitions and dates are reconciled with Census totals.

### Supplemental: SII F2890 sale transfers
- Link: https://www.sii.cl/destacados/transparencia/F2890_30-10-2026.zip
- Contents: semicolon-delimited records with `COMUNA;MANZANA;PREDIO;FECHA_INSCRIPCION;FECHA_ESCRITURA;MONTO_PESOS;MONTO_UF`.
- Granularity: property rol within named comuna; transfer dates. No coordinates, floor area, use, or deed identifier in these columns.
- Year: archive label 2026-10-30. Independently read sample inscription dates 2008, 2011, 2014, 2015, and 2022 for Santiago Centro. Full time bounds and latest transactions were not checked.
- Access: direct public ZIP. Independent curl range: HTTP 206, 262,144 bytes; Content-Range gives total 84,822,360 bytes. First member is `F2890_30-10-2026.txt`, deflate-compressed.
- Check: VERIFIED-ROWS, 2026-11-21. Independently inflated the first 10,000 uncompressed bytes of the partial ZIP and read Algarrobo records. Separately read five `SANTIAGO CENTRO` rows from the peer-downloaded local full text at `data/raw/sii_f2890/F2890_30-10-2026.txt`. Saved my 314-byte extract as `f2890-santiago-sample.csv` under this audit's raw folder. I did not repeat the full download.
- Example: Santiago Centro, manzana 1, predio 1, inscription 2011-01-13, deed 2010-12-27, UF 3000; pesos blank. Predio 6 has inscription 2008-04-03 and 127,000,000 pesos; UF blank.
- Verdict: OK on the peer's public-file and column claims. Any broad claim that public property sale-transfer history does not exist is WRONG. The owner's narrower address-level gap still needs a rol-to-address/geometry join.
- Role: INPUT candidate after property classification and transfer treatment are defined.
- Limits: SII labels the comuna `SANTIAGO CENTRO`, while MINVU green areas use `SANTIAGO`; join by a checked crosswalk. Missing amounts are not zeros. Peer warns that multi-rol deeds repeat amounts; no deed key is present to resolve this in the sampled schema. No inference of residential use from price alone.
- Use in the simulation: historical rol-level sale values after joining residential property attributes. This is not a rental series or a ready price-per-square-metre target.
- Reproduction: the main downloader covers the original eight families. Supplemental evidence uses curl range 0–262143 on the ZIP above; the Santiago extract depends on the peer's full download.

## Critique

1. Tiny samples establish access and columns, not coverage. No full city extraction, pagination, duplicate audit, or final-reception records were checked.
2. Reference dates remain weak: green-area rows lack survey dates; settlement edition labels differ from row entry labels. Count semantics need publisher documentation.
3. No zoning input or rental transaction history was verified. The additional SII sale file has real property records, but spatial joins, residential classification, and multi-rol deed treatment remain unresolved.
