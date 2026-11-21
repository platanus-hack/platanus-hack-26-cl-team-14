# Building geometry coverage in Santiago

The mesh combines 279,385 mapped OSM outlines with 486,694 generated outlines. The builder adds outlines in populated blocks with less than 5% mapped roof cover. Generated positions, shapes and heights describe the model. A flag identifies each generated outline.

## Coverage

The view extends from −70.85 to −70.45 longitude and −33.65 to −33.30 latitude. The two INE layers intersect the box in 58,467 polygons. Of these, 44,228 have residents in `blocks.csv`. This excludes the 53 Census containers without a polygon.

| Populated polygons | Count |
| --- | ---: |
| With at least one rendered OSM building intersection | 18,756 |
| With no rendered OSM intersection | 25,472 |
| With mapped buildings and less than 5% mapped roof cover | 4,196 |
| With new infill in a partly mapped block | 3,877 |
| With generated buildings, including blocks without a mapped roof | 29,348 |

One zero-OSM polygon in Padre Hurtado (`13605022006052`) enters the box as a narrow strip of 182 m². No footprint fits inside its 2 m setback. The [comuna table](coverage_by_comuna.csv) lists all 52 RM comunas. The `real_share` column divides populated polygons with any rendered OSM intersection by the number of populated polygons in the view, rounded to six decimals. Nine comunas have no such populated polygon in the view. Their share is blank. A share of zero means the view contains populated polygons but none touch a mapped building. Its real-shape count includes both populated and unpopulated blocks. It excludes 43 OSM shapes without a positive-area assignment to INE. Of those, one has a tagged height or floor count. The other 42 receive a separate neutral 3 m height source.

## How the geometry is built

1. Start with the OSM building layer after the packer excludes polygons that cross the view boundary and quantizes coordinates to five decimal places. Use the same exterior rings and holes that the view decodes. Use valid geometry for the spatial comparison (125 source components need a repair).
2. Join all rendered OSM shapes against INE `Manzanas_CPV24` and `Entidades_CPV24` by any geometric intersection in UTM zone 19S. A roof can cover parts of multiple blocks. Assign it to exactly one block for height and counts. Prefer a block containing the shape's interior point. Break ties by greatest overlap, then block ID. Require positive overlap for that assignment.
3. Learn a typical footprint from 7,957 populated reference blocks with at least five assigned buildings and summed OSM footprint area between 20% and 90% of block area. The median of block-level median footprint areas is 95.97 m². Median mapped coverage is 30.24%, and the median ratio is 1.48 dwellings per mapped building. These describe the selected reference blocks.
4. Clip each eligible polygon to the view in longitude/latitude before projection. For a partly mapped block, divide the union area of all mapped roofs inside the visible block by its visible area. Use a strict threshold of 5%. Subtract the mapped roofs from each component, preserve holes, and inset the open ground by 2 m. The target count is the smaller of `clipped area × 0.3024 / 95.97` and `dwellings × clipped area / full area / 1.48`, rounded to the nearest integer, with a minimum of one. For each component, subtract one target footprint per 95.97 m² of mapped roofs, rounded up. Skip components with a remaining target of zero or less. The builder uses the unrounded reference values.
5. Align a grid with the minimum-area rotated rectangle of the inset block. The cell spacing is the square root of inset area divided by target count. An MD5 hash of block ID, component and grid indices determines each centre offset, footprint dimensions and height variation. The builder accepts only rectangles completely inside the inset. A block that cannot fit a grid rectangle gets one smaller rectangle inside its largest inscribed circle if room permits.
6. Round coordinates to 0.00001 degrees. Reject shapes with zero area or coordinates outside the box. The resulting median generated footprint is 88.7 m². The 10th and 90th percentiles are 69.1 and 113.5 m². A spatial check on the final output found zero intersections with rendered OSM and zero shapes outside their source block.

## Three camera centres

Count every packed polygon that intersects a 500 m circle in UTM zone 19S. Clip each roof to the circle and measure the area of their union. Each pair shows the previous and current packs.

| Centre (longitude, latitude) | Mapped | Generated before → after | Roof cover before → after |
| --- | ---: | ---: | ---: |
| Tobalaba (−70.600974, −33.417357) | 460 | 0 → 0 | 29.11% → 29.11% |
| Puente Alto (−70.57536, −33.60958) | 337 | 5 → 82 | 17.51% → 18.39% |
| Quilicura (−70.72939, −33.36569) | 319 | 540 → 778 | 17.86% → 20.45% |

All 375,281 outlines from previously unmapped blocks retain their geometry and height. The 111,413 new outlines have a median area of 92.62 m². The 5% threshold is a modeling choice. Low roof cover can also reflect open ground.

