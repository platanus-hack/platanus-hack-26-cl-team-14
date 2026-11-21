# Metro closure projector mock

The V3 replay closes the full Metro. The V2 mock uses Tobalaba as a local view of that change. Both panels use the same camera and street data. No ridership output is available in this frame.

- W3C recommends sufficient contrast at the edges viewers need to see. Use wide, light tracks against a dark map. Use solid tracks for service and dashed tracks with a cross for closure. [Non-text contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html)
- Mapbox places a halo around map symbols to keep them readable over busy backgrounds. The mock uses dark outlines around the tracks and the MUT label. [Style specification](https://docs.mapbox.com/style-spec/reference/layers/#icon-halo-width)
- The mock reads the repository's DTPM track data, ground map, and MUT outline. It uses the Tobalaba endpoint of the L4 track as its center. [Metro tracks](../../web/data/metro_tracks.geojson.gz) · [Ground map](../../web/data/ground.geojson) · [MUT outline](../../web/data/mut_parts.geojson)

Preview from the repository root with `python3 -m http.server 43152`. Open <http://localhost:43152/pitch/metro-closure.html>.
