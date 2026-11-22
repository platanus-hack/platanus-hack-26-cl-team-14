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

## MAP07 approach from Parque Forestal

The south-bank sidewalk gives a mapped 60 m walk for the pier close-up. Follow OSM way 988341170, version 3, east from longitude −70.63639046, latitude −33.43606371 to longitude −70.63576779, latitude −33.43620778. These two points lie on the same centerline segment. The endpoint is 0.87 m from the nearest point of the repository's layer-0 sidewalk polygon in tile 210. [Versioned sidewalk](https://api.openstreetmap.org/api/0.6/way/988341170/3) · [Surface pack](../../web/data/road_surface.geojson.gz)

The sidewalk endpoint is 30.6 m from the projected berth at longitude −70.63568039, latitude −33.43594133. About 14 m of the straight gap lies in mapped water. Show any connection across that gap as a hypothetical pier or ramp. The closer bridge sidewalk lies on layer 1. Keep the path on the south bank before boarding. The sources supply no ramp grade, site access permission, or boarding time. [Routed berth](mapocho/water-route.geojson) · [Parque Forestal](https://www.santiagoturismo.cl/es/parque-forestal/)

I used local planar scales of 92,900 m per degree of longitude and 110,600 m per degree of latitude to match the river-route calculation. I projected the nearest point of the mapped south sidewalk polygon onto the OSM footway, then measured 60 m back along that footway. Actual walking access between the sidewalk and berth still needs evidence. [Footway tag](https://wiki.openstreetmap.org/wiki/Tag:highway%3Dfootway)
