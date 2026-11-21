# Santiago car traffic: counts, speed, tolls, parking, and ownership

### COPSA urban-highway annual flows
- Link: https://estadisticas.copsa.cl/autopistas-urbanas/
- Contents: one row per year: total toll-gate and gantry passes, plus annual change.
- Granularity: region. Annual.
- Year: 2012-2023.
- Access: direct web table. No documented bulk export.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 12 displayed rows. The 2023 value is 1,713,850,698 passes.
- Limits: it sums all urban-concession gates. A vehicle can create several passes. It has no route, time, or comuna field. Its 2014 total conflicts with the class table by 6,081,462 passes, and 2019 conflicts by 40,158.
- Role: NONE
- Use in the simulation: NONE until the table mismatch is reconciled with COPSA.

### COPSA urban-highway flows by vehicle class
- Link: https://estadisticas.copsa.cl/autopistas-urbanas/
- Contents: one row per year: light-vehicle, heavy-vehicle, and total passes.
- Granularity: region. Annual.
- Year: 2012-2023.
- Access: direct web table. No documented bulk export.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 12 displayed rows. The 2023 row has 1,553,720,812 light and 160,129,886 heavy passes.
- Limits: it has the same multi-gantry double count as the total. It does not identify the concession or vehicle owner. Its light-plus-heavy sum conflicts with COPSA's total table in 2014 and 2019.
- Role: NONE
- Use in the simulation: NONE until COPSA reconciles the table mismatch.

### INE permits of circulation
- Link: https://www.ine.gob.cl/docs/default-source/parque-de-vehiculos/cuadros-estadisticos/resultados/2024.xlsx?sfvrsn=5801b9b3_4
- Contents: annual motorized and non-motorized vehicles by region, province, comuna, vehicle type, fuel, and use. Sheet 3 is the region-province-comuna table.
- Granularity: comuna. Annual.
- Year: 2024, with prior annual files linked by INE.
- Access: direct XLSX, 189 kB on the source index.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 193,510 bytes downloaded. The workbook has 9 sheets. Sheet 3 has 485 XML rows and is titled for region, province, and comuna. No source data records established by this check.
- Limits: the comuna is where the municipality issued the permit, not necessarily the owner residence or where the vehicle drives. The 2024 value is provisional.
- Role: INPUT
- Use in the simulation: INPUT LEAD for vehicle totals. Issuing comuna cannot set residents' car availability without a validated residence allocation.

### UOCT controlled intersections
- Link: https://www.transporteinforma.cl/content/uploads/2023/10/Intersecciones-de-Semaforos-conectados-a-Sistema-UOCT-RM.pdf
- Contents: one row per connected intersection: control code, two approaches, comuna, latitude, and longitude.
- Granularity: point. Static list.
- Year: revision 15 May 2023.
- Access: direct PDF.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 2,360,461 bytes downloaded. The PDF has 29 pages and real point rows.
- Limits: several printed coordinates are malformed. It says where UOCT controls signals, not traffic counts or signal timing.
- Role: INPUT
- Use in the simulation: INPUT for the signal-control coverage mask. It cannot calibrate flows.

### UOCT normal-season count specification
- Link: https://www.transporteinforma.cl/content/uploads/2026/02/Temporada-Normal-2026.pdf
- Contents: the 2026 protocol for vehicle and pedestrian flow measurements in normal season.
- Granularity: point. Collection protocol.
- Year: 2026.
- Access: direct PDF.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, one-page PDF opened. The page does not contain measurement rows. No source data records established by this check.
- Limits: this is a collection specification, not a published count dataset. Obtain counts from UOCT or the responsible municipality.
- Role: NONE
- Use in the simulation: NONE until UOCT supplies observations. It defines how a future count target should be collected.

### UOCT traffic data request
- Link: https://www.chileatiende.gob.cl/fichas/23628/1/pdf
- Contents: continuous, periodic, and station traffic counts; signal programs; signal plans; Transyt/SIDRA models; and flow-sensor locations.
- Granularity: point and intersection. The requested product sets the time step.
- Year: current request service. Page updated 3 September 2026.
- Access: request, quotation, and payment by email. No public data file.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 11,956-byte official PDF. The document lists the data products and their prices. No source data records established by this check.
- Limits: this request page supplies no count rows. Cost and delivery terms require a request.
- Role: NONE
- Use in the simulation: NONE until acquired; candidate TARGET for intersection flow after obtaining records.

### INE monthly urban-toll passes
- Link: https://www.ine.gob.cl/docs/default-source/transporte-y-comunicaciones/boletines/2025/boletin_transporte-y-comunicaciones-marzo-2025.pdf
- Contents: monthly total vehicle passes through RM urban-highway gantries and year-on-year change.
- Granularity: region. Monthly.
- Year: January 2023-March 2025 in the checked chart.
- Access: direct PDF.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 1,442,755 bytes. March 2025 reports 138.1 million passes.
- Limits: a gantry pass is not a unique vehicle or trip. No road, gantry, comuna, or hour fields are published.
- Role: TARGET
- Use in the simulation: TARGET for monthly simulated gantry passes, counting every passage; not unique trips.

### UOCT 74-counter challenge schema
- Link: https://s3.amazonaws.com/documentos.anid.cl/investigacion-aplicada/2022/DesafiosPublicos/Guia_tecnica_Subsecretaria_Transportes.pdf
- Contents: documentation for 74 Santiago counters from March 2021. Listed fields include counter ID, comuna, latitude, longitude, count date, 15-minute interval, observed flow, and projected flow.
- Granularity: point. 15 minutes.
- Year: from March 2021.
- Access: restricted to the ANID challenge innovators. The guide calls the data confidential.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 1,057,651-byte PDF. No data rows were available. No source data records established by this check.
- Limits: east-side spatial bias and 30-90 minute processing delay are documented. Reuse requires authorization.
- Role: NONE
- Use in the simulation: NONE without authorization. It is the best documented link-level calibration lead.

