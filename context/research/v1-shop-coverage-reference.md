# What the Santiago map can tell us about shop coverage

In the MUT relocation demo, the engine needs places where people can shop. The November 2026 map contains 508 supermarket objects. The share of operating supermarkets that these objects represent is unknown. [Packed activity catalog](../../data/mvp/activity_points.md)

| Count | Unit | Area and date |
| ---: | --- | --- |
| 508 | OpenStreetMap objects classified as supermarkets | Santiago map rectangle, 20 November 2026. The full activity catalog covers parts of 41 comunas. |
| 453 | Supermarket establishments in the INE series | Entire Metropolitan Region, December 2025 |
| 1,357 | Supermarket establishments in the same series | Chile, December 2025 |

The first row counts entries with `kind=supermarket` in the activity CSV. The last two rows come from table 5 of the CNC report, which credits INE. In the December 2025 row, the first numeric column gives the national total. The last column gives the regional count. [Activity CSV](../../data/mvp/activity_points.csv) · [CNC table 5](https://cnc.cl/wp-content/uploads/2023/09/Ventas-de-Supermercados-por-Region-Base-2018-Diciembre-2025-30-01-2026.pdf)

Dividing 508 by 453 gives no coverage estimate. The dates, boundaries and units differ. OSM permits a node or an area to represent a shop. INE applies eligibility rules to establishments. SII company statistics assign firms to their registered tax address. One firm can operate several shops. A coverage check needs outlet records with a common date, boundary and definition. Check duplicate records and record unmatched outlets on both sides. [OSM shop guidance](https://wiki.openstreetmap.org/wiki/Shops) · [INE methodology](https://www.ine.gob.cl/docs/default-source/ventas-de-supermercados/metodologias/base-promedio-a%C3%B1o-2018-100/manual-metodol%C3%B3gico.pdf) · [SII company statistics](https://www.sii.cl/sobre_el_sii/estadisticas_de_empresas.html)

## A campus shows the limit of a point match

The activity pack reduces each mapped area to one point. For the Sótero del Río hospital campus, its point falls outside every packed footprint. The source campus boundary contains 64 mapped footprint records. Two contain a packed activity point strictly inside. These records establish a spatial relation to the campus. The function of each building remains unknown. A missing point match can occur even where both the site and its buildings appear on the map. [Campus, OSM way 41391269](https://www.openstreetmap.org/way/41391269) · [Activity pack method](../../tools/pack/activity_points.py) · [Footprint study, PR 406](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/406)

To repeat the campus check, use the 20 November 2026 Chile extract and the packs at `8d47249f0022abfb1a3dc41dd3d11aef1c1228ec`. Extract way `41391269` with its nodes. Decode the footprint polygons with their holes. Exclude generated records with flag bit 4. Count each footprint that the campus polygon covers. Count footprints that strictly contain a catalog point separately. [OSM source snapshot](https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf) · [Packed polygon layout](../../tools/pack/named_places.py)
