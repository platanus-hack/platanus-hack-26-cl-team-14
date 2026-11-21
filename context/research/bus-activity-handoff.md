# April bus activity for L3

`data/mvp/bus_activity.csv` contains all BUS observations for LABORAL, the average of April 13–17, 2026. It excludes Metro and Tren. The source and exact joins were inspected in the [verified transport research](santiago-transport-verified.md) and [offline export](transport-export-handoff.md). This file is a nearby boarding activity proxy, not feeder alightings or observed transfers. Validation-derived boardings exclude estimated fare evasion and are not unique people.

One row per source stop: `user_stop,internal_stop,comuna,gtfs_stop_id,stop_lat,stop_lon,join_status,b00,...,b47`. Bins are local America/Santiago half hours: b00 is 00:00 and b47 is 23:30. Values retain the original numeric text without rounding. An empty bin is a missing observation, never an inferred zero. Empty coordinates mean the stop cannot be located. The fixed source export has no duplicate keys; this compactor targets that verified publication.

`bus_metro_300m.csv` has `stop_id,parent_id,distance_m`. Join its stop ID to `bus_activity.csv.gtfs_stop_id`, then unpivot the 48 bins if long rows are needed. Distances use Haversine with Earth radius 6,371,000 m to the verified March GTFS Metro parent points in `metro_observations.json`. This is not distance to an entrance or a walking route. The crosswalk retains every parent within 300 m, including overlaps. The model owner must declare how shared stops contribute; this export does not split or multiply their boarding counts.

## Reproduce offline

First run the existing complete workbook exporter as documented in the linked handoff. Then:

```sh
python tools/scrape/export_bus_activity.py \
  data/raw/april-boardings data/mvp/metro_observations.json data/mvp
```

No downloads or extra packages. The manifest records source URLs, SHA-256 hashes, sizes, source period, missingness, counts, unmatched mass, and output hashes.

## Actual source audit, 2026-11-21

- 11,932 unique source stop rows, 354,267 observed bins, 218,469 empty bins. No source stop or observed bin was dropped. CSV size 3,263,141 bytes.
- Total 2,512,170.4 average weekday boardings. The one unlocated row (`SIN_SIMIT / SIN_PARADERO`) retains 38 observations and 790.4 boardings. It has no crosswalk entries. The 11,931 located rows retain 2,511,380 boardings.
- Local workbook SHA-256 `2982459cbe71d139000be6535a24534a68f53162a2550f1aba4b72cc61c2e90d` and GTFS SHA-256 `f10945e80beebc39ea2ac70de997d3d69714897b575de6fbf4e3ca643013b77b` match the verified source manifest. Recomputed compact-file observation count and decimal total match the complete export; every located coordinate matches the exact GTFS stop ID.
- Crosswalk: 1,180 links, 1,172 distinct bus stops, all 126 Metro parents. Eight bus stops link to two parents. Maximum included distance 299.849 m. The other located bus stops remain in the complete activity file even though they have no nearby Metro link.
- The complete workbook export already reconciles publisher cached totals within floating-point precision. Its input CSV hash is retained in the new manifest. No new source download, model fit, scoring, or tests.

April bus and Metro observations share the DTPM measurement process and target period. An L3 score would measure association with concurrent activity, not independent causal validation or a future-demand forecast.
