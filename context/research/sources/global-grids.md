# Global open datasets that cover Santiago

Topic: global raster and vector grids that give population, land cover, terrain, connectivity, or POIs for Santiago. Excludes local Chilean sources and building footprints (owned by clever-llama).

**Use priority:** Census 2024 manzanas (a local Chilean source) give real persons and dwellings by block. The global rasters here are fallbacks: use them when the census is unavailable or when you need spatial disaggregation below the manzana level.

Agent: claude-sonnet-4-6 (emersoftware@Emersons-MacBook-Air), 2026-11-21.

---

### WorldPop Chile 2020 (100 m constrained, R2025A)

- Link: https://data.worldpop.org/GIS/Population/Global_2015_2030/R2025A/2020/CHL/v1/100m/constrained/chl_pop_2020_CN_100m_R2025A_v1.tif
- Contents: One pixel = count of people living in that 100 m cell. UN-adjusted, constrained to built area. Single float band.
- Granularity: point (100 m pixel). Static 2020 snapshot.
- Year: 2020 (model from 2025 reanalysis R2025A)
- Access: direct download. GeoTIFF. 62 MB.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, Content-Length 62,328,145 bytes. No rows parsed.
- Role: INPUT
- Limits: modeled, not census. Constrained version underestimates in informal settlements if built-area mask misses them. License: CC BY 4.0.
- Use in the simulation: home location probability surface; calibrate total to INE commune projections.

---

### WorldPop Chile 2020 (1 km unconstrained, 2020 release)

- Link: https://data.worldpop.org/GIS/Population/Global_2000_2020/2020/CHL/chl_ppp_2020_UNadj.tif
- Contents: One pixel = UN-adjusted count of people in ~1 km cell. Single float band.
- Granularity: point (~1 km pixel). Static 2020.
- Year: 2020
- Access: direct download. GeoTIFF. 478 MB (whole Chile).
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, Content-Length 501,470,900 bytes. No rows parsed.
- Role: INPUT
- Limits: 1 km is too coarse for intra-urban use in a 100 m simulation. Use only as a cross-check for the 100 m product. License: CC BY 4.0.
- Use in the simulation: cross-check population totals against the 100 m R2025A product only.

---

### GHSL GHS-POP R2023A (100 m Mollweide, 2020 epoch)

- Link: https://jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/GHSL/GHS_POP_GLOBE_R2023A/GHS_POP_E2020_GLOBE_R2023A_54009_100/V1-0/GHS_POP_E2020_GLOBE_R2023A_54009_100_V1_0.zip
- Contents: One pixel = estimated resident population count in 100 m Mollweide cell. Covers 1975–2030 (5-year steps); E2020 epoch linked here.
- Granularity: point (100 m Mollweide pixel). Time series 1975–2030.
- Year: 2020 epoch (1975–2030 available)
- Access: direct download. ZIP of global GeoTIFF or per-tile ZIPs in tiles/ subfolder. Global ZIP: 5.1 GB.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206, Content-Range /5,097,074,334 bytes. No rows parsed; tile extraction for Santiago not done (Mollweide coordinates needed to identify tile row/column).
- Role: INPUT
- Limits: Mollweide projection (EPSG:54009); reproject to UTM 19S before use. Resolution nominally 100 m but input data varies. License: CC BY 4.0.
- Use in the simulation: alternative population surface; temporal trend 2000–2020 for urban growth model.

---

### GHSL GHS-BUILT-H R2023A (average building height, 3 arc-sec = ~92 m, 2018)

- Link: https://jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/GHSL/GHS_BUILT_H_GLOBE_R2023A/GHS_BUILT_H_AGBH_E2018_GLOBE_R2023A_4326_3ss/V1-0/GHS_BUILT_H_AGBH_E2018_GLOBE_R2023A_4326_3ss_V1_0.zip
- Contents: One pixel = average building height in meters per ~92 m cell (AGBH product). Derived from AW3D30, SRTM30, Sentinel-2. Sister product ANBH (normalized) in adjacent folder.
- Granularity: point (~92 m pixel, 3 arc-sec WGS84). Static 2018.
- Year: 2018
- Access: direct download. ZIP of global GeoTIFF or per-tile ZIPs in tiles/ subfolder. Global ZIP: 7.9 GB.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206, Content-Range /7,869,233,133 bytes. No rows parsed.
- Role: INPUT
- Limits: Predicted, not measured. No LiDAR calibration in Chile. Very tall buildings underestimated. Building footprints (clever-llama scope) are needed to convert pixel height to per-building height. Do not sum pixel height values across a footprint — use a per-footprint statistic (mean or max). License: CC BY 4.0.
- Use in the simulation: building mass proxy; floor count estimation when combined with footprints.

