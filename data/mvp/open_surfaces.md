# Santiago open surfaces

The city view can read `web/data/open_surfaces.geojson.gz` as a GeoJSON FeatureCollection. It covers the same rectangle as the city buildings (−70.85 to −70.45 longitude, −33.65 to −33.30 latitude). Each polygon carries `kind`, `osm_type`, `osm_id` and an optional OSM name. Coordinates use WGS84 longitude and latitude. Draw every inner ring as a hole.

| Kind | OSM objects | Packed polygons | Selection |
| --- | ---: | ---: | --- |
| `surface_parking` | 864 | 866 | Explicit `amenity=parking` and `parking=surface`. Exclude tagged covers, interiors, buildings, building parts and negative layers. [OSM surface parking](https://wiki.openstreetmap.org/wiki/Tag:parking%3Dsurface) |
| `parking_unspecified` | 1,084 | 1,084 | Mapped `amenity=parking` with no parking subtype. Apply the same structure and cover exclusions. The physical parking type is unknown. [OSM parking](https://wiki.openstreetmap.org/wiki/Tag:amenity%3Dparking) |
| `brownfield` | 138 | 138 | Land mapped for redevelopment. Existing structures can remain. [OSM brownfield](https://wiki.openstreetmap.org/wiki/Tag:landuse%3Dbrownfield) |
| `greenfield` | 206 | 206 | Undeveloped land mapped for future development. [OSM greenfield](https://wiki.openstreetmap.org/wiki/Tag:landuse%3Dgreenfield) |
| `vacant` | 0 | 0 | The source has no polygon tagged `landuse=vacant` in this view. |
| `pitch` | 6,461 | 6,461 | Mapped sports pitch polygons. Exclude tagged interiors. [OSM pitch](https://wiki.openstreetmap.org/wiki/Tag:leisure%3Dpitch) |
| `track` | 142 | 142 | Sports tracks mapped as multipolygons or ways with `area=yes`. Exclude tagged interiors. Tracks drawn only as paths have no measured width. [OSM track](https://wiki.openstreetmap.org/wiki/Tag:leisure%3Dtrack) |

The pack has 8,895 source objects, 8,897 valid polygons, 84 interior rings, and 49,577 vertices including ring closures. It uses 330,686 bytes compressed and 2,324,779 bytes inflated. The builder clips to the city view, simplifies in UTM zone 19S with a 4 m tolerance, and rounds coordinates to five decimal places.

## Places checked in the pack

| Place | OSM object | Kind | Packed area |
| --- | --- | --- | ---: |
| Quilicura, Avenida Manuel Antonio Matta 437 | [Way 758744506](https://www.openstreetmap.org/way/758744506) | Brownfield | 24,203 m² |
| Quilicura, customer parking | [Way 1181902474](https://www.openstreetmap.org/way/1181902474) | Surface parking | 12,405 m² |
| Puente Alto, athletics track | [Relation 14644878](https://www.openstreetmap.org/relation/14644878) | Track with one interior ring | 3,649 m² |

## Building overlap in the first infill pack

Measured against building commit `d6b81a07c2f7558f7e8db0a4d2418af5bf461bd7`. Of its 111,413 new generated roofs, 2,643 overlap at least one packed surface by positive area. Pitches intersect 1,746 roofs, greenfields 495, tracks 227, brownfields 83, unspecified parking 74, and surface parking 53. A roof can intersect more than one class.

None of these overlaps falls inside the 500 m circles at Tobalaba (−70.600974, −33.417357), Puente Alto (−70.57536, −33.60958), or Quilicura (−70.72939, −33.36569). This comparison covers only those newly generated roofs and the surface classes above.

## Source and rebuild

The cached urban extract comes from the Geofabrik Chile snapshot with OSM data through 2026-11-20 21:21:21 UTC. The pack uses ODbL 1.0. Display © OpenStreetMap contributors. [Geofabrik snapshot](https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf) · [OpenStreetMap copyright](https://www.openstreetmap.org/copyright)

Run from the repository root with the same cached urban extract as the building pack, the osmium CLI, and DuckDB Spatial.

```sh
python3 tools/pack/city_open_surfaces.py data/raw/osm-buildings-city/urban.osm.pbf
```
