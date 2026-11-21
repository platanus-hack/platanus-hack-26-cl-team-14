# Santiago road surfaces

The candidate turns mapped OSM centerlines into estimated road and path surfaces. It reads the same urban PBF as the building pack. Widths determine the geometry. These surfaces do not establish surveyed curb or legal parcel boundaries.

## Build on the test laptop

Use the exact builder commit and the cached November 20, 2026 Santiago urban PBF. Use the repository's Python 3, osmium and DuckDB CLI with Spatial installed. Run from a full checkout so the report can read the published movement and street layers.

```sh
python3 -B tools/pack/road_surface.py /absolute/path/to/urban.osm.pbf
python3 -B tools/pack/road_surface_report.py /absolute/path/to/cameras.csv
shasum -a 256 web/data/road_surface.geojson.gz
```

The camera CSV header is `camera,lon,lat,radius_m`. Supply the capture site by the tower, Plaza Baquedano, the motorway junction, the Maipú residential grid and the Quilicura industrial street. Use the agreed camera centres and report the radius measured on the ground for each. The report accepts no CSV when only the whole-pack checks are needed. Its scratch database and files live under `data/raw/road-surface/`.

## Geometry contract

`web/data/road_surface.geojson.gz` contains one WGS84 GeoJSON FeatureCollection. Each feature is a Polygon with a closed exterior and zero or more closed hole rings in longitude, latitude order. Draw each piece in its own tile. Display © OpenStreetMap contributors. The source uses ODbL 1.0. [Geofabrik snapshot](https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf) · [OpenStreetMap credit](https://www.openstreetmap.org/copyright)

| Property | Values |
| --- | --- |
| `column`, `row`, `tile` | Building grid: 20 columns from west to east and 18 rows from south to north. `tile = 20 × row + column`. Cell size 0.02°, except the last row at 0.01°. Bounds −70.85, −33.65 to −70.45, −33.30. |
| `structure` | `ground`, `bridge`, `tunnel` from OSM. |
| `layer` | Signed OSM layer when tagged. Otherwise 1 for bridges, −1 for tunnels or 0. Relative order only. |
| `use` | `road`, `cycle`, `foot`, `sidewalk`. The last class requires an independently mapped `highway=footway`, `footway=sidewalk`. |
| `kind` | `sidewalk` for a separate sidewalk; otherwise its `structure`. Use `structure` to place a bridge sidewalk or hide a tunnel sidewalk. |

The builder uses each raw way once. Separate OSM carriageways keep their own centerlines. It buffers unsimplified lines in UTM zone 19S with round joins and caps. It rounds the buffered outlines to 0.000001°, clips them to the integer-aligned building grid, then unions polygons of the same tile, structure, layer and use. Road intersections merge regardless of highway class. Separately drawn sidewalks and standalone cycle paths remain distinct features. A sidewalk or cycle lane tagged only on a host carriageway has no separate measured line. This pack does not invent an offset line for it.

## Width sources

The first usable source wins. Read the full OSM `width` in metres (or convert `ft`). Otherwise multiply an explicit positive integer `lanes` count by the display assumption of 3.2 m. Otherwise use a class default below. A missing width on a one-way carriageway uses the same class rule. The pack reports each source separately. [OSM width](https://wiki.openstreetmap.org/wiki/Key:width) · [OSM lanes](https://wiki.openstreetmap.org/wiki/Key:lanes) · [OSM sidewalk](https://wiki.openstreetmap.org/wiki/Tag:footway%3Dsidewalk)

| Highway class | Assumed width (m) |
| --- | ---: |
| motorway, trunk, primary | 7 |
| secondary, tertiary | 6.4 |
| residential, unclassified, road | 5.6 |
| living_street, pedestrian | 4 |
| motorway_link, trunk_link | 3.6 |
| primary_link, service, busway | 3.5 |
| secondary_link, tertiary_link | 3.2 |
| track | 3 |
| cycleway, bridleway | 2.5 |
| footway, path | 1.8 |
| steps | 1.5 |

Area-tagged ways and explicitly indoor ways are excluded. Access restrictions affect who can use a road, not its physical presence. A visible private road is not a new routing permission.

## Compare each candidate to the same baseline

Record the input PBF SHA, commit, inflated and compressed bytes, polygon and hole counts, and decoder output. The report checks valid polygon geometry, tile IDs, tile containment, and matching coverage on neighboring tile edges. It compares shared raw-way coordinates on the same mapped structure and layer with each surface. A shared coordinate is geometry evidence and can occur without a routable connection.

The movement comparison uses each distinct local OSRM route once per mode, plus the published representative bus shapes. It clips each path to each camera circle and samples the midpoint of intervals at most 20 m long. It reports sampled metres, metres outside a visible surface and the percentage outside, by camera, mode and source. The current baseline uses the renderer's widths along the two existing simplified street layers. The candidate includes bridge surfaces and excludes tunnels from the visible comparison. A hit on a nearby road at another height can still count. These plan-view distances do not establish route legality or confirm what the renderer displays.

Keep the same PBF, camera CSV, routes and baseline for the next iteration. Compare all five fixed views after the rendering agent loads the candidate. Record any mode or camera that gets worse as well as the ones that improve. The repository contains the builder before the test laptop produces its first pack.
