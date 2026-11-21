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
| [MMA SINCA](https://sinca.mma.gob.cl/) | Measured air pollution; PM10, PM2.5, ozone and other pollutants, depending on station. | Monitoring station points. Select the Metropolitana/MACAM network. | Online display updates hourly. Historical coverage varies by station and pollutant; exact ranges were not checked. | Public portal. Page checked; historical export not checked. | Online values are preliminary. The display uses different averaging windows for different pollutants. Station measurements are not a street-level pollution map. |
| [DMC Servicios Climáticos catalog](https://climatologia.meteochile.gob.cl/application/index/busqueda) | Temperature, precipitation, humidity, wind, and solar radiation products. | Station points. The portal includes a station catalog and element availability. | Hourly, daily, monthly and historical products; dates depend on station and element. | Public portal. Catalog checked. Product RE3017 advertises automatic-station CSV downloads; no station export was completed. | RE3017 data have no quality control. Daily precipitation uses an observation day ending at 12:00 UTC, not local midnight. |
| [CR2MET v2.5](https://www.cr2.cl/datos-productos-grillados/) | Daily precipitation and maximum/minimum near-surface temperature. | Regular 0.05-degree grid across continental Chile. | Fixed v2.5 archive: 1960–2021. | Publisher page checked. NetCDF downloads are listed; the archive was not downloaded. | Statistical estimates calibrated against observations. The grid cannot resolve individual Santiago blocks. This fixed archive is not a current-weather feed. |
| [USGS Landsat Collection 2 surface temperature](https://www.usgs.gov/landsat-missions/landsat-collection-2-surface-temperature) | Satellite estimates of land surface temperature for heat patterns. | Scene rasters; [Landsat 4–5 products](https://www.usgs.gov/centers/eros/science/usgs-eros-archive-landsat-archives-landsat-4-5-tm-collection-2-level-2-science) have 30 m output pixels. | Mission-dependent archive from 1982; Landsat 8 from April 2013 and Landsat 9 from February 2022. | USGS pages checked through web access after AX returned HTTP 403. Products are available through EarthExplorer at no cost; no scene was downloaded. | Surface temperature differs from air temperature. Cloud, shadow and emissivity gaps affect retrievals. A 30 m output pixel does not establish 30 m independent thermal detail. |
| [MMA ARClim](https://arclim.mma.gob.cl/atlas/index/) | Climate threats, exposure, sensitivity and relative risk across sectors. | National maps at commune level, or points for some impact chains. | Recent-past simulations: 1980–2010. Mid-century scenario: 2035–2065, RCP8.5. | Public pages checked. The [urban commune repository](https://arclim.mma.gob.cl/features/datos_climaticos/comunas_centros_urbanos/) lists Excel, GeoJSON and geometry downloads. Files were not downloaded. | Relative risk is not event probability. Different impact chains cannot be added. Future climate is a scenario projection, not a forecast. Physical-hazard chains remain optional. |
| [Copernicus DEM GLO-30](https://doi.org/10.5270/esa-c5d3d65) (optional geography) | Elevation surface, including buildings and vegetation. | Global 30 m grid; also GLO-90 at 90 m. | Main TanDEM-X acquisition: 2011–2015. Older data fill some gaps. | Publisher page checked. Registered general-public users can download GLO-30 and GLO-90 at no cost. No tile was downloaded. | This is a surface model, not bare-earth terrain. It does not supply drainage capacity, street flood depths, or current building geometry. |
| [SENAPRED Chile Preparado](https://www.senapred.cl/visor-chile-preparado/) (optional hazards) | Exposure viewer for wildfire, volcanic and tsunami threats; evacuation and supporting geographic information. | Location-based maps; source scale varies. | Layer-specific dates and update cycles were not established. | Official description checked. It links to an interactive viewer. Bulk geometry access was not established. | This is exposure information, not an observed incident catalog. The published description does not establish a Santiago pluvial-flood layer. |

## Simulation use and evidence

All 7 source families are **UNVERIFIED for data rows**. Publisher pages were read; no dataset samples were retrieved.
SINCA, DMC, CR2MET and Landsat are candidate **INPUT** sources for environmental conditions.
Their effect on travel or outdoor activity requires separate behavioral evidence before use in the engine.
ARClim is a candidate scenario INPUT only if the research question needs climate assumptions.
Copernicus terrain and SENAPRED hazards have current simulation use **NONE** under the accepted scope.
None of these sources supplies an observed social-activity **TARGET** for the required engine score.

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
No bulk downloads, authenticated exports, raster processing, or station completeness checks were performed.
Data licenses and attribution requirements must be checked for the selected files before redistribution.
No application code or tests were added.