---

### Kontur Population Chile H3 (400 m hexagons, 2023)

- Link: https://geodata-eu-central-1-kontur-public.s3.amazonaws.com/kontur_datasets/kontur_population_CL_20231101.gpkg.gz
- Contents: One row = one H3 hexagon (resolution 8, ~460 m diameter). Columns: h3 (H3 index), population (integer). Population from WorldPop + OSM built-area intersection.
- Granularity: H3 hex resolution 8 (~460 m diameter). Static Nov 2023.
- Year: 2023
- Access: direct download. GeoPackage gzip. 10.7 MB.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206, Content-Range /10,693,383 bytes, Last-Modified 2023-10-31. File is a valid GeoPackage (application/geopackage+sqlite3 content-type). Rows not parsed.
- Role: INPUT
- Limits: Derived from WorldPop; same biases apply. H3 hexagons cross administrative boundaries, making commune-level aggregation approximate. License: CC BY 4.0.
- Use in the simulation: ready-to-use hex grid for the agent simulation; avoids raster-to-vector conversion.

---

### ESA WorldCover 2021 v200 (10 m land cover, tile S36W072)

- Link: https://esa-worldcover.s3.eu-central-1.amazonaws.com/v200/2021/map/ESA_WorldCover_10m_2021_v200_S36W072_Map.tif
- Contents: One pixel = land cover class (11 classes coded uint8: 10 tree cover, 20 shrubland, 30 grassland, 40 cropland, 50 built-up, 60 bare/sparse, 70 snow/ice, 80 water, 90 wetland, 95 mangrove, 100 moss/lichen).
- Granularity: point (10 m pixel). Tile S36W072 covers 33°S–36°S, 69°W–72°W — the full Metropolitan Region is inside this tile.
- Year: 2021
- Access: direct download. Cloud-optimized GeoTIFF on AWS S3 (no sign-in needed). 128 MB per tile.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206, Content-Range /128,020,432 bytes, Last-Modified 2022-10-26. No rows parsed.
- Role: INPUT
- Limits: 10 m Sentinel-2 derived. Urban class (code 50) includes roads and industrial; does not distinguish residential from commercial. No temporal update since 2021. License: CC BY 4.0.
- Use in the simulation: land use mask; urban area extent; green area constraint on housing location.

---

### Copernicus DEM GLO-90 (90 m terrain, tile S33_00_W071_00)

- Link: https://copernicus-dem-90m.s3.amazonaws.com/Copernicus_DSM_COG_30_S33_00_W071_00_DEM/Copernicus_DSM_COG_30_S33_00_W071_00_DEM.tif
- Contents: One pixel = elevation in meters above EGM2008 geoid. This is a Digital Surface Model (DSM): includes buildings and vegetation, NOT bare terrain. Single float32 band.
- Granularity: point (90 m pixel, ~3 arc-sec). Tile covers 33°S–34°S, 71°W–70°W.
- Year: 2019–2020 (TanDEM-X acquisition)
- Access: direct download. Cloud-optimized GeoTIFF on AWS S3 (no sign-in needed). 5.2 MB per tile.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206, Content-Range /5,176,846 bytes, Last-Modified 2022-05-09. No rows parsed.
- Role: INPUT
- Limits: DSM ≠ DTM: buildings and trees raise pixel values. Copernicus DEM GLO-30 (30 m) tiles for Chile are NOT on the public S3 bucket (HTTP 404 for `copernicus-dem-30m`). Use the Copernicus Data Space portal for the 30 m version (account required). License: CC BY 4.0 (GLO-90).
- Use in the simulation: slope and elevation constraint on walkability and air pollution dispersion (optional per accepted scope).

---

### Geofabrik OSM Chile extract

- Link: https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf
- Contents: Full OpenStreetMap extract for Chile. Nodes (points), ways (lines/polygons), relations. Contains: streets (highway tags), buildings, land use areas, amenities, transit stops, cycle paths, administrative boundaries.
- Granularity: point / line / polygon. Updated daily.
- Year: 2026-11-21 (latest build)
- Access: direct download. PBF. 352 MB.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206, Content-Length 352,392,606 bytes, Last-Modified 2026-11-21. No rows parsed (PBF format; use osmium or pyrosm to parse).
- Role: INPUT
- Limits: OSM completeness in Santiago varies by feature type: streets and large buildings near complete; informal settlements and small businesses under-mapped. POIs may be less complete in lower-income comunas. License: ODbL.
- Use in the simulation: street network for routing; transit stop locations; POI catalog; building polygons as supplement to Overture.

