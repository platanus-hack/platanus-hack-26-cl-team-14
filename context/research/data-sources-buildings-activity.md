# Data sources: building geometry and activity places (Santiago)

Part of the goal "catálogo completo de fuentes de datos urbanos de Santiago" (Discord `#agent-chat` thread).
Owner: clever-llama. Other areas: `santiago-transport.md`, `santiago-people-work.md`, `santiago-housing-land.md`, `santiago-environment-hazards.md`.
"Unverified" means that the research agent did not confirm the link or the access.

## MVP recommendation

- **Blocks:** INE Censo 2024 manzanas (GeoParquet), joined to the manzana table (persons, dwellings).
- **Footprints:** Overture buildings. It merges OSM, Microsoft, and Google footprints.
- **Height:** OSM/Overture `height` or `num_floors` first. Then GlobalBuildingAtlas (predicted height, LoD1). Then Google Open Buildings 2.5D raster, summed for each footprint. Use the Santiago exposure model as ground truth for residential buildings with 5 or more floors.
- **Use, m2, year, floors:** SII Detalle Catastral is the richest source. It needs an SII login, has no geometry, and SII manzana is not INE manzana. INE georeferenced building permits (2011 to now) give units, floors, m2, and use as points for new buildings.
- **Activity places:** Overture Places (points, with confidence) plus OSM polygons for malls, parks, and stadiums. Use Foursquare OS Places only to fill gaps (download is gated).
- **Attractiveness:** mall GLA from operator reports, plus press visit counts (MUT: about 650k visits per month). SII PUB_COMU_RUBR for commercial density by comuna. SIEDU for green areas.
- **Redistribution model:** copy the gravity and co-visitation model of Beiró et al. 2018 (16 Santiago malls). Feed it with home population and EOD 2012 shopping and recreation trips.
- **Do not plan on:** open LiDAR (none found), Google Photorealistic 3D Tiles (visual only, terms block extraction), Bip OD, telco data, ANFP match data, municipal patentes (no verified public download for Gran Santiago).

## Building geometry

