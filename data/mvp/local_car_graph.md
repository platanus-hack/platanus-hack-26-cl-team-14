# Local car graph for building detours

The viewer uses `web/data/local_car_graph.json.gz` only when a placed building blocks a car route.
This pack adds residential and other local roads that the editable major-road graph does not contain.

Build from an OSM PBF that covers the city rectangle:

```sh
python3 -B tools/pack/local_car_graph.py /path/to/region.osm.pbf
```

The builder uses `osmium extract --strategy complete_ways` and `osmium tags-filter`.
Both retain original member nodes. It clips each source segment to the existing city rectangle,
from longitude -70.85 to -70.45 and latitude -33.65 to -33.30.

- `nodes` maps original OSM node IDs to longitude and latitude. Boundary cuts have distinct synthetic IDs.
- `ways` maps OSM way IDs to `structure` and directed `edges`. Each edge has `from`, `to`, and `length_m`.
- Only shared source nodes make junctions. Line crossings do not create junctions.
- Car access and one-way rules use the same helpers as `editable_roads.py`. They include directional access, reverse-only ways, and implied one-way roundabouts and motorways. Private and destination-only roads are excluded.
- Motorway, trunk, primary, secondary, tertiary, their links, residential, unclassified, living streets, and service roads are eligible.
- `structure` is ground, bridge, or tunnel, from OSM tags. A tunnel tag takes precedence over a bridge tag.
- The graph omits turn restrictions, conditional access, node barriers, and traffic delays. Detours correct display geometry; they do not establish a new travel-time estimate.

## Published pack

The source is a retained regional OSM extract. Its exact snapshot date is unverified.
`osmium fileinfo -e` reports a last object timestamp of `2026-11-20T21:14:28Z`.
That timestamp does not establish the extraction date.

- Source: 62,420,035 bytes; SHA-256 `43596705fc3af495df63c47ed66b10ac19e699a02d54867a7b1786a6049de529`.
- Pack: 105,371 ways, 458,902 nodes, 833,550 directed edges; 12,574,846 compressed bytes.
- The known stadium crossing, OSM way 23350356, is present as a ground road with nine directed edges.
- Sorted JSON and a zero gzip timestamp make the output deterministic for the same source.

Sources: [Osmium manual](https://osmcode.org/osmium-tool/manual.html),
[local direction and access rules](../../tools/pack/editable_roads.py).
Credit © OpenStreetMap contributors. Data and derived geometry use [ODbL 1.0](https://www.openstreetmap.org/copyright).
