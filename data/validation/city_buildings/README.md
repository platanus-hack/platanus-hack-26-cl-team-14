# Building geometry coverage in Santiago

The mesh combines 279,385 mapped OSM outlines with 459,845 generated outlines. The builder adds outlines in populated blocks with less than 5% mapped roof cover. Generated positions, shapes and heights describe the model. A flag identifies each generated outline.

## Coverage

The view extends from −70.85 to −70.45 longitude and −33.65 to −33.30 latitude. The two INE layers intersect the box in 58,467 polygons. Of these, 44,228 have residents in `blocks.csv`. This excludes the 53 Census containers without a polygon.

| Populated polygons | Count |
| --- | ---: |
| With at least one rendered OSM building intersection | 18,756 |
| With no rendered OSM intersection | 25,472 |
| With mapped buildings and less than 5% mapped roof cover | 4,196 |
| With new infill in a partly mapped block | 3,845 |
| With generated buildings, including blocks without a mapped roof | 29,271 |

One zero-OSM polygon in Padre Hurtado (`13605022006052`) enters the box as a narrow strip of 182 m². No footprint fits inside its 2 m setback. The [comuna table](coverage_by_comuna.csv) lists all 52 RM comunas. The `real_share` column divides populated polygons with any rendered OSM intersection by the number of populated polygons in the view, rounded to six decimals. Nine comunas have no such populated polygon in the view. Their share is blank. A share of zero means the view contains populated polygons but none touch a mapped building. Its real-shape count includes both populated and unpopulated blocks. It excludes 43 OSM shapes without a positive-area assignment to INE. Of those, one has a tagged height or floor count. Of the other 42, five now have a satellite estimate and 37 receive the neutral 3 m height.

## How the geometry is built

1. Start with the OSM building layer after the packer excludes polygons that cross the view boundary and quantizes coordinates to five decimal places. Use the same exterior rings and holes that the view decodes. Use valid geometry for the spatial comparison (125 source components need a repair).
2. Join all rendered OSM shapes against INE `Manzanas_CPV24` and `Entidades_CPV24` by any geometric intersection in UTM zone 19S. A roof can cover parts of multiple blocks. Assign it to exactly one block for height and counts. Prefer a block containing the shape's interior point. Break ties by greatest overlap, then block ID. Require positive overlap for that assignment.
3. Learn a typical footprint from 7,957 populated reference blocks with at least five assigned buildings and summed OSM footprint area between 20% and 90% of block area. The median of block-level median footprint areas is 95.97 m². Median mapped coverage is 30.24%, and the median ratio is 1.48 dwellings per mapped building. These describe the selected reference blocks.
4. Clip each eligible polygon to the view in longitude/latitude before projection. For a partly mapped block, divide the union area of all mapped roofs inside the visible block by its visible area. Use a strict threshold of 5%. Subtract the mapped roofs from each component, preserve holes, and inset the open ground by 2 m. The target count is the smaller of `clipped area × 0.3024 / 95.97` and `dwellings × clipped area / full area / 1.48`, rounded to the nearest integer, with a minimum of one. For each component, subtract one target footprint per 95.97 m² of mapped roofs, rounded up. Skip components with a remaining target of zero or less. The builder uses the unrounded reference values.
5. Align a grid with the minimum-area rotated rectangle of the inset block. The cell spacing is the square root of inset area divided by target count. An MD5 hash of block ID, component and grid indices determines each centre offset, footprint dimensions and height variation. The builder accepts only rectangles completely inside the inset. A block that cannot fit a grid rectangle gets one smaller rectangle inside its largest inscribed circle if room permits.
6. Round coordinates to 0.00001 degrees. Reject shapes with zero area or coordinates outside the box. Reject each whole rounded polygon that touches an exclusion polygon. Keep its boundary and holes in this comparison. The resulting median generated footprint is 88.6 m². The 10th and 90th percentiles are 69.1 and 113.4 m². The generator found zero intersections with mapped roofs or its working exclusion geometry. Every generated roof lies inside its source block.

