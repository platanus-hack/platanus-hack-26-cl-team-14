# Housing prices at point level: sale and rent (Santiago)

Owner: lucky-otter (claude-opus-5-5). Checked 2026-11-21.

Answer to "public asking-price bulk data remains unverified" (clever-heron):
- **Sale prices (real transactions): an open bulk file exists.** SII F2890 has 9,219,582 transfers at predio (rol) level with the price. No coordinates.
- **Asking prices (sale and rent): no open bulk dump exists.** The large portals block bots. Chilepropiedades.cl is the only robots-compliant portal with price, m², and address. We scrape it and geocode the address ourselves.
- **Rent: no transaction source.** Only asking prices (Chilepropiedades) and short-stay nightly prices (Inside Airbnb).

## Verified sources

### SII F2890: property transfers (notaries and Conservadores)
- Link: https://www.sii.cl/destacados/transparencia/F2890_30-10-2026.zip (linked from https://www.sii.cl/destacados/transparencia/index.html, "Transferencia de bienes raíces")
- Contents: one row = one rol in one registered transfer. Columns: `COMUNA;MANZANA;PREDIO;FECHA_INSCRIPCION;FECHA_ESCRITURA;MONTO_PESOS;MONTO_UF`.
- Granularity: predio (rol = comuna + SII manzana + predio). No coordinates. SII manzana is not INE manzana. Day.
- Year: inscription dates 1998–2026 are dense (Providencia: 3,213 rows in 1998, 6,989 in 2026 up to October). File cut 2026-10-30.
- Access: direct download. ZIP 84.8 MB, one `;`-separated TXT of 416 MB, Latin-1.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 84,822,360 bytes, 9,219,582 lines, 347 comunas. Las Condes 530,926 rows, Ñuñoa 307,592, Providencia 277,135, Puente Alto 172,158.
- Limits:
  - No use type (destino) and no m². You cannot separate a dwelling from its parking or bodega, and you cannot compute price per m², without SII Detalle Catastral (login).
  - One deed that covers several roles repeats the full amount on each row. Providencia 2025: 8,078 rows, but only 6,580 distinct (manzana, deed date, amount). Do not sum amounts per row.
  - Santiago comuna appears as 3 units: `SANTIAGO CENTRO`, `SANTIAGO OESTE`, `SANTIAGO SUR`. Comuna names have no accents (`NUNOA`, `MAIPU`). COMUNA is an SII name, not an INE CUT code; SII numeric codes also differ from CUT (Providencia 15103 vs 13123, see `santiago-source-joins.md`). Build a name crosswalk before a census join.
  - Some inscription years are typos (3015, 3200, 9204). Filter to 1990–2026. Old rows often have no amount; recent rows have it (Providencia 2025: all rows).
  - To get a point, you need rol → location. See "SII public map" below. It is not verified.
- Role: TARGET candidate for residential sale values after property classification and multi-rol deed treatment.
- Use in the simulation: retain rol-level transfer history. A location join alone does not resolve repeated full-deed prices or distinguish dwellings from other property. There is no deed identifier in this schema. Transfers do not observe household moves; do not use them as move rates. SII blocks also need a checked crosswalk before comparison with INE blocks.

### Chilepropiedades.cl: asking prices for sale and rent (scrape + geocode)
- Link: https://chilepropiedades.cl/propiedades/venta/departamento/providencia/0 and detail pages `/ver-publicacion/...`. robots.txt: https://chilepropiedades.cl/robots.txt (allows these paths, `Crawl-delay: 2`).
- Contents: one row = one listing. JSON-LD `RealEstateListing`: price, currency, floor m², bedrooms, bathrooms, year built, common expenses, street address, comuna, date posted. Our script adds lat/lon from Nominatim (OpenStreetMap).
- Granularity: point (geocoded street address). Snapshot of listings online today.
- Year: listings posted 2026 (dates seen 2026-10 to 2026-11-19).
- Access: scrape, HTML with JSON-LD, 10 listings per list page. Script: `tools/scrape/chilepropiedades.py`. Output: `data/raw/chilepropiedades/listings.csv`.
- Check: VERIFIED-ROWS 2026-11-21, per the owner’s listing inspection. The completed sample count was not recorded; full scrape coverage is unverified.
- Limits:
  - Small portal. It is not a census of the market. Most listings are departamentos from brokers.
  - The owner reports ambiguous currency labels, for example `Moneda 1576, 3150, CLP`. Preserve the source label and mark the analytical currency unknown until the listing confirms it. Never infer UF from a numeric threshold. The scraper preserves `priceCurrency` as supplied.
  - Nominatim sometimes returns the street centroid, not the house number. Error up to a few hundred meters.
  - Asking price, not sale price. Duplicates are possible (same building, many units).
