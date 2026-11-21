# Mapocho pier geometry for the replay

All ten planning points lie inside the mapped Mapocho water polygons. A point shows the assumed location of a stop on the river axis. The close-up needs a separate point on land for the walking approach and a point in water for the ferry. OSM describes a pier as a walkway over water. [Planning points](mapocho/piers.json) · [OSM pier tag](https://wiki.openstreetmap.org/wiki/Tag:man_made%3Dpier)

| Planning point | Distance to nearest mapped water edge, m |
| --- | ---: |
| MAP01 | 15.2 |
| MAP02 | 21.4 |
| MAP03 | 10.1 |
| MAP04 | 11.4 |
| MAP05 | 10.6 |
| MAP06 | 13.5 |
| MAP07, near Pío Nono | 14.5 |
| MAP08 | 17.0 |
| MAP09 | 3.6 |
| MAP10 | 14.1 |

At approximately 5 m sample spacing, eight of the nine straight segments between consecutive points leave the mapped water. About 6% of the straight MAP07 to MAP08 segment lies inside water. The replay needs a route that follows the river before it can place a moving boat.

I used the two water multipolygons at commit `0d44d76636d9af6e64b473530ec55d8a8e480a95`. The OSM extract timestamp is 21 November 2026, 13:49 Chile time. I projected longitude and latitude to local planar metres with 111,195 m per degree of latitude and cosine scaling at 33.42° south. I tested point containment, including holes, and measured distance to the nearest polygon edge. I tested each straight segment at the midpoint of subdivisions at most 5 m long. [Water geometry](../../web/data/mapocho_detail.geojson) · [Geometry source](../../tools/pack/mapocho_detail.py)

The mapped edge gives a distance reference. Actual walking access, the final boat route and water depth still need evidence.
