# Environment data sources for Santiago

Topic: environment data that changes how people move or where they live.
Scope: air quality alerts, car restriction days, rain, heat, noise, trees, green area access.
Out of scope: physics simulation, terrain, flood zones, wildfire, seismic hazard (see NONE entries).

---

### SINCA air quality monitoring network
- Link: https://sinca.mma.gob.cl/
- Contents: hourly concentrations of PM2.5, PM10, O3, NO2, SO2, CO per station. One row is one hour at one station. Columns: date, hour, validated value, preliminary value, unvalidated value.
- Granularity: point (station). Time step: 1 hour. About 11 stations in Región Metropolitana (Las Condes, Independencia, Pudahuel, Cerrillos, El Bosque, La Florida, Puente Alto, Quilicura, Talagante, Parque O'Higgins, Cerro Navia).
- Year: 1997 to present (varies by station; most RM stations from ~2000).
- Access: reverse-engineered CGI endpoint (no documented API). Python library `atmchile` (PyPI) and R library `AtmChile` (CRAN) automate downloads. URL pattern: `sinca.mma.gob.cl/cgi-bin/APUB-MMA/apub.tsindico2.cgi?outtype=xcl&...&stId=<id>&pId=<param>&tStep=60&fromDate=YYMMDD&toDate=YYMMDD`. Format: CSV with semicolon separator and comma decimal. Station list JSON at `/index.php/json/listadomapa2k19/`.
- Check: UNVERIFIED by this agent (HTTP 403 to curl on all endpoints). However, calm-puma (Vicente Codex session, PR #6 head c02afd1) independently verified SINCA: HTTP 200 text export, 7 daily PM2.5 rows for Cerrillos II station, Nov 12-18 2026. All preliminary; timezone unspecified. The URL path that worked for them differs from the CGI endpoint I tried.
- Limits: no documented public API; endpoints are reverse-engineered from the Airviro platform and may change. 16% of communes have a PM2.5 station (USS 2022 report). Station placement favors populated areas.
- Use in the simulation: INPUT. Daily air quality determines preemergencia/alerta days, which trigger car restrictions (4 digits instead of 2), close schools, reduce outdoor activity. The simulation needs daily PM2.5 at each station to decide which days are restricted and how movement patterns change.

### Episodios críticos de gestión del aire (GEC reports)
- Link: https://airerm.mma.gob.cl/wp-content/uploads/2025/12/INFORME-GEC-2025_v1.pdf (one report per year; 2015 to 2025 confirmed)
- Contents: annual summary of critical air episodes in the Región Metropolitana. One row is one episode day. Columns: date, type (alerta, preemergencia, emergencia), PM2.5 level, station, confirmed/forecast. Also total hours in episodes per year.
- Granularity: region (Región Metropolitana as a whole). Time step: 1 day. Covers May to August each year.
- Year: 2015 to 2025 (annual reports). The 2026 season balance is in press coverage but no PDF found yet.
- Access: direct download. PDF. Each year is a separate file at airerm.mma.gob.cl. Size: ~1-5 MB per report.
- Check: VERIFIED 2026-11-21, HTTP 206, application/pdf, 1024+ bytes (2025 report). Also checked 2024 and 2022 reports: same pattern.
- Limits: PDF format, not machine-readable. Would need to parse tables from PDF. Only covers May-August. No CSV series found. Declared vs. confirmed episodes differ in count.
- Use in the simulation: INPUT. The specific dates and types of episodes (alerta, preemergencia, emergencia) are scenario inputs that trigger restriction rules. The simulation does not predict air quality; it takes the episode calendar as given and simulates how movement changes in response.

### Restricción vehicular (car restriction calendar)
- Link: https://www.diariooficial.interior.gob.cl/publicaciones/2026/04/30/44438/01/2805129.pdf (2026 decree)
- Contents: the annual decree sets which last-digit pairs cannot drive on each weekday during May-August. Applies to non-catalytic (green-seal) cars only; catalytic (blue-seal), electric, and exempt vehicles (disability, emergency) are not restricted on normal days. For green-seal cars: Mon 8-9, Tue 0-1, Wed 2-3, Thu 4-5, Fri 6-7. On preemergencia days, 4 digits are restricted and catalytic cars registered before 2011-09-01 also join. On emergencia days, 6 digits plus more catalytic vehicles. Hours: 07:30-21:00. Area: Provincia de Santiago plus Puente Alto and San Bernardo.
- Granularity: zone (the restriction area). Time step: 1 day.
- Year: published annually since ~1990. The 2026 decree is Resolución Exenta from the Seremi de Transportes.
- Access: direct download. PDF from Diario Oficial. Can be encoded as a simple lookup table (day-of-week × plate-digit → restricted yes/no).
- Check: VERIFIED 2026-11-21, HTTP 200 for Diario Oficial PDF. News sources confirm the 2026 calendar.
- Limits: no structured dataset. Must be parsed from the decree or news. Changes yearly. Separate calendar for trucks/buses/motorcycles. The simulation needs to combine this with GEC episode type to get the full restriction for each day.
- Use in the simulation: INPUT. On each simulated day, the restriction removes a share of cars that depends on fleet age composition (green-seal share, pre-2011 catalytic share) and episode type. The simulation must encode vehicle eligibility rules, not assume a fixed percentage. Restricted drivers switch to transit or stay home.

### DMC weather stations (Dirección Meteorológica de Chile)
- Link: https://climatologia.meteochile.gob.cl/application/informacion/fichaDeEstacion/330020 (Quinta Normal)
- Contents: hourly and daily temperature, precipitation, wind speed/direction, humidity, pressure, solar radiation. One row is one observation (hourly or daily). Key RM stations: Quinta Normal (330020, since 1849), Pudahuel (330021, since 1966), Eulogio Sánchez/Tobalaba (330019).
- Granularity: point (station). Time step: hourly (automatic stations), daily (manual stations).
- Year: Quinta Normal since 1849 (oldest in Chile). Automatic stations from ~2000.
- Access: form or request. The portal has product pages (e.g., RE8011 for automatic station hourly data). Some products are free for academic use; others require payment. No public bulk download. A JSON endpoint exists for recent 12-hour minutely data but returned HTML when I tried it.
- Check: VERIFIED 2026-11-21, HTTP 200, 480 KB for Quinta Normal station page. Station metadata confirmed: lat -33.445, lon -70.683, altitude 520 m.
- Limits: no bulk download API. Must request data products through the portal, which may involve fees or waiting. The atmchile library can also query DMC, but I did not test this path. Only 3 RM stations found in the catalog.
- Use in the simulation: INPUT. Daily rain triggers mode-choice changes (fewer cyclists, more transit demand). Temperature extremes change outdoor activity patterns. Wind speed affects PM2.5 dispersion and therefore episode probability.

### CR2 Explorador Climático
- Link: https://explorador.cr2.cl/
- Contents: daily precipitation, temperature (max, min, mean), and streamflow from DMC and DGA stations. Web tool for visualization and download. National coverage since ~1950.
- Granularity: point (station). Time step: daily.
- Year: 1950 to present (varies by station).
- Access: direct download through web tool. Free. The CR2 also publishes gridded products and the Mawün tool for point extraction.
- Check: VERIFIED 2026-11-21, HTTP 200, 23 KB for the main page. The portal is live.
- Limits: station data, not gridded for Santiago at fine resolution. The web tool may not support bulk programmatic download (no documented API). Gridded products exist but were not tested.
- Use in the simulation: INPUT. Historical daily rain and temperature feed the weather module. The simulation can replay real weather to match GEC episodes and mode-choice shifts.

### CR2 PM2.5 dichotomous data for Santiago
- Link: https://www.cr2.cl/wp-content/uploads/2018/08/MP25-DICOTOMICO-1998-2014.xlsx
- Contents: daily dichotomous PM2.5 measurements at Santiago stations. One row is one day at one station. Data was delivered to CR2 by MMA officials.
- Granularity: point (station). Time step: daily.
- Year: 1998 to 2014.
- Access: direct download. XLSX file.
- Check: VERIFIED 2026-11-21, HTTP 206, content-type application/vnd.openxmlformats-officedocument.spreadsheetml.sheet, 4096+ bytes.
- Limits: ends in 2014. Only covers PM2.5 dichotomous method (not continuous TEOM). For 2015+ data, use SINCA. This is a curated academic dataset, not an official MMA product.
- Use in the simulation: INPUT. Historical daily PM2.5 for calibrating the relationship between weather and air quality in Santiago before 2015.

### Mapa de Ruido del Gran Santiago (MMA)
- Link: https://ruido.mma.gob.cl/mapas-de-ruido/
- Contents: modeled road-traffic noise levels (Lden day-evening-night and Ln night) for the Gran Santiago urban area. The model uses CadnaA software with traffic counts and street geometry. Three editions: 2011, 2016, 2023.
- Granularity: grid (10 m × 10 m modeled surface). Coverage: 34 comunas of Gran Santiago.
- Year: 2011, 2016, 2023.
- Access: direct download. The page offers two ZIP files. The 2024 ZIP (Mapa-de-Ruido-GSU-2023-MMA-2024.zip, 209 MB) contains a single PDF, not GIS layers. The 2023 ZIP is also 209 MB, likely another PDF. The visor web page has an iframe at `ruido.mma.gob.cl/wp-content/mapas/gsu_2024/` for interactive viewing.
- Check: VERIFIED 2026-11-21, HTTP 206, application/zip, 209 MB. Downloaded and inspected: contains `Mapa de Ruido GSU 2023 [MMA 2024].pdf` (219 MB). No shapefile or GeoTIFF in the ZIP.
- Limits: the downloadable files are PDFs, not GIS data. To get raster or vector layers, one would need to scrape the web visor or request the data from the MMA noise department. The map covers only road-traffic noise (not construction, nightlife, or aircraft). It is modeled, not measured.
- Use in the simulation: INPUT. Noise levels affect residential desirability. High-noise streets have lower housing demand. The simulation can use the noise grid as a penalty factor for residential location choice.

### SIEDU / SIET green area access indicators
- Link: https://www.siet-chile.cl/ (successor to siedu.ine.cl since December 2025)
- Contents: indicators of proximity to parks and plazas by manzana. Key indicators: distance to nearest park (≥20,000 m²), distance to nearest plaza (450-19,999 m²), percentage of population within acceptable distance. Measured by INE, MINVU, and CNDU.
- Granularity: manzana. Coverage: all urban areas of Chile.
- Year: first measurement 2019, updated periodically (5th measurement manual exists). Now integrated into SIET with 27 indicators (expanding to 92).
- Access: web platform. siedu.ine.cl returned SSL error; siet-chile.cl returned HTTP 200. An ArcGIS Open Data portal at geoine-ine-chile.opendata.arcgis.com may have downloadable layers, but this was not verified.
- Check: UNVERIFIED. siet-chile.cl returned HTTP 200 (300 KB), but I did not download actual indicator data or confirm GIS layer availability. siedu.ine.cl gave SSL error.
- Limits: indicator data, not raw park polygons. Manzana-level data may require the ArcGIS platform. The transition from SIEDU to SIET may have broken old download links.
- Use in the simulation: INPUT. Green area access is a factor in residential location choice. Manzanas far from parks have lower residential attractiveness. The simulation can use distance-to-park as a utility component in the housing choice model.

### MapBiomas Chile (land cover)
- Link: https://chile.mapbiomas.org/ (platform: plataforma.chile.mapbiomas.org)
- Contents: annual land cover maps from Landsat imagery. Classes include forest, scrub, grassland, cropland, urban, water, bare soil. 30 m resolution. Produced by Universidad de Chile and partners.
- Granularity: pixel (30 m). Coverage: all of Chile.
- Year: Collection 2.0 covers 1999-2024 (annual).
- Access: direct download from the platform. License: Creative Commons CC-BY. Format: GeoTIFF raster.
- Check: VERIFIED 2026-11-21. plataforma.chile.mapbiomas.org returned HTTP 302 (redirect, platform active). chile.mapbiomas.org returned HTTP 200.
- Limits: 30 m resolution makes individual parks hard to distinguish from surrounding vegetation. The classification is for natural land cover, not specifically urban green spaces. Santiago is at the edge between scrub and urban classes. No per-pixel accuracy assessment for Santiago was found.
- Use in the simulation: INPUT. Annual land cover change tracks urban expansion and loss of green areas over 25 years. The simulation can use the vegetation fraction as a proxy for neighborhood greenness in the residential attractiveness model.

### ARClim (Atlas de Riesgo Climático)
- Link: https://arclim.mma.gob.cl/
- Contents: climate projections and risk indicators at the commune level. Built by CR2 and MMA. Includes projected temperature change, precipitation change, heat-wave frequency, and compound risk indices for cities, health, agriculture, and other sectors.
- Granularity: comuna. Time step: 30-year climate periods (historical, mid-century, end-century).
- Year: historical baseline (~1980-2010) and projections to 2070.
- Access: web platform with download. The portal has an API endpoint at `/atlas/api/` (returned HTML, needs JavaScript). Documentation PDFs available at arclim.mma.gob.cl/media/learning/.
- Check: VERIFIED 2026-11-21, HTTP 200, 112 KB for main page. The API endpoint requires a browser; curl returns HTML.
- Limits: commune-level granularity is too coarse for intra-city differences. Projections, not observations. Useful for long-term scenario testing, not for day-to-day simulation of current Santiago.
- Use in the simulation: NONE for the MVP. Could feed a "Santiago in 2050" scenario mode, but the hackathon demo shows current urban dynamics.

### Copernicus DEM / SRTM terrain elevation
- Use in the simulation: NONE. Terrain elevation is a physical property. It does not change how people move in the urban context unless combined with flood or landslide models, which are out of scope.

### SENAPRED flood zones and wildfire exposure
- Use in the simulation: NONE. Physical hazard simulation is out of scope per the accepted decision (20261121-1200-urban-dynamics-engine-scope.md).

### Landsat land surface temperature (urban heat island)
- Link: https://earthexplorer.usgs.gov/ (Landsat Collection 2 Level-2 ST product)
- Contents: land surface temperature (LST) derived from Landsat thermal bands. One scene per 16-day revisit cycle. Multiple academic studies cover Santiago: Ortega 2018 (2001-2017), MDPI Remote Sensing 2020 (summers 2005-2017), Sarricolea & Martín-Vide 2014 (MODIS-based).
- Granularity: pixel (100 m thermal, resampled to 30 m). Coverage: Landsat path/row covering Santiago.
- Year: Landsat 5 (1984-2011), Landsat 7 (1999-present), Landsat 8 (2013-present), Landsat 9 (2021-present).
- Access: direct download from USGS EarthExplorer or via Google Earth Engine. Free. GeoTIFF format.
- Check: UNVERIFIED. I did not download a Landsat scene in this session. The portal and Earth Engine are well-known public platforms. Academic papers confirm the data covers Santiago.
- Limits: 16-day revisit means no daily coverage. Cloud cover reduces usable scenes (especially in winter). LST is surface temperature, not air temperature. Converting to a "heat comfort" index needs additional meteorological data.
- Use in the simulation: INPUT (secondary). The urban heat island map identifies hot neighborhoods (industrial areas, concrete plazas) vs. cool ones (parks, tree-lined streets). This can modulate outdoor activity and residential desirability in summer months. Not essential for the MVP.

---

## Critique

1. **SINCA is the most important source and I could not verify it.** The portal blocks all curl access. The Python library claims to work but I did not install or run it. This is the biggest gap: without hourly air quality data, the simulation cannot decide which days are preemergencia.

2. **No structured dataset for the car restriction calendar.** The decree is a PDF. Someone must encode it as a lookup table by hand or by parsing. The same applies to the GEC episode reports. Both are small enough to encode manually, but neither is machine-ready.

3. **Noise map has no downloadable GIS layers.** The ZIP is a PDF, not a shapefile or raster. The web visor shows the data, but I did not find a WMS/WFS endpoint to query it. Without the raster, the simulation cannot use noise as a spatial input.
