# Two fixed stadium origins

`data/mvp/event_venues.json` records two derived WGS84 venue centers for the conditional extra-event contract. It does not implement that scenario or authorize arbitrary citywide placement.

Primary OpenStreetMap full-way responses retrieved on 2026-11-21 are preserved here (about 26 KB combined). The table records each response SHA-256, way version and way edit timestamp. The timestamp is the way's edit time, not a survey date; referenced nodes can have different versions. The snapshots pin all node coordinates used. Attribution: © OpenStreetMap contributors, Open Database License.

- [Nacional way 28092600](https://www.openstreetmap.org/api/0.6/way/28092600/full), version 27: named stadium bowl at Avenida Grecia 2001. The [government heritage record](https://patrimonio.bienes.cl/patrimonio/estadio-nacional/) confirms the address; its UTM datum is unstated, so those coordinates were not converted. The [official IND access page](https://parqueestadionacional.cl/como-llegar) distinguishes park entrances from Metro access.
- [Monumental way 297163434](https://www.openstreetmap.org/api/0.6/way/297163434/full), version 7: named stadium bowl, not the larger sports complex. [Colo-Colo's museum page](https://colocolo.cl/museo) locates the venue at Marathon 5300, Macul.

The source geometry is observed open mapping. Using its centroid as one event origin is a modeling assumption. It is not an entrance, walking route, crowd-access distribution or capacity estimate. Venue coordinates are distinct from GTFS station coordinates. The fixed 1 km radius and inverse-distance rule give Nacional only ESN; Monumental gives PE and MA. The nearest excluded parents are over 1.2 km away. The 50 m floor binds neither venue.

## Reproduce the centroid

For each snapshot, read `node` longitude/latitude by ID and follow the `way/nd` order, including its repeated closing node. Translate coordinates by the first vertex `(ox, oy)`. For each consecutive pair `(x_i, y_i), (x_j, y_j)`, compute `c_i = x_i*y_j - x_j*y_i` and `S = sum(c_i)`. Then:

```
longitude = ox + sum((x_i + x_j) * c_i) / (3 * S)
latitude  = oy + sum((y_i + y_j) * c_i) / (3 * S)
```

This is the planar polygon area centroid in longitude/latitude. Translation avoids cancellation. It is a reproducible derived point, not geodesic precision or a surveyed location. Applying a constant local flat scale to both axes gives the same centroid.

The table's parent audit uses all 126 rows of the pinned `metro_training.json` and the exact accepted L4b function in `engine/src/stations.rs`: `dx = delta_lon * 111320 * cos(-33.45 degrees)`, `dy = delta_lat * 110540`, `d = hypot(dx, dy)`. Include `d <= 1000`; normalize `1 / max(d, 50)` over included parents. Runtime must derive this allocation, not use the expected audit shares as inputs.

No model, scorer, weekday fit, runtime route or UI changes are included. No tests were added or run.
