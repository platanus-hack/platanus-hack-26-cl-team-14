# A building move across census blocks

The MUT replay moves the Monumental to the MUT site. The current engine rejects the move when no census block covers the whole destination footprint. The renderer then receives no building edit for that request. [Pitch moment](../pitch.md) · [Reported move](https://discord.com/channels/1573514576400752640/1573809708047798364/1573840718609383427)

At `db0aa683ddd380c7be685b5fd8e85d016094c724`, the check is in `Catalog::block`. It returns one block ID, CUT and block point. `BuildingEdits::apply` uses these fields to build `effects.blocks`. The scenario loop already accepts multiple effect rows and publishes one original and one new geometry per edit. The existing check on stock withdrawals evaluates all effect rows before it accepts the edit. [Block lookup](../../engine/src/buildings/catalog.rs) · [Building edits](../../engine/src/buildings.rs) · [Scenario application](../../engine/src/scenario.rs)

## Proposed allocation

1. Intersect the full footprint with each census polygon whose bounding box overlaps it. Keep positive area intersections and respect holes. Use the current single-block result when one block covers the entire footprint. A maintained Rust polygon overlay can calculate the intersections for the other case. [Geo BooleanOps](https://docs.rs/geo/latest/geo/algorithm/bool_ops/trait.BooleanOps.html)
2. Store the resulting block shares with the scenario building. Divide each intersection area by the sum of the covered areas. This preserves known job and resident totals even when the footprint includes a street gap. Report the covered share and the assumption that known stocks have uniform density over the covered area. Require at least one block with positive area. [Areal weighting method](https://gistbok-ltb.ucgis.org/current/concept/AM-02-040)
3. For known stock, debit the old block shares and credit the new ones in `effects.blocks`. Keep one complete footprint in `city_edits`. A catalog stadium with unknown employment or residents can move for display while its stock stays unknown. The MUT replay can keep its explicit job swap between the two blocks.

For the demo, run the rejected stadium move again and inspect the complete destination geometry. Check an additional move of a scenario building with declared stocks. The sum of the debits and credits should be zero for each stock that the move preserves. The footprint and allocation should reach the existing scenario endpoint.

## Exact stadium request

Tinta's `B-clear-resolved-request.json` has SHA-256 `d7ee05f24b3e969b4c632de5f8133512dffab88f18926f98994bfc6b1d3990b7`. It contains 32 edits, including the move of `mesh:152:317`. I intersected its destination polygon with the packed census polygons in EPSG:32719. The 17,128 m² footprint overlaps 12 blocks with positive area. The union covers 72.83% of the footprint. The largest block, `13114131009026`, covers 29.37%. The move needs to retain the complete geometry across the gaps. [Request attachment](https://discord.com/channels/1573514576400752640/1573809708047798364/1573841839079956686) · [Census polygons](../../data/mvp/building_blocks.geojson.gz)