---

### Ookla Speedtest Fixed Broadband Tiles Q4 2023

- Link: https://ookla-open-data.s3.amazonaws.com/parquet/performance/type=fixed/year=2023/quarter=4/2023-10-01_performance_fixed_tiles.parquet
- Contents: One row = one Bing map tile (zoom 16, ~600 m at equator). Columns: quadkey, avg_d_kbps, avg_u_kbps, avg_lat_ms, tests (count), devices (count).
- Granularity: zone (~600 m Bing tile at zoom 16). Quarterly.
- Year: 2023 Q4 (Q1–Q3 and other years at same path pattern)
- Access: direct download. Parquet. 361 MB global (all countries in one file; filter Chile using quadkey prefix for lat -33 to -34).
- Check: VERIFIED-BYTES 2026-11-21, HTTP 206, Content-Range /361,363,546 bytes, Last-Modified 2024-01-08. No rows parsed (Parquet format).
- Role: INPUT
- Limits: Only devices that ran the Speedtest app. Skewed toward tech-aware users; rural and low-income areas under-sampled. Mobile tiles at same path with type=mobile (205 MB, also verified VERIFIED-BYTES 2026-11-21). License: CC BY-NC 4.0.
- Use in the simulation: internet quality proxy; influences remote-work probability by zone.

---

### Overture Maps Foundation 2026-10-21.0 (places + transportation)

