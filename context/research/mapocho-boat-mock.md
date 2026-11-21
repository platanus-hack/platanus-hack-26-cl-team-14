# Mapocho boat mock

The navigable Mapocho scene needs a ferry silhouette that reads on a projector. This mock uses the central reach near Parque Forestal. It shows one hypothetical ferry.

- Use a broad cabin and distinct bow. Brisbane's CityCat fleet gives a real example of a river passenger ferry. [RiverCity Ferries fleet](https://www.rivercityferries.com.au/our-fleet)
- Align the bow with the water. MapLibre uses the same line direction principle for map symbols. [MapLibre layer specification](https://maplibre.org/maplibre-style-spec/layers/#icon-rotation-alignment) · [MapLibre placement source](https://github.com/maplibre/maplibre-gl-js/blob/main/src/symbol/placement.ts)
- Use Parque Forestal and Puente Pío Nono to orient the viewer. The mock draws the river and bridges from the repository's OSM data. [Santiago Turismo](https://www.santiagoturismo.cl/es/parque-forestal/) · [Mapocho data](../../web/data/mapocho_detail.geojson)

The ferry is larger than geographic scale. Its hull, cabin, and wake use separate shapes. This still shows the proposed look of the replay. The location and heading illustrate one frame.