### SECTRA Santiago origin-destination survey
- Link: https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/
- Contents: household travel records, zoning, and report download links. The site reports total daily trips and private-transport trips.
- Granularity: zone and person trip after download. Daily survey expansion.
- Year: 2012.
- Access: direct Google Drive links from the official SECTRA page.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, official page shows 2,051,310 households, 18,461,134 daily trips, and 5,859,965 private-transport trips.
- Limits: 2012 predates major Metro extensions and the pandemic. The Drive ZIP was not downloaded in this pass.
- Role: INPUT
- Use in the simulation: INPUT for private-car trip purpose and OD priors. Reweight with current counts.

### TomTom Traffic Index Santiago
- Link: https://www.tomtom.com/traffic-index/city/santiago/
- Contents: city and metro congestion, travel time for 10 km, speed, distance in 15 minutes, day-hour profiles, and highway ratio.
- Granularity: city and metro. Annual and day-hour aggregates.
- Year: 2025.
- Access: direct web dashboard. The underlying trip data is not downloadable from this page.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, page reports 52.4% average congestion, 23 min 54 s per 10 km, and 18.5 km/h peak speed.
- Limits: vendor-derived aggregates with no street, comuna, vehicle, or sample-size export. Live fields returned NaN during the check.
- Role: TARGET
- Use in the simulation: TARGET for metropolitan travel-time and peak-speed outputs, not a link-level input.

### Costanera Norte 2026 toll schedule
- Link: https://web.costaneranorte.cl/tarifas/
- Contents: one row per gantry, direction, segment length, vehicle class, and off-peak, peak, and saturation toll.
- Granularity: gantry and direction. Time bands within 2026.
- Year: 2026.
- Access: direct web tables and PDF.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, page returns gantry rows. P1 east-west is 4.2 km and costs CLP 449.40, 865.20, or 1,310.40 for the light-vehicle bands.
- Limits: price schedule only. It gives no pass counts, route choice, or payment behavior.
- Role: INPUT
- Use in the simulation: INPUT for generalized monetary cost on Costanera Norte.

### Autopista Central 2026 toll schedule
- Link: https://www.autopistacentral.cl/wp-content/uploads/2026/01/ACSA-completo_afiche_25x32-cm_ajuste_21.pdf
- Contents: gantry, direction, segment length, vehicle class, time band, and toll values for the Norte-Sur system.
- Granularity: gantry and direction. Time bands within 2026.
- Year: 2026.
- Access: direct PDF.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, PDF opened. It states CLP 103.924/km off peak, 207.849/km peak, and 311.773/km saturation for 2026.
- Limits: price schedule only. It has no observed traffic transactions by gantry.
- Role: INPUT
- Use in the simulation: INPUT for generalized monetary cost on Autopista Central.

### Vespucio Norte 2024 annual report
- Link: https://www.vespucionorte.cl/Content/AVN/pdfs/memorias/AVN_memoria_2024.pdf
- Contents: annual IMD change, transaction total, and vehicle-class shares for the concession.
- Granularity: concession. Annual.
- Year: 2024.
- Access: direct PDF.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, report states 296.5 million transactions, 88% motorcycles and cars, 7.5% buses and trucks, and 4.5% articulated trucks.
- Limits: it is a company annual report. It has no per-gantry, hour, origin, or destination data.
- Role: TARGET
- Use in the simulation: TARGET for total demand and vehicle mix on this corridor.

### Parking availability and price data
- Link: https://odecu.cl/wp-content/uploads/2025/11/Estudio-ODECU-Analisis-de-la-ejecucion-de-la-Ley-20.967-2025.pdf
- Contents: an inspection sample of parking charging practices, not an inventory, price API, or occupancy series.
- Granularity: named facility sample. One inspection period.
- Year: 2025.
- Access: direct PDF.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, report text identifies five inspected parking facilities in comuna de Santiago. No source data records established by this check.
- Limits: no location-complete inventory, capacity, vacancy, or reusable price table. It cannot represent parking supply citywide.
- Role: NONE
- Use in the simulation: NONE. This catalog found no verified open parking supply or occupancy dataset.

### ArcGIS parking feature layer
- Link: https://services2.arcgis.com/cRH3tEMPESJz6DMX/ArcGIS/rest/services/Estacionamientos/FeatureServer/0/query?where=1%3D1&outFields=*&resultRecordCount=2&f=geojson
- Contents: the layer schema defines parking ID, infrastructure, use, and security fields.
- Granularity: point.
- Year: not stated.
- Access: public ArcGIS REST query.
- Check: UNVERIFIED for source data; 2026-11-21, HTTP 200, 42-byte GeoJSON FeatureCollection with zero features. No source data records established by this check.
- Limits: the live layer has no rows. Its coverage and publisher cannot be established from an empty response.
- Role: NONE
- Use in the simulation: NONE.

## Critique

- The strongest count targets aggregate gantry passes. They cannot calibrate cars on ordinary streets or unique vehicles.
- UOCT data confirms control points and measurement methods, but no public count, speed, phase, or detector feed was obtained.
- Parking is a major missing mechanism. This pass found only a small consumer inspection, not a usable supply or occupancy source.
