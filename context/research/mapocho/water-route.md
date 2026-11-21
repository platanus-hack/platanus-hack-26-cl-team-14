# Mapocho route along the mapped river

[Water route](water-route.geojson) gives nine contiguous eastbound legs from MAP01 to MAP10. Each line lists its WGS84 coordinates in longitude, latitude order. `from` and `to` match [the planning piers](piers.json). For westbound service, reverse both the leg order and the line coordinates.

The route follows the OpenStreetMap centerline from WaterwayMap. Projecting the ten planning points onto the centerline moved them by at most 1.3 m. Every saved leg lies within the two packed water polygons. For MAP07 → MAP08, the line path is 2,736 m. [Source centerline](https://waterwaymap.org/river/R%C3%ADo%20Mapocho%20000243500841/geometry.geojson) · [Packed water polygons](../../../web/data/mapocho_detail.geojson)

## Timing

The 22.778 km route takes 80.4 minutes at the model's 17 km/h speed. Eight one-minute stops bring one-way time to 88.4 minutes, before any terminal layover. The [boat line input](boat_line.json) used straight distances between piers. To use this route for both motion and modeled travel time, update `segment_minutes` from `ride_minutes_at_17_kmh` and regenerate the replay. Preserve the separate dwell and headway assumptions.

## Reproduce

Use `piers.json` and `web/data/mapocho_detail.geojson` from commit `0d44d76636d9af6e64b473530ec55d8a8e480a95`. The downloaded centerline SHA-256 is `ae07c19f7b3264b999a3bb5f51216fb8c6da4c2868f5342bb7ad57bb85d3948d`.

Scale longitude and latitude by 92,900 and 110,600 meters per degree, as in the local distance model. Merge the connected centerline ways and take the longest line. Project the pier points onto that line, then take the substring between each pair of projections. Convert the coordinates back to WGS84 and round to eight decimals. I checked that the union of the two water polygons covers each saved line.

The route gives a path for a simulated boat. The maps do not establish depth, clearance or navigability. For the pier close-up, keep land access separate from the boat path.
