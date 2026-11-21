# Santiago floor area and Census 2024 blocks

The municipal layer covers Santiago (CUT 13101) in 2021. Its 1,458 published polygons overlap 1,414 Census 2024 urban blocks in 1,534 pairs. Match the municipal feature identifier to `land_use.csv` using `area_id`. Match the receiving Census feature using `cut` and `block_id`. Use the Census polygons for this operation. The population file omits some blocks with no residents. [Municipal layer](https://services9.arcgis.com/zqDAjElD9y8GPDk4/arcgis/rest/services/TOTAL_DESTINOS_M2_CONSTRUIDOS_2021_gdb/FeatureServer/0) · [INE regional GeoPackage](https://storage.googleapis.com/bktdescargascenso2024/Cartografia/GPKG/Cartografia_censo2024_R13.zip)

`source_area_share` is the area of the intersection divided by the area of the municipal polygon. Both areas use UTM zone 19 south. Multiplying floor area by this fraction assumes that each use has uniform density within its municipal polygon. Streets and boundary changes leave unmatched area. Coverage per municipal polygon ranges from 71.96% to nearly 100%. The median is 99.99986%. The longitude and latitude mark the centroid of the largest part of the receiving Census block, in degrees.

To rebuild the crosswalk, download the municipal polygons with the command below and unpack the regional Census GeoPackage. Run with Shapely 2.1 and pyproj.

```sh
curl -fLG 'https://services9.arcgis.com/zqDAjElD9y8GPDk4/arcgis/rest/services/TOTAL_DESTINOS_M2_CONSTRUIDOS_2021_gdb/FeatureServer/0/query' \
  --data-urlencode 'f=geojson' --data-urlencode 'where=1=1' \
  --data-urlencode 'outFields=OBJECTID_1' --data-urlencode 'outSR=4326' \
  --data-urlencode 'orderByFields=OBJECTID_1' -o data/raw/santiago2021-shapes.geojson
python3 tools/pack/land_use_blocks.py data/raw/verify-people-work/cartography-rm.gpkg data/raw/santiago2021-shapes.geojson
```

## Candidate job weights

For each source polygon, calculate `office_m2 / (1.175 × 12) + commerce_m2 / (1.175 × 19) + industry_m2 / 36`. Multiply the result by each overlap fraction. Sum by receiving block. The employment guide gives 12 m² net per office full-time equivalent, 19 m² net for high-street retail, and 36 m² gross internal for general industry. We use the midpoint of its 15–20% gross-to-net premium for offices and retail. Applying British employment densities to Chilean cadastral floor areas is a model assumption. [Employment Densities Guide, §§2.8 and 3](https://assets.publishing.service.gov.uk/media/5a7dedd8e5274a2e8ab44baf/employ-den.pdf)

Preserve original jobs outside source coverage, the airport row, and the two site-specific estimates. Allocate the remaining Census employment total within each commune by its covered block weights. Keep original jobs if every weight in the commune is zero. Use the same integer allocation as `tools/pack/jobs.py`. The rule was posted before scoring. The comparison uses original jobs and April-normalized jobs on the same engine build, with a threshold of at least 1,000 fewer error boardings than the incumbent. [Registration](https://discord.com/channels/1573514576400752640/1573688249287708682/1573697362101141544)
