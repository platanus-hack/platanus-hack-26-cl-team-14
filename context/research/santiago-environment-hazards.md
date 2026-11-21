# Santiago: weather, air quality, heat, and optional hazards

Checked: 2026-11-21. Scope: public data for the Santiago urban simulation.
The catalog goal is research. It does not approve a new simulation mechanism.
Weather, heat, and air quality can supply candidate inputs for social activity research.
Terrain and physical hazards are optional context, outside the required urban dynamics engine scope.

## Source catalog

“Page checked” means the publisher page was read. It does not mean a data file was downloaded.
Coverage dates describe the product, not a guarantee of complete records for Santiago.

| Source and direct link | What it contains | Spatial detail | Time and update | Access and status | Limits |
| --- | --- | --- | --- | --- | --- |
| [MMA SINCA](https://sinca.mma.gob.cl/) | Measured air pollution; PM10, PM2.5, ozone and other pollutants, depending on station. | Monitoring station points. Select the Metropolitana/MACAM network. | Online display updates hourly. Historical coverage varies by station and pollutant; exact ranges were not checked. | Public portal. Page and one daily text-export sample checked; see evidence below. | Online values are preliminary. The display uses different averaging windows for different pollutants. Station measurements are not a street-level pollution map. |
| [DMC Servicios Climáticos catalog](https://climatologia.meteochile.gob.cl/application/index/busqueda) | Temperature, precipitation, humidity, wind, and solar radiation products. | Station points. The portal includes a station catalog and element availability. | Hourly, daily, monthly and historical products; dates depend on station and element. | Public portal. Catalog and one public monthly report sample checked. Product RE3017 advertises automatic-station CSV downloads; that export was not tested. | RE3017 data have no quality control. Daily precipitation uses an observation day ending at 12:00 UTC, not local midnight. |
| [CR2MET v2.5](https://www.cr2.cl/datos-productos-grillados/) | Daily precipitation and maximum/minimum near-surface temperature. | Regular 0.05-degree grid across continental Chile. | Fixed v2.5 archive: 1960–2021. | Publisher page checked. NetCDF downloads are listed; the archive was not downloaded. | Statistical estimates calibrated against observations. The grid cannot resolve individual Santiago blocks. This fixed archive is not a current-weather feed. |
| [USGS Landsat Collection 2 surface temperature](https://www.usgs.gov/landsat-missions/landsat-collection-2-surface-temperature) | Satellite estimates of land surface temperature for heat patterns. | Scene rasters; [Landsat 4–5 products](https://www.usgs.gov/centers/eros/science/usgs-eros-archive-landsat-archives-landsat-4-5-tm-collection-2-level-2-science) have 30 m output pixels. | Mission-dependent archive from 1982; Landsat 8 from April 2013 and Landsat 9 from February 2022. | USGS pages checked through web access after AX returned HTTP 403. Products are available through EarthExplorer at no cost; no scene was downloaded. | Surface temperature differs from air temperature. Cloud, shadow and emissivity gaps affect retrievals. A 30 m output pixel does not establish 30 m independent thermal detail. |
| [MMA ARClim](https://arclim.mma.gob.cl/atlas/index/) | Climate threats, exposure, sensitivity and relative risk across sectors. | National maps at commune level, or points for some impact chains. | Recent-past simulations: 1980–2010. Mid-century scenario: 2035–2065, RCP8.5. | Public pages checked. The [urban commune repository](https://arclim.mma.gob.cl/features/datos_climaticos/comunas_centros_urbanos/) lists Excel, GeoJSON and geometry downloads. Files were not downloaded. | Relative risk is not event probability. Different impact chains cannot be added. Future climate is a scenario projection, not a forecast. Physical-hazard chains remain optional. |
| [Copernicus DEM GLO-30](https://doi.org/10.5270/esa-c5d3d65) (optional geography) | Elevation surface, including buildings and vegetation. | Global 30 m grid; also GLO-90 at 90 m. | Main TanDEM-X acquisition: 2011–2015. Older data fill some gaps. | Publisher page checked. Registered general-public users can download GLO-30 and GLO-90 at no cost. No tile was downloaded. | This is a surface model, not bare-earth terrain. It does not supply drainage capacity, street flood depths, or current building geometry. |
| [SENAPRED Chile Preparado](https://www.senapred.cl/visor-chile-preparado/) (optional hazards) | Exposure viewer for wildfire, volcanic and tsunami threats; evacuation and supporting geographic information. | Location-based maps; source scale varies. | Layer-specific dates and update cycles were not established. | Official description checked. It links to an interactive viewer. Bulk geometry access was not established. | This is exposure information, not an observed incident catalog. The published description does not establish a Santiago pluvial-flood layer. |

## Simulation use and evidence

SINCA and DMC are **SAMPLE VERIFIED** for the bounded public records below. Full archives and ingestion remain unverified.
The other 5 source families are **UNVERIFIED for data rows**; only publisher pages were read.
SINCA, DMC, CR2MET and Landsat are candidate **INPUT** sources for environmental conditions.
Their effect on travel or outdoor activity requires separate behavioral evidence before use in the engine.
ARClim is a candidate scenario INPUT only if the research question needs climate assumptions.
Copernicus terrain and SENAPRED hazards have current simulation use **NONE** under the accepted scope.
None of these sources supplies an observed social-activity **TARGET** for the required engine score.

## Public row samples

Retrieved with AX on **2026-11-21 at 05:05:46 UTC**. Both responses were public HTTP **200**.
These checks verify the stated rows only. They do not verify archive completeness, stable export contracts, or engine suitability.
Raw sample artifacts remain in /tmp; no data files are added to this repository.

### SINCA: Cerrillos II, PM2.5 daily records

The [station page](https://sinca.mma.gob.cl/index.php/estacion/index/id/290) identifies station 290 in Cerrillos, Metropolitana.
Its historical graph opens CGI frames. The right frame JavaScript builds the public Texto export GET.
The [exact text response](https://sinca.mma.gob.cl/cgi-bin/APUB-MMA/apub.tsindico2.cgi?outtype=txt&macro=./RM/D35/Cal/PM25//PM25.diario.diario.ic&from=261112&to=261119&path=/usr/airviro/data/CONAMA/&lang=esp&rsrc=&macropath=) returned text/plain without login.
The response labels the product “MP 2,5 - registro diario”, with a one-day database resolution.
Its 7 data rows cover **2026-11-12 through 2026-11-18**, despite the requested upper bound of 2026-11-19.
Columns are date (YYMMDD), time (HHMM), validated, preliminary, and non-validated values; a trailing comma is present.
The header names the three value series “Registros validados”, “Registros preliminares”, and “Registros no validados”, in µg/m³.

| Date | Time field | Validated | Preliminary PM2.5 | Non-validated |
| --- | --- | --- | --- | --- |
| 2026-11-12 | 0000 | empty | 16 µg/m³ | empty |
| 2026-11-14 | 0000 | empty | 10 µg/m³ | empty |

All 7 returned values are in the preliminary series; validated and non-validated series have n=0.
The export does not establish the time zone of 0000. Do not infer one.
This daily historical series is distinct from the portal online rolling 24-hour display.
The online PM10/PM2.5 display requires at least 18 hourly observations (75%) and updates hourly.

### DMC: Quinta Normal daily air temperature, January 2025

The [public monthly report](https://climatologia.meteochile.gob.cl/application/mensual/temperaturaMediaMensual/330020/2025/1) returned text/html without login.
Its heading identifies **Quinta Normal, Santiago (330020), January 2025**, with temperature in °C.
The main table has **31 daily rows**, plus monthly summary rows; other tables show ten-day summaries.
Columns are Día; T. Mínima (Valor, Hora Local); T. Máxima (Valor, Hora Local); Media (Climatológica); Media (Aritmética); Datos.
On **2025-01-01**, minimum temperature is **14.2 °C at 06:32 local**, maximum is **31.7 °C at 16:21 local**.
Both means are 23.1 °C and Datos is 24. The report exposes no per-row validation flag or definition of Datos.
Quality state is **not supplied in this retrieved report**; the RE3017 no-quality-control warning must not be assigned to this different product.

The catalog route is RE1005 → select station, year and month → monthly report.
The selection form posts to /application/requerimiento/procesaFormularioEstacionAnoMes/RE1005 and includes a session token.
The public report URL above can be read directly; no form submission, token reuse, login, or CSV export was tested.

## Details that affect use

### Air quality

The [SINCA regional description](https://sinca.mma.gob.cl/index.php/region/info/id/M) describes the Santiago monitoring network.
Its background statistics are old. Use station metadata for coverage; do not reuse those population or economic figures.
SINCA reports online PM10 and PM2.5 as rolling 24-hour means, subject to data completeness.
Keep the measurement time, averaging period, units and validation state with each value.

### Weather and urban heat

Use measured DMC station records when the research needs observed weather on a given day.
Use CR2MET for a consistent historical regional grid. Keep the v2.5 version and dates explicit.
ARClim describes scenario changes. It cannot replace observations for a historical simulation day.
Use Landsat to compare surface heat patterns on selected clear days.
A claim about indoor heat, human exposure or changed travel behavior needs separate evidence.

### Optional flood and wildfire context

ARClim includes flood and wildfire impact chains, but these do not approve physical-hazard simulation work.
SENAPRED's published viewer description confirms wildfire exposure, not a complete event history or flood-depth raster.
No public, ready-to-use Santiago street-level flood-depth dataset was verified in this pass.
No wildfire perimeter download or dated CONAF incident archive was verified.
Keep these as research gaps until the pitch or engine scope needs them.

## Suggested first use

1. Select the simulation date and area before downloading data.
2. Check DMC station coverage and quality flags for that date.
3. Check nearby SINCA stations with the same time window.
4. Add historical CR2MET or Landsat only if those spatial patterns answer a defined research question.
5. Keep ARClim scenarios and optional hazard layers separate from historical observations.

## Verification limits

All linked publisher pages were opened in this research session.
Only the bounded SINCA text sample and DMC HTML report above were sampled. No bulk downloads, authenticated exports, raster processing, or station completeness checks were performed.
Data licenses and attribution requirements must be checked for the selected files before redistribution.
No application code or tests were added.