- Role: INPUT (rent and asking price by point)
- Use in the simulation: INPUT for rent level by place (the only open rent price with m² and a point). TARGET check for the rent/price gradient.

### Inside Airbnb: Santiago short-stay listings
- Link: https://data.insideairbnb.com/chile/rm/santiago/2026-09-23/visualisations/listings.csv (index: https://insideairbnb.com/get-the-data/)
- Contents: one row = one Airbnb listing. Columns: id, name, neighbourhood (comuna), latitude, longitude, room_type, price (CLP per night), minimum_nights, number_of_reviews, availability_365. Also `listings.csv.gz` (full), `calendar.csv.gz`, `reviews.csv.gz`, `neighbourhoods.geojson`.
- Granularity: point (Airbnb shifts each point up to about 150 m). Snapshot 2026-09-23.
- Year: 2026.
- Access: direct download, CSV 3.5 MB. Licence CC BY 4.0.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 3,494,306 bytes, 18,904 rows (18,006 with price). Santiago 7,354, Providencia 3,021, Las Condes 2,819, Ñuñoa 1,922. We removed the host name and host id columns from our copy.
- Limits: nightly price for tourists, not monthly rent. No m². Does not meet the target "price, area, coordinates".
- Role: INPUT (short-stay listing intensity).
- Use in the simulation: count listings by zone and room type. Listings can represent rooms, occasional home sharing, or duplicate units; activity and distinct dwelling counts are unverified. Do not subtract listing counts from the long-term housing stock. Weak use for price.

### SII statistics of property by comuna (avalúo fiscal)
- Link: https://www.sii.cl/sobre_el_sii/estadisticas/csv/ebbrrc_16.csv (page: https://www.sii.cl/sobre_el_sii/estadisticas/estadisticas_bienes_raices_por_comuna.html, region 16 = RM)
- Contents: one row = one comuna (several tables: total, by use). Columns: predios total, exempt, assessed value total, mean and median assessed value, contribution.
- Granularity: comuna. It stops at comuna.
- Year: second semester 2026 (values at 2026-07-01).
- Access: direct download, CSV 25.8 KB, `;`-separated.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 25,801 bytes, 262 lines. Providencia: 216,221 predios, median avalúo M$11,381.
- Limits: assessed value, not market value. Comuna level only.
- Role: INPUT (comuna calibration)
- Use in the simulation: INPUT for comuna calibration of the price surface. Weak.

### MINVU "Avaluo_RM": land asking prices from Portal Inmobiliario, geocoded (ArcGIS)
- Link: https://services3.arcgis.com/cTnMkBRk4HWkUCRo/ArcGIS/rest/services/Avaluo_RM/FeatureServer/0 (service note: "Info apoyo Gestión de suelo"). Lead from clever-puma.
- Contents: one row = one Portal Inmobiliario listing. Columns: Tipo_Propiedad, tipo_aviso, comuna, Fecha (Excel serial), Total_UF, Sup_m2, UFm2, Direccion, CoordY/CoordX, Zona (PRC zone code), url.
- Granularity: point. Snapshot.
- Year: 2022-01-01 to 2022-04-21 (Fecha 44562 to 44651).
- Access: ArcGIS REST query API, no key. CSV, shapefile, GeoPackage export.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 521 rows, 3 read. All 521 are land: Terrenos 264, Sitios 201, Industriales 27, Loteos 23, Parcelas 6. Extent lon -70.83 to -70.47.
- Limits: land only, no dwellings. Old (2022). `Observaciones` and `Vendedor` hold broker names and phone numbers: do not copy those columns.
- Role: INPUT (land price per m² by point)
- Use in the simulation: INPUT for land price per m² (UF/m²), the cost side of new housing (for example the "vertical ghetto" site). Small sample.

## Blocked or not usable

### Portal Inmobiliario (Mercado Libre)
- Link: https://www.portalinmobiliario.com/robots.txt
- Check: BLOCKED 2026-11-21. Even `robots.txt` redirects (302) to `/gz/account-verification` (bot wall). We did not scrape.
- Role: NONE
- Use in the simulation: NONE for direct access (largest Chilean portal). A 2022 land-only slice exists in MINVU "Avaluo_RM" above.

### Mercado Libre public API (site MLC, real estate)
- Link: https://api.mercadolibre.com/sites/MLC/search?category=MLC1459
- Check: BLOCKED 2026-11-21. HTTP 403 `forbidden` without a token. `robots.txt` says `Disallow: /`.
- Role: NONE
- Use in the simulation: NONE without an app token. A human can register an ML app; the robots rule still applies.

### iCasas.cl
- Link: https://www.icasas.cl/robots.txt
- Check: BLOCKED 2026-11-21. HTTP 200, ends with `Disallow: /`.
- Role: NONE
- Use in the simulation: NONE.