| Name | URL | Contains | Granularity | Santiago coverage | Year | Access | Limits |
|---|---|---|---|---|---|---|---|
| OpenStreetMap buildings | openstreetmap.org; download.geofabrik.de/south-america/chile.html | Footprints, optional `building`, `building:levels`, `height`, `amenity` | building | Full, quality varies | live | PBF/SHP, Overpass API; ODbL | Height and levels sparse outside centre and high-rise zones |
| Microsoft Global ML Building Footprints | github.com/microsoft/GlobalMLBuildingFootprints | ML footprints; 2025-01 update added 3.4M Chile footprints (2021–24 imagery) | building | Chile | 2014–2024 imagery | GeoJSONL in csv.gz; CDLA-Permissive 2.0 | No heights for Chile, no use |
| Google Open Buildings v3 | sites.research.google/open-buildings/ | Footprints, area, confidence | building | Chile | 2023 | CSV by S2 cell, Earth Engine; CC BY 4.0 / ODbL | No height or use |
| Google Open Buildings 2.5D Temporal | sites.research.google/gr/open-buildings/temporal; GEE `GOOGLE/Research/open-buildings-temporal/v1` | Rasters of presence, count, height | ~4 m raster | Chile | 2016–2023 yearly | GEE, GCS; CC BY 4.0 / ODbL | Coarse, must be summed onto footprints |
| Overture Maps buildings | docs.overturemaps.org/guides/buildings | Conflated footprints with `height`, `num_floors`, `roof_shape` | building | Global | Monthly | GeoParquet on S3/Azure, DuckDB; ODbL | Height only where a source has it (mostly OSM in Chile) |
| GlobalBuildingAtlas (TUM) | github.com/zhu-xlab/GlobalBuildingAtlas; gee-community-catalog.org/projects/gba/ | 2.68B polygons with predicted height, LoD1 | building | Global, Santiago not checked | 2019 imagery, paper 2025 | GeoJSON/tiles, GEE; license not checked | Height RMSE 1.5–8.9 m |
| 3D-GloBFP | essd.copernicus.org/articles/16/5357/2024/ | Global 3D footprints with heights | building | Global | 2024 | Per paper | Unverified for Santiago |
| Cesium OSM Buildings | cesium.com/platform/cesium-ion/content/cesium-osm-buildings/ | 3D Tiles extruded from OSM | building | Global | Quarterly | Cesium ion token | Same data as OSM, default heights |
| Google Photorealistic 3D Tiles | developers.google.com/maps/documentation/tile/3d-tiles-overview | Photogrammetric mesh | mesh | Chile listed | live | API key, billing, ToS | Visual only, no attributes, no extraction |
| Santiago residential exposure model | aleaicv.github.io/data/exposureModel_ch_2026/ | 9,793 residential buildings ≥5 floors: footprint, height, floors, dwellings, households, inhabitants, year | building | Urban Santiago, mid/high-rise | 2023 | GitHub release; license not stated | Residential ≥5 floors only |
| INE Censo 2024 cartography + manzana table | censo2024.ine.gob.cl/resultados/ | Manzanas, comunas, urban limits, persons and dwellings per manzana | manzana | Full RM | 2024 (pub. Dec 2025) | GeoParquet, EPSG:4674; free | Download path confirmed only through third parties |
| INE Censo 2024 base cartográfica APC 2023 | ine.gob.cl/herramientas/portal-de-mapas/geodatos-abiertos | Pre-census base | manzana | National | 2023 | GDB/SHP | Not final census output |
| INE Censo 2017 cartography | geoine-ine-chile.opendata.arcgis.com | Manzana, zona censal, distrito, comuna, urban limit, streets | manzana / zona / comuna | Full RM | 2017 | SHP/GDB; free | Older |
| INE georeferenced building permits | INE Geodatos Abiertos > Cartografía > Permisos de Edificación | New-build permits: units, floors, use, m2, lat/lon | point | National | 2011–now, half-yearly | SHP; free | Provisional, new works only; exact file not opened |
| SII Detalle Catastral | sii.cl/destacados/transparencia/; spec sii.cl/bbrr/descargas/estructura_detalle_catastral.pdf | Per rol: avalúo, destino, land m2; per construction: material, year, m2, destino, floors | predio (SII manzana) | All RM comunas | Current + historical | Text files per comuna; SII RUT/clave login | No geometry; SII manzana ≠ INE manzana |
| SII Mapas | sii.cl (Cartografía Digital) | Predio lookup | predio | Full | live | Web viewer only | No bulk download |
| SUBDERE / IDE Chile DPA | geoportal.subdere.gov.cl; ide.cl | Comuna, provincia, región polygons | comuna | Full | 2018 | SHP; free | Legal limits differ slightly from INE |
| IDE MINVU / Observatorio Urbano | ide.minvu.cl; geoide.minvu.cl | Planning instruments (PRC/PRMS), programmes | zone | RM | varies | ArcGIS REST/WMS | Unverified for buildings; useful for zoning limits |
| Copernicus DEM GLO-30 | registry.opendata.aws/copernicus-dem/ | 30 m DSM | 30 m raster | Full | 2011–2015 | COG on S3; free | Terrain only |
| Santiago SPOT 2014 point cloud | doi.org/10.5069/G97M064K | Stereo point cloud | 10 m | Santiago bbox | 2014 | OpenTopography; CC BY 4.0 | Too coarse for buildings |
| Open LiDAR | — | — | — | — | — | — | None found |
| Processed repos | github.com/pachadotdev/censo2024-cartografia-postgresql; github.com/bastianolea/censo_2024_mapas | Censo 2024 cartography in PostGIS and R | manzana | Full | 2025 | GitHub | No repo joins SII cadastre to footprints |

## Activity places

