# MUT shop points and footprint

The shopping replay removes the 26 mapped shop points nearest to the MUT center. Treat this as the scope of the experiment. The packed data give point locations and a building footprint. They do not establish which businesses occupy each floor.

I compared the Las Condes shop, mall and supermarket rows from the activity pack with the packed podium outline at `12075a0`. I used the polygon's coordinates in WGS84 and counted points on the boundary. [Activity points](../../data/mvp/activity_points.csv) · [MUT podium](../../web/data/mut_parts.geojson) · [OpenStreetMap polygon rules](https://wiki.openstreetmap.org/wiki/Relation:multipolygon)

| Geographic selection | Mapped points |
| --- | ---: |
| Podium bounding box | 26 |
| Exterior ring, with holes filled | 22 |
| Full podium polygon, with its three holes | 16 |

All 16 points in the full polygon are among the 26 nearest points. One of the 16 is the mall object itself, `r18760408`. Inside the exterior ring, six shop points fall in holes. Four more nearest points fall outside the exterior ring. The holes describe a footprint in two dimensions. A shop can be mapped at the same horizontal location on another floor.

To reproduce, read the feature with `kind = podium` and filter the activity CSV to CUT `13114` and kinds `shop`, `mall`, and `supermarket`. For each count, apply the Shapely `covers` predicate to the row's longitude and latitude, first with the bounds rectangle, then with the exterior ring, then with the complete polygon. For the nearest selection, sort by distance to the MUT row in the named places table. [Named places](../../data/mvp/named_places.csv)