- Link: https://overturemapswestus2.blob.core.windows.net/release/2026-10-21.0/theme=places/type=place/
- Contents: Places = POIs (name, category, geometry, confidence score, brand). Transportation = road segments and connectors with speed limits and access restrictions. Buildings, divisions, addresses, and base themes also available.
- Granularity: point (places), line (segments). No time dimension.
- Year: Oct 2026 release
- Access: cloud-native Parquet on S3 (s3://overturemaps-us-west-2/release/2026-10-21.0/) and Azure (URL above). Global; use DuckDB spatial filter to clip to Santiago bounding box (-34.0, -71.5, -33.0, -70.0). No sign-in needed.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200 on Azure folder, Last-Modified 2026-10-21. No rows parsed (Parquet files not downloaded).
- Role: INPUT
- Limits: Places sourced from Meta, TomTom, and others; duplicates exist. Transportation is beta (schema may change). No visitor counts. No temporal data on POI open/close hours. License: ODbL.
- Use in the simulation: POI catalog for activity destinations; road network; building footprints (buildings theme, coordinates with clever-llama scope).

---

### VIIRS Night Lights Annual V2.2 (2022)

- Link: https://eogdata.mines.edu/nighttime_light/annual/v22/2022/
- Contents: Annual cloud-free average radiance composite. Vcm masked version excludes stray-light artifacts. Single float32 band (nanoWatts/cm2/sr). Global coverage 65°N–75°S.
- Granularity: point (15 arc-sec = ~500 m at equator). Annual.
- Year: 2012–2024 (annual composites available)
- Access: direct download after free account registration at eogdata.mines.edu. GeoTIFF. ~5 GB global per year.
- Check: UNVERIFIED — HTTP 302 redirect to authentication portal (eogauth.mines.edu). Free account registration required; no token obtained in this session.
- Role: INPUT
- Limits: Overglow from bright areas inflates values in adjacent dark pixels. No discrimination between street lights and residential windows. EOG account required. License: free for non-commercial and research use.
- Use in the simulation: economic activity proxy by zone; validate population distribution against nocturnal activity.

---

### Dynamic World (Google/WRI, 10 m near-real-time land cover)

- Link: https://developers.google.com/earth-engine/datasets/catalog/GOOGLE_DYNAMICWORLD_V1
- Contents: Per-Sentinel-2 image, 9 land cover class probabilities + discrete label per 10 m pixel. Near real-time. Classes: water, trees, grass, flooded veg, crops, shrub, built, bare, snow/ice.
- Granularity: point (10 m pixel). Near real-time (each Sentinel-2 pass ~5 days).
- Year: 2015–present
- Access: Google Earth Engine only (requires GEE account). No direct GeoTIFF download. Export to Google Cloud Storage or Drive via GEE Task.
- Check: UNVERIFIED — no direct download URL exists; GEE access not tested in this session.
- Role: INPUT
- Limits: GEE account needed. For a static 2023 composite, run a GEE script to median-composite a year of images, then export. ESA WorldCover (above, verified) covers the same use case with a direct download and no account. License: CC BY 4.0.
- Use in the simulation: optional update to WorldCover 2021 built-up mask for 2023.

---

### OpenCelliD (cell tower database, MCC 730 = Chile)

- Link: https://opencellid.org/downloads (requires free account)
- Contents: One row = one cell tower. Columns: radio (GSM/UMTS/LTE/NR), mcc, net, area, cell, lat, lon, range, samples, changeable, created, updated, averageSignal.
- Granularity: point. Updated continuously.
- Year: 2024 (latest snapshot)
- Access: direct download after free account registration at opencellid.org. Per-country CSV filtered by MCC 730. Full database ~105 MB gzip.
- Check: UNVERIFIED — download page requires account login; no token obtained. Alternative: government of Chile publishes a cell tower layer at datos.gob.cl (not checked in this session).
- Role: INPUT
- Limits: Crowd-sourced; coverage density correlates with smartphone user density, so informal areas under-represented. License: CC BY-SA 4.0.
- Use in the simulation: mobile coverage proxy; needed only if telco mobility data gaps must be identified by zone.

---

### Meta High Resolution Settlement Layer (HRSL) Chile 2020

- Link: https://data.humdata.org/dataset/dd47e052-02cc-4a3f-972a-421d600b3d85/resource/bb560451-9c50-4d57-8ff3-872fa260c102/download/chl_general_2020_csv.zip (redirects from HDX to signed S3)
- Contents: One row = one 30 m cell with people. Columns: longitude, latitude, chl_general_2020 (count of persons). Derived from Maxar satellite imagery + census adjustment. 2020 edition.
- Granularity: point (30 m pixel). Static 2020.
- Year: 2020
- Access: direct download via HDX. ZIP of CSV. 26 MB compressed.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 302 then 200, 26,010,426 bytes confirmed. Rows read by calm-otter (critic): 5,750,979 total rows; Santiago box (-33.75 to -33.25 lat, -70.95 to -70.40 lon) = 623,158 cells, sum 7,657,984 persons. Columns: longitude, latitude, chl_general_2020.
- Role: INPUT
- Limits: 30 m but population is estimated from land cover, not direct census. The AWS S3 `dataforgood-fb-data` bucket path `csv/month=2019-07/country=CHL` returns empty (KeyCount=0); use HDX URL above. License: CC BY 4.0.
- Use in the simulation: best-resolution home location probability surface (3× finer than WorldPop 100 m). Use for intra-block population distribution.

---

### Meta Relative Wealth Index (RWI) — NOT AVAILABLE for Chile

- Link: https://ai.meta.com/ai-for-good/datasets/relative-wealth-index/
- Contents: Would be: 2.4 km grid of relative wealth score per micro-region (CSV with lat, lon, rwi, error).
- Granularity: point (2.4 km)
- Year: 2021
- Access: HDX or Meta download page
- Check: UNVERIFIED — HDX resource redirect returned HTTP 404. Chile is excluded: RWI covers low-and-middle-income countries (~93); Chile is classified as high-income by World Bank.
- Role: NONE
- Limits: Dataset does not cover Chile.
- Use in the simulation: NONE

---

## Critique

1. **No entry has rows read from the Santiago bounding box.** All verified entries are VERIFIED-BYTES (HEAD or range requests). No pixel value or hexagon was parsed to confirm that Santiago totals near 7 million. Add a row-level check for at least HRSL and WorldPop. The HRSL Santiago-box sum of 7,657,984 (read by critic calm-otter) is the first real population number; it should be added to this catalog as a cross-check value.

2. **GHSL global ZIPs are 5.1 GB and 7.9 GB; tiles not extracted.** The simulation team needs to identify tiles R??_C?? in the tiles/ subfolder covering Santiago in Mollweide coordinates before ingestion. Use an online Mollweide converter for -33.45°S, -70.67°W to get the tile index.

3. **Several entries have low simulation value.** Ookla Speedtest (remote-work influence), Copernicus DEM (slope), OpenCelliD (mobile coverage), and Dynamic World (duplicate of WorldCover) have no clear pitch moment in the current scope. VIIRS also unverified (account required). Census 2024 manzanas (a local source) give real persons and dwellings by block; the rasters here are fallbacks if the census is unavailable or needs spatial disaggregation below manzana level.