| Name | URL | Contains | Granularity | Santiago coverage | Year | Access | Limits |
|---|---|---|---|---|---|---|---|
| OpenStreetMap (Geofabrik) | download.geofabrik.de/south-america/chile.html | amenity, shop, leisure, tourism, malls, parks, stadiums, cinemas, bars | point / polygon | Full | Daily | PBF/SHP/GPKG; ODbL | POI completeness 7–73% by area; poorer comunas less mapped |
| Overture Maps Places | docs.overturemaps.org/guides/places | ~88M POIs, ~2,300 categories, confidence | point | Global | Monthly | GeoParquet, `overturemaps` CLI, DuckDB; CDLA / Apache | Category schema changed in 2026; filter on confidence; no visits |
| Foursquare OS Places | huggingface.co/datasets/foursquare/fsq-os-places | 100M+ POIs, categories, date_closed | point | Global | 2025–2026 | Parquet; Apache 2.0; gated on Hugging Face | No popularity; filter `date_closed IS NULL` |
| SII Estadísticas de Empresas | sii.cl/sobre_el_sii/estadisticas_de_empresas.html | Firms, sales, workers by comuna × activity (PUB_COMU_RUBR, PUB_COMU_ACT) | comuna | All comunas | 2005–2025 | xlsb / TSV zip; free | HQ address, not place of business; small cells masked |
| SII Nómina Personas Jurídicas | sii.cl/sobre_el_sii/nominapersonasjuridicas.html | Firms with rubro, activity, sales bracket, workers | firm (region) | National | 2005–2024 | ZIP/TXT; free | Public comuna field unverified |
| Municipal patentes comerciales | datos.gob.cl (example: dataset cfeb0d8b-635d-47a8-a330-e663b0538f06) | Permits: address, giro, type (incl. alcohol) | address | Unverified for Gran Santiago | to 2024 | CSV/XLSX | May need Transparencia requests; geocoding |
| Mall visit index (Cámara / CNC) | press, e.g. eldinamo.cl 2025-11-21 | National visits and sales index (~300 centers, 600M+ visits/year) | national | No per-mall data | 2025 | Press only | Aggregate |
| Mall operator reports (Mallplaza, Cenco Malls, Parque Arauco) | CMF / investor relations sites | GLA and visits by operator, some per mall | mall | Large malls | 2023–2026 | PDF | MUT (Territoria) only in press: ~650k visits/month, ~16k m² |
| Catastro de Infraestructura Cultural | cultura.gob.cl/redcultura/publicaciones/catastro-de-infraestructura-cultural-publica-y-privada/ | 2,298 cultural spaces | comuna | National | 2017 | PDF | Not machine-readable, old |
| IDE Patrimonio (Mincap) | cultura.gob.cl (IDE Patrimonio) | 2,900+ heritage points: museums, libraries, sites | point | National | Recent | Web viewer; WMS/WFS unverified | Heritage-focused |
| Museum visitor statistics (SNPC) | museoschile.gob.cl (Estadísticas generales 2025 PDF) | Visits per state museum | museum | Many in Santiago | 2020–2025 | PDF | State museums only |
| Estadísticas Culturales (INE + Mincap) | patrimoniocultural.gob.cl (Informe Anual 2024 PDF) | Attendance at shows, theater, museums, cinema | region | RM aggregate | 2024 | PDF | No site-level data |
| Cinema attendance (CAEM / Mincap) | cultura.gob.cl (estudio audiencias cine 2019) | National admissions, complexes per chain | national | Partial | 2019 | PDF | No per-complex counts |
| Football attendance (Estadio Seguro / ANFP) | alairelibre.cl (Boletín Estadio Seguro 2018 PDF) | Attendance by team, % capacity | stadium / team | Santiago clubs | 2018 | PDF | Match-level data unverified |
| Sports facilities registry (IND / MINDEP) | — | 30k+ facilities mentioned (2021) | unknown | National | 2021 | Not found as open data | Use OSM `leisure=*` instead |
| SIEDU green areas (INE + CNDU) | ine.gob.cl/herramientas/portal-de-mapas/siedu | Green m² per person, distance to plazas and parks, ~90 urban indicators | comuna / manzana | 117 comunas incl. Gran Santiago | ~2019–2024 | XLSM + cartography ZIP; free | Read technical note before comparing years |
| MINVU GeoIDE parks | geoide.minvu.cl/server/rest/services/Planes_Programas/ | Urban park layers | polygon | Programme parks only | Recent | ArcGIS REST | Not a full registry |
| Chile Cultura events | chileatiende.gob.cl/fichas/77453-Chile-cultura | Cultural activity listings | event / venue | National | live | Web only | No API verified |
| EOD Santiago 2012 (SECTRA) | sectra.gob.cl (404); copy in github.com/zorzalerrante/aves | 18k households, trips with purpose (shopping, recreation), mode, 866 zones | zona / trip | 45 comunas | 2012–13 | aves repo; license not checked | Old; official download unverified |
| Encuesta de Movilidad Santiago 2024 (CEDEUS) | emol.com 2024-08-22 | EOD update, n=2,217 | comuna | Gran Santiago | 2024 | Microdata unverified | Small sample |
| Bip / DTPM OD matrices | dtpm.cl | Smart-card OD | stop / zone | Public transport | — | No public download found | No car trips (see transport catalog) |
| Telefónica mall-visit study (Beiró et al. 2018) | arxiv.org/abs/1802.00041 | 387k devices, 16 malls, gravity and co-visitation models | mall × home zone | 16 malls | ~2015 | Paper open, data private | Best template for "remove the MUT" |
| Telefónica commuting study | arxiv.org/abs/2507.21743 | XDR commuting (March 2023) | zone | Gran Santiago | 2023 | Paper open, data private | Commuting, not leisure |
| Google Popular Times scrapers | github.com/m-wrzr/populartimes; github.com/philshem/gmaps_popular_times_scraper | Hourly relative busyness | point | Any Google place | live | Python, unofficial | Breaks Google ToS; relative values only |
| IDB urbanpy | github.com/EL-BID/urbanpy | OSM POI download, accessibility | hex / point | Any city | — | Open source | Generic |
