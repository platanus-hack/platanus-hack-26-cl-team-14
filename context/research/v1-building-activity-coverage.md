# Activity coverage in the current building pack

At `8d47249f0022abfb1a3dc41dd3d11aef1c1228ec`, **10,254 of 279,385 mapped viewer footprints (3.67%)** cover at least one point in the packed activity catalog. Both packs use the same Santiago rectangle. The catalog contains retail, health and public-office objects. This figure measures coverage of these files and categories. [Viewer scope](../../viz/README.md) · [Activity catalog](../../data/mvp/activity_points.md)

| Measured unit | Count |
| --- | ---: |
| Mapped footprint records | 279,385 |
| Mapped footprints with at least one catalog point | 10,254 |
| Generated shapes, excluded from the join | 459,845 |
| Catalog points | 16,697 |
| Catalog points covered by at least one mapped footprint | 13,767 |
| Catalog points covered by more than one mapped footprint | 161 |

Decode the polygon pack using the `SGB1` offsets and coordinate conversion in `tools/pack/named_places.py`. Keep mapped footprint records, where flag bit 4 is clear. Search the activity points with a spatial index. Count a match when each polygon covers the point in WGS84, with interior rings and boundaries included in the predicate. Count each footprint and each point once in their respective totals. Ten matched polygon records fail Shapely's validity check. [Footprint pack](../../web/data/city_building_polygons.bin.gz) · [Polygon decoder](../../tools/pack/named_places.py) · [Activity points](../../data/mvp/activity_points.csv)

A match gives a candidate place inside a mapped footprint. It does not establish shared identity, opening hours, capacity, or floor-level use. Many mapped buildings can contain homes, workplaces, or services outside the packed activity classes. The generated shapes have no observed building identity. The overlap supports a first lookup of nearby places that offer a service. Throughput at city scale still needs a run with the proposed behavior.