## Heights

For an INE polygon of area `A` m² with `D` dwellings and `J` modeled jobs at the same block ID, estimate height as `3 × max(1, (65D + 20J) / (0.30A))` metres. The display assumes 65 m² per dwelling, 20 m² per job, 30% building coverage, and 3 m per storey. These are display assumptions. Census dwelling counts and modeled jobs remain distinct inputs. The airport has no block ID match.

Tagged OSM height takes priority. The packer uses tagged levels times 3.5 m when height is absent. Cap density estimates at 77 m, the 99th percentile among 23,358 shapes with a height or levels tag. For generated footprints, multiply the block estimate by a deterministic factor from 0.78 to 1.23, then constrain the result to 3–77 m. The median generated height is 3.5 m. The cap applies only to modeled heights.

## Rebuild

Install DuckDB CLI 2.0.0 and its `spatial` extension (`duckdb -c 'INSTALL spatial'`). Run the OSM preparation step from the mesh builder first. Then run from the repository root:

```sh
python3 -I tools/pack/building_fill.py data/raw/verify-people-work/cartography-rm.gpkg data/raw/building-fill/real_rendered.geojsonseq data/raw/building-fill
```

The builder writes the real shape assignments, generated shape stream and coverage table in the raw directory. Only the packed mesh and the comuna coverage table belong in the release. Geometry processing uses DuckDB Spatial and GEOS. [DuckDB Spatial functions](https://duckdb.org/docs/stable/core_extensions/spatial/functions.html) · [INE RM cartography](https://storage.googleapis.com/bktdescargascenso2024/Cartografia/GPKG/Cartografia_censo2024_R13.zip) · [INE Census block data](https://storage.googleapis.com/bktdescargascenso2024/Datos_agregados/Base_manzana_entidad_CPV24.zip) · [© OpenStreetMap contributors](https://www.openstreetmap.org/copyright)

## Building-to-block lookup for the Rust viewer

`web/data/city_building_block_ids.bin.gz` maps a mesh tile and its local building index to the INE block ID and comuna CUT. The near and far meshes use the same index. The sidecar has 766,079 entries and 45,291 distinct assigned blocks. It covers all 486,694 generated shapes and 279,342 mapped shapes. The other 43 mapped shapes have no block assignment. A separate dictionary preserves 1,039 assigned polygon IDs absent from the home-block list in `city_blocks.json`.

Gunzip the file once. It is 303,653 bytes compressed and 1,986,528 bytes inflated. All integers are little-endian. Arrays are packed without padding.

| Byte offset | Contents |
| ---: | --- |
| 0 | ASCII magic `SGBI` |
| 4 | Column count `u16`, then row count `u16` |
| 8 | Dictionary length `D: u32`, then building count `N: u32` |
| 16 | `cols × rows + 1` consecutive `u32` offsets, in **building entries** |
| After offsets | `D` dictionary records of 10 bytes each, containing `block_id: u64` then `cut: u16` |
| After dictionary | `N` dictionary indexes (`u16`). Value `65535` means unassigned. |

Tiles run west to east within each row, starting at the south. Use the same tile and zero-based building index passed to the mesh decoder. Render a shape with an unassigned entry at its baseline. The function below accepts the inflated bytes and a valid tile-local index. Use byte reads because the dictionary's 10-byte records can be unaligned.

```rust
fn building_block(bytes: &[u8], col: usize, row: usize, i: usize) -> Option<(u64, u16)> {
    let u16_at = |p: usize| u16::from_le_bytes(bytes[p..p+2].try_into().unwrap());
    let u32_at = |p: usize| u32::from_le_bytes(bytes[p..p+4].try_into().unwrap()) as usize;
    let cols = usize::from(u16_at(4));
    let rows = usize::from(u16_at(6));
    let dict = 16 + 4 * (cols * rows + 1);
    let entries = dict + 10 * u32_at(8);
    let start = u32_at(16 + 4 * (row * cols + col));
    let key = u16_at(entries + 2 * (start + i));
    if key == u16::MAX { return None; }
    let p = dict + 10 * usize::from(key);
    Some((u64::from_le_bytes(bytes[p..p+8].try_into().unwrap()), u16_at(p+8)))
}
```

Rebuild after the mesh preparation and fill steps with `python3 -B tools/pack/building_block_ids.py`. Its optional argument specifies another raw fill directory. It matches source assignments to the exact quantized exterior and hole coordinates of each published polygon. Mapped and generated shapes have separate lookup keys. All 766,079 source and mesh records match. Six identical source geometries have duplicate records, each with the same block assignment. Within-tile raw feature order can differ from the published order. Use the sidecar and both meshes from the same build.
