# Santiago Metro passenger paths

Read `web/data/metro_tracks.geojson.gz` as a GeoJSON FeatureCollection. It covers the seven passenger lines in the March 2026 DTPM feed. The pack contains 272 directed station-to-station paths, 286 scheduled boarding points, and two platform points from OSM passenger-route members. It uses 226,288 bytes inflated and 22,662 bytes compressed.

## Animate a public route

1. Select a top-level `routes` entry by `line` and GTFS `direction`. It supplies ordered `parent_station_ids` and `segment_ids`.
2. Resolve each segment ID to a LineString with `properties.feature=track`. Each path runs from `from_parent_station_id` to `to_parent_station_id`. The end of one path equals the start of the next. Coordinates use WGS84 longitude and latitude.
3. Read `kind` for the whole leg. A `mixed` leg has multiple classes. Read `origin_kind` and `destination_kind` for the adjoining track at a station. Four endpoint observations lack vertical evidence. The array `kind_sequence` lists consecutive classes in travel order and merges adjacent equal values. An unmatched sequence is empty. The aggregate lengths in `osm_kind_metres` describe the whole leg; the pack has no transition positions or durations for individual runs.
4. Use Point features with `feature=platform` and `source=gtfs_stop` for boarding. They carry the line, parent station ID and DTPM stop ID. Two additional points represent the mean vertices of OSM platform ways in the L2 passenger relation. Their GTFS parents are `RO` and `PQ`.

The paths use the published GTFS shape, split at the nearest point to each intermediate parent station. Both directions match the original shape length within 1 m per line. All 14 ordered OSM stop sequences match GTFS in station count and proximity (each pair within 350 m). The same 143 line-station pairs occur in the paths, the GTFS boarding points and `data/mvp/metro_station_service.csv`. Transfers reduce this to 126 distinct parent IDs. An unmatched sequence has `osm_match=false` and retains its GTFS geometry with unknown evidence.

## Observed passenger platform settings

Four boarding points carry optional `station_setting` and `station_setting_source` properties. Use the station observation for those boarding points. Track classifications apply to train movement. Other boarding points have no verified station-level category in this pack.

| GTFS parent | Line | Platform setting | Evidence |
| --- | --- | --- | --- |
| NP, Neptuno | L1 | `partly_open_below_street` | Hidalgo describes the partially open station type and identifies Neptuno. Her station photographs document the passenger area. [Hidalgo, pp. 76–78](https://www.tesisenred.net/bitstream/handle/10803/80299/TRHC1de17.pdf?isAllowed=y&sequence=1) |
| BL, Blanqueado | L5 | `underground` | The builder identifies Blanqueado among the stations for which it excavated platform tunnels. [OHLA, Santiago Subway, lines 4 and 5](https://www.ohla-group.com/wp-content/uploads/2021/07/Railway-Infrastructures-monografico.pdf) |

Six directed mixed legs have tunnel classifications at both ends. Two contain an inferred surface class between them; the middle class of the other four is unknown.

## Vertical evidence

The builder reads two all-stops OSM passenger route relations per line. It follows their ordered subway ways between passenger stops, trims any ends beyond the terminals, and excludes service tracks, depots and proposed lines. Each leg contains its `osm_relation_id`, `tags`, set of `tagged_layers`, and `osm_kind_metres`. Layer values describe relative vertical order. They do not measure depth.

| Kind | Source evidence | Unique OSM geometry km |
| --- | --- | ---: |
| tunnel | Explicit `tunnel=yes` | 211.5764 |
| viaduct | Explicit `bridge=viaduct` | 32.6242 |
| viaduct | Explicit `bridge=yes`; bridge subtype unspecified | 1.0443 |
| surface | Inferred from `tunnel=no` and no bridge tag; grade unverified | 35.5611 |
| unknown | No tunnel or bridge tag | 1.8425 |

Whole-leg counts across both directions are 194 tunnel, 26 elevated, 11 inferred surface, 40 mixed and one entirely unknown. The unknown leg is L2 from `RO` to `PQ` in direction 1.

**Unique mapped track geometry** totals 282.6485 km. This counts each undirected OSM node-pair geometry once, across both directions. Parallel mapped tracks contribute separately. A single geometry tagged for multiple tracks contributes once. The figure measures mapped passenger geometry and does not establish physical rail inventory.

**Passenger corridor geometry** totals 141.7875 km. This measures one GTFS direction per line with great-circle distances. Direction 0 contributes 97.943 km of wholly tunnel legs, 12.849 km of wholly elevated legs, 7.869 km of wholly inferred surface legs and 23.127 km of mixed legs. Interchanging these figures with track-km would double-count most corridors.

## Sources and rebuild

The public passenger geometry and station IDs come from the DTPM March 2026 feed used for `web/data/transit_paths.json`. GTFS defines a parent station and its boarding locations in `stops.txt`. [DTPM feed](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip) · [GTFS reference](https://gtfs.org/documentation/schedule/reference/)

The OSM input is the same cached Geofabrik urban extract as the city building pack, with source data through 2026-11-20 21:21:21 UTC. Display © OpenStreetMap contributors. OSM data uses the Open Database License. [Geofabrik snapshot](https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf) · [OSM route relations](https://wiki.openstreetmap.org/wiki/Relation:route) · [Tunnel](https://wiki.openstreetmap.org/wiki/Key:tunnel) · [Bridge](https://wiki.openstreetmap.org/wiki/Key:bridge) · [Layer](https://wiki.openstreetmap.org/wiki/Key:layer)

Run from the repository root with Python 3 and osmium.

```sh
python3 -I tools/pack/metro_tracks.py data/raw/osm-buildings-city/urban.osm.pbf data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip
```
