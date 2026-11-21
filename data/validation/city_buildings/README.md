# Building geometry coverage in Santiago

The mesh combines 279,385 mapped OSM building outlines with 375,281 generated outlines. Generate only inside populated Census polygons that have zero intersection with the rendered OSM layer. Even one mapped building, or contact with its boundary, disqualifies the whole Census polygon. Generated positions, shapes and heights represent probable building grain. They do not identify surveyed buildings.

## Coverage

The view extends from −70.85 to −70.45 longitude and −33.65 to −33.30 latitude. The two INE layers intersect the box in 58,467 polygons. Of these, 44,228 have residents in `blocks.csv`. This excludes the 53 Census containers without a polygon.

| Populated polygons | Count |
| --- | ---: |
| With at least one rendered OSM building intersection | 18,756 |
| With no rendered OSM intersection | 25,472 |
| With generated buildings | 25,471 |

One zero-OSM polygon in Padre Hurtado (`13605022006052`) enters the box as a narrow strip of 182 m². No footprint fits inside its 2 m setback. The [comuna table](coverage_by_comuna.csv) records all 43 comunas with populated polygons that intersect the view. Its real-shape count includes both populated and unpopulated blocks. It excludes 43 OSM shapes without a positive-area assignment to INE. Of those, one has a tagged height or floor count. The other 42 receive a separate neutral 3 m height source.

## How the geometry is built

1. Start with the OSM building layer after the packer excludes polygons that cross the view boundary and quantizes coordinates to five decimal places. Use the same exterior rings and holes that the view decodes. Use valid geometry for the spatial comparison (125 source components need a repair).
2. Join all rendered OSM shapes against INE `Manzanas_CPV24` and `Entidades_CPV24` by any geometric intersection in UTM zone 19S. A shape may exclude more than one block from generation. Assign it to exactly one block for height and counts. Prefer a block containing the shape's interior point. Break ties by greatest overlap, then block ID. Require positive overlap for that assignment.
3. Learn a typical footprint from 7,957 populated reference blocks with at least five assigned buildings and summed OSM footprint area between 20% and 90% of block area. The median of block-level median footprint areas is 95.97 m². Median mapped coverage is 30.24%, and the median ratio is 1.48 dwellings per mapped building. These describe the selected reference blocks.
4. Clip each eligible polygon to the view in longitude/latitude before projection. Retain its component polygons and holes. Inset each component by 2 m. The target count is the smaller of `clipped area × 0.3024 / 95.97` and `dwellings × clipped area / full area / 1.48`, rounded to the nearest integer, with a minimum of one. The builder uses the unrounded reference values.
5. Align a grid with the minimum-area rotated rectangle of the inset block. The cell spacing is the square root of inset area divided by target count. An MD5 hash of block ID, component and grid indices determines each centre offset, footprint dimensions and height variation. The builder accepts only rectangles completely inside the inset. A block that cannot fit a grid rectangle gets one smaller rectangle inside its largest inscribed circle if room permits.
6. Round coordinates to 0.00001 degrees. Reject shapes with zero area or coordinates outside the box. The resulting median generated footprint is 88.0 m². The 10th and 90th percentiles are 68.9 and 111.9 m². A spatial check on the final output found zero intersections with rendered OSM and zero shapes outside their source block.

## Heights

For an INE polygon of area `A` m² with `D` dwellings and `J` modeled jobs at the same block ID, estimate height as `3 × max(1, (65D + 20J) / (0.30A))` metres. The display assumes 65 m² per dwelling, 20 m² per job, 30% building coverage, and 3 m per storey. These are display assumptions. Census dwelling counts and modeled jobs remain distinct inputs. The airport has no block ID match.

Tagged OSM height takes priority. The packer uses tagged levels times 3.5 m when height is absent. Cap density estimates at 77 m, the 99th percentile among 23,358 shapes with a height or levels tag. For generated footprints, multiply the block estimate by a deterministic factor from 0.78 to 1.23, then constrain the result to 3–77 m. The median generated height is 3.6 m. The cap applies only to modeled heights.

## Rebuild

Install DuckDB CLI 2.0.0 and its `spatial` extension (`duckdb -c 'INSTALL spatial'`). Run the OSM preparation step from the mesh builder first. Then run from the repository root:

```sh
python3 -I tools/pack/building_fill.py data/raw/verify-people-work/cartography-rm.gpkg data/raw/building-fill/real_rendered.geojsonseq data/raw/building-fill
```

The builder writes the real shape assignments, generated shape stream and coverage table in the raw directory. Only the packed mesh and the comuna coverage table belong in the release. Geometry processing uses DuckDB Spatial and GEOS. [DuckDB Spatial functions](https://duckdb.org/docs/stable/core_extensions/spatial/functions.html) · [INE RM cartography](https://storage.googleapis.com/bktdescargascenso2024/Cartografia/GPKG/Cartografia_censo2024_R13.zip) · [INE Census block data](https://storage.googleapis.com/bktdescargascenso2024/Datos_agregados/Base_manzana_entidad_CPV24.zip) · [© OpenStreetMap contributors](https://www.openstreetmap.org/copyright)