### Mitula
- Link: https://www.mitula.cl/robots.txt → https://casas.mitula.cl/robots.txt
- Check: BLOCKED 2026-11-21. HTTP 401 `Access Denied`. It is an aggregator of other portals.
- Role: NONE
- Use in the simulation: NONE.

### Trovit
- Link: https://www.trovit.cl/robots.txt
- Check: BLOCKED for listings 2026-11-21. HTTP 200, `Disallow: /listing/` and `/details/`. Aggregator.
- Role: NONE
- Use in the simulation: NONE.

### Toctoc
- Link: https://www.toctoc.com/robots.txt, sitemap https://www.toctoc.com/gw-landing/sitemap/Sitemap_Comprar_DepartamentoMetropolitana.xml
- Check: UNVERIFIED 2026-11-21. robots.txt disallows `/search`, `/resultados/`, `/propiedades/arriendo/`. The sitemap listing URL we opened returned 404. Data loads in the browser only.
- Role: NONE
- Use in the simulation: NONE (no compliant bulk route found).

### Goplaceit
- Link: https://www.goplaceit.com/cl/propiedad/venta/comuna/providencia (allowed by robots.txt), API base `https://api.goplaceit.com/api/v4/` (its robots.txt is empty)
- Check: UNVERIFIED 2026-11-21. The page is a JS app (HTML 5.7 KB, no listings). We did not find the search endpoint in the JS bundle. The bundle has `locations/role_by_number?role_number=...`, a possible rol → location route. Not tested.
- Role: NONE
- Use in the simulation: unknown. Lead for rol → point.

### Yapo
- Link: https://www.yapo.cl/bienes-raices/region-metropolitana
- Check: UNVERIFIED 2026-11-21. HTTP 200, 81 KB, no listings, prices, or coordinates in the HTML (client-rendered).
- Role: NONE
- Use in the simulation: NONE found.

### Properati
- Link: https://www.properati.cl/robots.txt
- Check: no response 2026-11-21 (HTTP 000, two tries). The open Properati dataset on Zenodo is Lima, Peru (https://zenodo.org/records/7846211).
- Role: NONE
- Use in the simulation: NONE.

### Banco Central: house price index (IPV)
- Link: https://si3.bcentral.cl/siete/ES/Siete/Cuadro/CAP_BDP/MN_IND_SEC20/IS_GENERAL_PROPIEDAD_C_08
- Check: UNVERIFIED rows 2026-11-21. BDE page HTTP 200 shows the table (quarterly 2003–2026, by type and by zone), but the values load by script. The main bcentral.cl pages return an Incapsula challenge (212 bytes). The IPV is built from the same SII transactions as F2890.
- Limits: zone level, not comuna. F2890 gives the raw data below it.
- Role: TARGET (aggregate trend only)
- Use in the simulation: TARGET for the aggregate price trend only.

### CChC housing price index and sales
- Link: https://cchc.cl/centro-de-informacion/indicadores/indice-real-de-precios-de-viviendas
- Check: UNVERIFIED 2026-11-21, HTTP 404. BDE lists "Compraventa de viviendas, CChC (unidades, promesas)" as a series.
- Role: NONE
- Use in the simulation: NONE at point level.

### MINVU Observatorio Urbano
- Link: https://www.observatoriourbano.cl/
- Check: UNVERIFIED 2026-11-21, HTTP 403 (9 bytes) to curl.
- Role: NONE
- Use in the simulation: unknown.

### Conservador de Bienes Raíces de Santiago (CBRS)
- Link: https://www.conservador.cl/portal/indice_propiedad
- Check: UNVERIFIED 2026-11-21, HTTP 200, search form. Search by person or by foja, one certificate at a time, paid. It holds personal data.
- Role: NONE
- Use in the simulation: NONE (F2890 already has the same transfers in bulk).

### SII public map: rol → location (lead, needs human approval)
- Link: https://www4.sii.cl/mapasui/internet/ (services `/mapasui/services/data/mapasFacadeService/getPredioNacional`, `listComunas`)
- Check: UNVERIFIED. The public map shows each predio. Our tool permission system denied calls to the predio service, so we stopped. A human must decide if the team may use this service.
- Role: NONE
- Use in the simulation: it would turn F2890 rows into points. This is the missing join for point-level sale prices.

## Critique
1. F2890 is the best source, but it has no point and no use type. Until rol → location and rol → destino are solved, it gives price per rol and per SII manzana only, and apartment sales mix with parking and bodega sales.
2. Chilepropiedades is small and broker-biased. Our sample covers 8 comunas, departamentos only. It is not representative of the south and west of Santiago, and the geocoding has street-level error.
3. Not checked: Portal Inmobiliario with an official ML app token, the Goplaceit `role_by_number` endpoint, SII Detalle Catastral (login), the values in BCCh IPV, and SII homogeneous-area land values (WMS layers in the SII map).
