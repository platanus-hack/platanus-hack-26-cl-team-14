# Surface-car water crossings

Rebuild the sidecar from the OSM urban extract used by `road_centerlines.geojson.gz`. Its SHA-256 is `4a1cfdb3ffdfdb58cbc6247ab36cac38ad2dc35509b5c1e102a7e53461234cb3`. The source snapshot is dated 20 November 2026. The clip follows the road pack bounds, from (−70.85, −33.65) to (−70.45, −33.30). [Road source and bounds](road_surface.md) · [OSM attribution](https://www.openstreetmap.org/copyright)

```sh
uv run --with shapely==2.2.0 python -B tools/pack/car_water_spans.py /path/to/urban.osm.pbf
```

`water` contains 627 WGS84 polygons with 122 interior rings. `water_ids` identifies the corresponding OSM ways and relations. Sources include `natural=water`, the older `waterway=riverbank` and `landuse=reservoir`. For the two Mapocho relations the pack uses the exact river polygons drawn by the viewer. Waterways mapped only as lines have no measured banks in this pack. A dry detention basin also needs a water tag to qualify. The decorative outer river ribbon supplies no additional water area. [OSM water mapping](https://wiki.openstreetmap.org/wiki/Tag:natural%3Dwater) · [Mapocho geometry](../../tools/pack/mapocho_detail.py)

`bridges` stores the OSM way ID, full mapped or estimated width in metres, and WGS84 motor-road centerline for each of 208 bridge parts near water. `local` uses the local car path index. `od` uses the origin and destination comuna codes joined by a hyphen. Each interval stores the starting and ending fractional vertex index on that path, and one of `bridge`, `tunnel` or `water`. The native viewer converts those indices to distances along its terrain-following path.

A structural match requires the path and road axes to agree within about 35° and lie within half of the mapped width plus 3.8 m. The allowance covers the 3 m local route simplification and coordinate rounding. The 20 m simplification of some comuna-pair routes can leave a larger gap. An aligned bridge takes precedence. A tunnel or negative-layer road qualifies for the underground class only in water and where there is no aligned surface candidate. `water` records a geometric overlap without a supported structure. The source does not identify the physical cause of every overlap. Neither layer nor deck width supplies a measured elevation. [Local route pack](../../tools/pack/local_mover_routes.py) · [Comuna routes](../../tools/pack/car_paths.py) · [OSM layer semantics](https://wiki.openstreetmap.org/wiki/Key:layer)

Local path 322 illustrates both outcomes. At (−70.6304044, −33.4345256) the route is 0.52 m from the Costanera Norte tunnel, OSM way 26296717. At (−70.6182834, −33.4225089) it is 0.44 m from the mapped bridge, way 1343886627. Both coordinates fall inside Mapocho relation 15533903. [Tunnel way](https://www.openstreetmap.org/way/26296717) · [Bridge way](https://www.openstreetmap.org/way/1343886627)
