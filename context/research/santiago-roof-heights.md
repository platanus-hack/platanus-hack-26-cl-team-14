# Santiago roof heights

Google Open Buildings 2.5D Temporal supplies building height and presence rasters for 2023. The model uses Sentinel-2 imagery. Its height values are estimates in metres, capped at 100 m. Google had height reference data only in the Global North. The building-presence score is uncalibrated. The publisher offers CC BY 4.0 or ODbL 1.0. This derived dataset uses ODbL 1.0. Credit Google Research, Open Buildings 2.5D Temporal, and © OpenStreetMap contributors. [Publisher and licences](https://sites.research.google/gr/open-buildings/temporal/) · [Band definitions](https://developers.google.com/earth-engine/datasets/catalog/GOOGLE_Research_open-buildings-temporal_v1)

The builder uses the publisher’s June 30, 2023 manifest for S2 cell 97 in UTM zone 19S. Nineteen source rasters intersect the city bounds. Their complete files total 10,683,126,189 bytes. For each raster the builder reads only the relevant window in the 4 m overview of the height and presence bands. The manifest gives the origin and dimensions. Some rasters overlap. The first qualifying sample in manifest order wins. [Tile manifest](https://storage.googleapis.com/open-buildings-temporal-data/v1/manifests/97_EPSG_32719_2023_06_30.json) · [Publisher’s region download example](https://github.com/google-research/google-research/blob/master/building_detection/open_buildings_temporal_download_region_geotiffs.ipynb)

For each mapped OSM roof, take the 4 m pixel that contains its projected point on surface. The pixel centre must lie inside the roof polygon, at least half the pixel diagonal from every boundary, including holes. This requires the whole pixel to fit inside the mapped roof. Keep samples from 2 to 100 m with a presence score of at least 0.7. The thresholds are model choices. The resulting value describes one part of a roof; the model can miss steps, spires and taller buildings. Generated outlines have no surveyed location and receive no satellite match.

The municipal cadastre gives constructed floor area by land use and census block. It does not give the number of storeys or the height of each building. The Microsoft Global ML Building Footprints tile for Santiago (quadkey 210231031, November 12, 2026 release) has 1,020,125 footprints. Zero contain a positive height. [Municipal field list](https://services9.arcgis.com/zqDAjElD9y8GPDk4/arcgis/rest/services/TOTAL_DESTINOS_M2_CONSTRUIDOS_2021_gdb/FeatureServer/0?f=json) · [Microsoft release](https://github.com/microsoft/GlobalMLBuildingFootprints)

Two mapped building parts supply additional evidence for previously untagged parent outlines. A 42 m part covers 99.458% of Edificio Presidente Philips (parent way 1391091230). Two parts of way 1300156274 each report two floors and together cover 99.429% of its outline. Each group has one matching parent and no conflicting dimensions. The second roof uses the pack’s existing convention of 3.5 m per floor. The builder records the part source separately in both cases. [42 m part](https://api.openstreetmap.org/api/0.6/way/728253207.json) · [First two-storey part](https://api.openstreetmap.org/api/0.6/way/1300156268.json) · [Second two-storey part](https://api.openstreetmap.org/api/0.6/way/1300156273.json)

The operator of Santander Arena reports a 50 m dome. The pack uses this apex height for its single outline and keeps its source flag. The municipal tourism page gives 45 m. The pack follows the operator’s value already used in the landmark layer. [Venue operator](https://www.santanderarena.cl/sobre-nosotros) · [Santiago Turismo](https://www.santiagoturismo.cl/es/movistar-arena/)

## Check against mapped heights

The filter found 75,980 qualifying roof samples across the city. Of these, 665 roofs have a height tag and an assigned census block. On those same 665 roofs, median absolute error against the OSM value was 3.00 m for Google and 4.00 m for the existing density rule. Mean absolute error was 7.42 m and 9.74 m respectively. OSM reporting and the roof size filter select this sample. It does not establish accuracy for all Santiago buildings.

| Tagged roof height | Roofs | Satellite median absolute error | Density median absolute error | Satellite mean absolute error | Density mean absolute error |
| --- | ---: | ---: | ---: | ---: | ---: |
| Below 10 m | 377 | 2.30 m | 2.80 m | 3.12 m | 3.53 m |
| 10 through 30 m | 222 | 5.20 m | 8.65 m | 9.99 m | 10.17 m |
| Above 30 m | 66 | 23.90 m | 35.85 m | 23.33 m | 43.82 m |

The higher buildings have much larger errors. Above 30 m, satellite minus OSM has a median of +21.70 m. The estimate exceeds the tag for 55 of 66 roofs. All five matched roofs tagged above 100 m have smaller satellite estimates. The tallest is tagged 200 m and sampled at 91.2 m. No accepted sample reaches 100 m.

The pack gives priority to existing OSM heights, OSM floors, matching building parts and the arena operator. This leaves 65,857 satellite estimates for previously untagged roofs. The source table includes qualifying tagged roofs so the comparison can be reproduced. It stores the OSM area ID, polygon part index and height in decimetres.

The normal pack command reads the published lookup in `data/mvp/satellite_roof_heights.csv.gz`. To regenerate the lookup from the 2023 imagery, install rasterio, Shapely 2, pyproj, NumPy and certifi. After `building_mesh.py prepare`, run the command below from the repository root. It reads byte ranges from the public Google bucket.

```sh
python3 tools/pack/satellite_roof_heights.py data/raw/building-fill/real_rendered.geojsonseq
```

The lookup is 322,309 gzip bytes. Its SHA-256 is `8591baea7d12d5334df86cffdbe21eb5d34f48ac2ea6fe35ad0775457a1c3ed4`.