The exclusions come from the same November 20, 2026 OSM extract as the buildings. The builder reads 31,972 polygon features. They cover mapped school and higher-education grounds, squares, green areas, sports grounds and buildings, pitches, area tracks, outdoor parking, and vacant, brownfield and greenfield land. For parking without a subtype, the builder includes polygons without a building, roof, indoor or below-ground tag. The builder checks both original polygon boundaries and the 4 m simplified, five-decimal boundaries used by the ground layers. Source polygons can overlap. A missing site boundary can still leave open ground eligible for infill. [OSM school tag](https://wiki.openstreetmap.org/wiki/Tag:amenity%3Dschool) · [OSM parking tag](https://wiki.openstreetmap.org/wiki/Tag:amenity%3Dparking)

The city-view review reported 13 generated roofs at Liceo Industrial A-116, 10 at Colegio Santiago Quilicura, three at Plaza Villa Las Golondrinas, and two on parking grounds. On the complete polygons, the unmasked infill pack had 13, 11, three and five intersecting **new** roofs, respectively. The five parking hits span OSM ways 1365992954 and 533593349. The school ways are 28410927 and 966324740. The plaza way is 750755596. No generated roof now intersects these sites. A separate check decoded all 459,845 generated roofs from the published polygon binary. It compared the roofs with the green and open-surface polygons on their shared five-decimal integer grid. Two roofs each share one vertex with a green polygon. Both intersections have zero area. The open-surface layer had zero contacts.

| Green polygon | Roof tile / index | Shared vertex (longitude, latitude) |
| --- | --- | --- |
| Park, OSM way 1126358604 | 82 / 546 | −70.79840, −33.55028 |
| Wood, OSM relation 13222496 | 296 / 351 | −70.52783, −33.35671 |

## Three camera centres

Count every packed polygon that intersects a 500 m circle in UTM zone 19S. Clip each roof to the circle and measure the area of their union. Each pair compares the original 654,666-shape pack with the current pack.

| Centre (longitude, latitude) | Mapped | Generated before → after | Roof cover before → after |
| --- | ---: | ---: | ---: |
| Tobalaba (−70.600974, −33.417357) | 460 | 0 → 0 | 29.11% → 29.11% |
| Puente Alto (−70.57536, −33.60958) | 337 | 5 → 69 | 17.51% → 18.25% |
| Quilicura (−70.72939, −33.36569) | 319 | 540 → 757 | 17.86% → 20.24% |

Compared with the original packed files, 13,971 of the 375,281 previously generated roofs intersect exclusions and were removed. Of the 111,413 candidate additions in partly mapped blocks, 98,535 survive. The net change is 84,564 generated roofs. The retained additions have a median area of 92.25 m². The infill change preserved the coordinates, flags, heights and block assignments of all 279,385 mapped footprints. The subsequent height update is described below.

Of the 361,310 surviving older generated footprints, 14 have a different modeled height in the rebuilt data. Two examples are block 13201141006003 (10.9 → 13.0 m) and block 13119041001011 (23.7 → 29.1 m). Comparison used the delivered polygon binaries and block dictionaries. Every surviving older generated outline retains its geometry and assigned block. The 5% cutoff remains a modeling choice.

## Heights

For an INE polygon of area `A` m² with `D` dwellings and `J` modeled jobs at the same block ID, estimate height as `3 × max(1, (65D + 20J) / (0.30A))` metres. The display assumes 65 m² per dwelling, 20 m² per job, 30% building coverage, and 3 m per storey. These are display assumptions. Census dwelling counts and modeled jobs remain distinct inputs. The airport has no block ID match.

The packer prefers a reported height or floor count on the OSM outline, then a matching OSM part, then the operator’s value. It estimates previously untagged mapped roofs from Google Open Buildings 2.5D Temporal (2023) where one 4 m sample fits inside the roof and meets the presence threshold. For the 665 matched outlines with height tags and a census assignment, the median absolute difference was 3.00 m with satellite data and 4.00 m with the density rule. The comparison leaves the tagged height in the pack. [Sources, method and accuracy by height](../../../context/research/santiago-roof-heights.md).

The packer converts reported floor counts to metres at 3.5 m per floor. Cap density estimates at 77 m, the 99th percentile among 23,358 shapes with a height or levels tag. For generated footprints, multiply the block estimate by a deterministic factor from 0.78 to 1.23, then constrain the result to 3–77 m. The median generated height is 3.6 m. The 77 m cap applies to the census model. Google caps its estimates at 100 m.

## Rebuild

Install DuckDB CLI 2.0.0 and its `spatial` extension (`duckdb -c 'INSTALL spatial'`). Install the osmium CLI. Keep the cached urban OSM PBF used for mesh preparation at `data/raw/osm-buildings-city/urban.osm.pbf`, or pass its path as the last argument below. Run the OSM preparation step from the mesh builder first. Then run from the repository root:

```sh
python3 -I tools/pack/building_fill.py data/raw/verify-people-work/cartography-rm.gpkg data/raw/building-fill/real_rendered.geojsonseq data/raw/building-fill
```

The builder writes the real shape assignments, generated shape stream and coverage table in the raw directory. Only the packed mesh and the comuna coverage table belong in the release. Geometry processing uses DuckDB Spatial and GEOS. [DuckDB Spatial functions](https://duckdb.org/docs/stable/core_extensions/spatial/functions.html) · [INE RM cartography](https://storage.googleapis.com/bktdescargascenso2024/Cartografia/GPKG/Cartografia_censo2024_R13.zip) · [INE Census block data](https://storage.googleapis.com/bktdescargascenso2024/Datos_agregados/Base_manzana_entidad_CPV24.zip) · [© OpenStreetMap contributors](https://www.openstreetmap.org/copyright)

## Building-to-block lookup for the Rust viewer

`web/data/city_building_block_ids.bin.gz` maps a mesh tile and its local building index to the INE block ID and comuna CUT. The near and far meshes use the same index. The sidecar has 739,230 entries and 45,246 distinct assigned blocks. It covers all 459,845 generated shapes and 279,342 mapped shapes. The other 43 mapped shapes have no block assignment. A separate dictionary preserves 1,039 assigned polygon IDs absent from the home-block list in `city_blocks.json`.

Gunzip the file once. It is 302,473 bytes compressed and 1,932,380 bytes inflated. All integers are little-endian. Arrays are packed without padding.

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

Rebuild after the mesh preparation and fill steps with `python3 -B tools/pack/building_block_ids.py`. Its optional argument specifies another raw fill directory. It matches source assignments to the exact quantized exterior and hole coordinates of each published polygon. Mapped and generated shapes have separate lookup keys. All 739,230 source and mesh records match. Six identical source geometries have duplicate records, each with the same block assignment. Within-tile raw feature order can differ from the published order. Use the sidecar and both meshes from the same build.
