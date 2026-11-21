# Mapocho boat mock

The navigable Mapocho scene needs a ferry silhouette that reads on a projector. This mock uses the central reach near Parque Forestal. It shows one hypothetical ferry.

- Use a broad cabin and distinct bow. Brisbane's CityCat fleet gives a real example of a river passenger ferry. [RiverCity Ferries fleet](https://www.rivercityferries.com.au/our-fleet)
- Align the bow with the water. MapLibre uses the same line direction principle for map symbols. [MapLibre layer specification](https://maplibre.org/maplibre-style-spec/layers/#icon-rotation-alignment) · [MapLibre placement source](https://github.com/maplibre/maplibre-gl-js/blob/main/src/symbol/placement.ts)
- Use Parque Forestal and Puente Pío Nono to orient the viewer. The mock draws the river and bridges from the repository's OSM data. [Santiago Turismo](https://www.santiagoturismo.cl/es/parque-forestal/) · [Mapocho data](../../web/data/mapocho_detail.geojson)

The ferry is larger than geographic scale. Its hull, cabin, and wake use separate shapes. This still shows the proposed look of the replay. The location and heading illustrate one frame.

## Cached replay preview

The mock can read the cached baseline and ferry results. It draws only accepted boat-line additions. It walks each line by distance between consecutive stops and uses the saved travel, dwell, and headway values. The two directed lines come from the result. The engine does not run for this preview. [Routed replay bundle](https://discord.com/channels/1573514576400752640/1573809606277337280/1573822794423345193) · [Water route and timing](mapocho/water-route.md)

The cache specifies a headway but gives no first departure. The mock uses departures at :00, :10 and each later ten-minute mark in both directions. It assumes service is already running when the 07:00 scene begins. This is a display assumption for V3’s draft. A frequency timetable needs explicit start and end times. [GTFS frequency reference](https://gtfs.org/documentation/schedule/reference/#frequenciestxt)

1. Put `mapocho-baseline.json` and `mapocho-ferry.json` from the linked bundle in a local `preview/` folder at the repository root.
2. From the repository root, run `python3 -m http.server 43927 --bind 127.0.0.1`.
3. Open `/pitch/mapocho-boat.html?replay=/preview/mapocho-baseline.json&at=42` and `/pitch/mapocho-boat.html?replay=/preview/mapocho-ferry.json&at=42` on that server.

The two URLs show the same camera at 07:00:42. The baseline shows no ferry. The experiment shows the 06:10 eastbound vessel west of Pío Nono. Add `&play=1` to advance from the same time at eight city seconds per wall second.
