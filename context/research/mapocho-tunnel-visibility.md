# Keep tunnel traffic below the Mapocho

The Mapocho replay needs moving cars on the banks and bridge decks. Use the mapped road structure to decide where a car is visible. This change belongs in road route packing and playback. The engine's route and travel time stay the same. The existing story places this view at 2:15–2:40. [Live pitch draft](pitch-live.md)

## What we checked

At `7b34deca3e5a71f90d5f1bb846ee1f91541bb1b0`, the baseline viewer selects local car path 322 for trip group 387. Its longest continuous overlap with mapped water outside a 12 m motor-bridge buffer is 986.86 m. At the overlap midpoint (−70.6304044, −33.4345256), the route is 0.52 m from the Costanera Norte tunnel way `26296717`. The pinned OSM extract and the checked-in road centerline pack agree that this way is a tunnel at layer −1. [Road centerline pack](../../web/data/road_centerlines.geojson.gz) · [Audit and inputs](https://discord.com/channels/1573514576400752640/1573688247572238336/1573802846145613926)

The renderer hides road meshes tagged as tunnels. The regular car animation samples height from the terrain even on those routes. Other car paths may show different issues. The audit flagged 46 groups by their water overlap and excluded 44 groups that use short local fallback paths. [Road mesh selection](../../viz/src/city/streets.rs) · [Vehicle playback](../../viz/src/movers.rs)

## Source rules

- OpenStreetMap splits road ways at tunnel portals and tags the covered part `tunnel=yes`. `layer` expresses vertical order where features overlap. Its value gives no depth measurement. [OSM tunnel tag](https://wiki.openstreetmap.org/wiki/Key:tunnel) · [OSM layer tag](https://wiki.openstreetmap.org/wiki/Key:layer)
- OSRM can return OSM node IDs along full route geometry. Adjacent node IDs can identify road edges in the same OSM extract. The current packed routes keep simplified coordinates. [OSRM route annotations](https://project-osrm.org/docs/v5.24.0/api/#annotation-object)
- OpenStreetMap Carto gives tagged tunnels a distinct style. Using a road's structure is an established rendering approach. For the surface view here, use the same hidden state as the tunnel road mesh. [OpenStreetMap Carto road styles](https://raw.githubusercontent.com/gravitystorm/openstreetmap-carto/master/style/roads.mss)

## Plan for V2

For new route packs, retain tunnel spans from the selected OSM edges before simplifying the geometry. Keep the start and end positions on the packed route. For the existing routes, compare route geometry with the tagged centerlines offline. Treat coincident surface and tunnel candidates as unresolved until route edge IDs identify the selected way. Nearest geometry can misclassify stacked roads. [Newson and Krumm, map matching](https://www.microsoft.com/en-us/research/publication/hidden-markov-map-matching-noise-sparseness/)

In surface playback, omit the car model and its map mark during the confirmed spans. Continue advancing the route clock. Use group 387 as a witness. Copy 0 should reach the audited point around 09:22:26 in simulated time. This estimate uses planar route length at the viewer's 9 m/s display speed. In the same native take, check a bridge crossing or a car on the riverbank. Preserve visibility in both places. The native capture and the remaining flagged routes are still unverified.
