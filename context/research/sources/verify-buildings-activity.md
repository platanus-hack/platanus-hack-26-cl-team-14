# Verification: buildings and activity in Providencia

Checked 2026-11-21. Counts refer to source records, not a ground-truth count of physical buildings. A height or floor tag is not proof of a measured value.

## Recommendation for the 3D scene

Use **Overture Buildings** as the provisional footprint layer: it combines OSM and ML footprints and merges available height attributes. Use its height or floor values where present, and evaluate the Santiago residential exposure model for matching tall buildings. Mark estimated or default heights as estimates. This choice follows the documented conflation, not a verified local comparison: the Overture count and matching accuracy in Providencia remain unknown. Do not sum heights or claim visual height is measured.

| Footprint source | Providencia records | With height or floors | Verdict |
|---|---:|---:|---|
| Microsoft Global ML | 10,359 (approximate boundary assignment) | 0 positive heights; no floor field | OK, rows checked |
| Google Open Buildings v3 | Unknown | No height or floor columns in v3 | OK for footprints; total unverified |
| OSM | Unknown | Unknown | UNVERIFIED: query did not return |
| Overture Buildings | Unknown | Unknown | UNVERIFIED: local records not obtained |

## Microsoft Global ML Building Footprints

- [Index](https://minedbuildings.z5.web.core.windows.net/global-buildings/dataset-links.csv); [Chile partition](https://minedbuildings.z5.web.core.windows.net/global-buildings/2026-02-03/global-buildings.geojsonl/RegionName=Chile/quadkey=210231031/part-00158-4feead82-d499-422b-94cb-c036c212127a.c000.csv.gz); [Providencia OSM boundary](https://nominatim.openstreetmap.org/search?q=Providencia%2C%20Chile&format=jsonv2&polygon_geojson=1).
- VERIFIED-ROWS: HTTP 200, 95,803,584 compressed bytes. Chile quadkey 210231031, release 2026-02-03. Records contain GeoJSON geometry, `height` and `confidence`; an example near -70.58816, -33.43697 has `height=-1`. All 10,359 selected records had `height=-1`; no floors field. Release date is not imagery date.
- Method: both corners of the comuna bounding rectangle fall in this quadkey. Test the arithmetic mean of each exterior polygon's vertices against OSM relation 162983. This is an approximate point assignment, not polygon intersection; boundary cases and actual detection accuracy were not checked. Role: INPUT.

## Google Open Buildings v3

- [S2 tile index](https://openbuildings-public-dot-gweb-research.uw.r.appspot.com/public/tiles.geojson); [tile 967](https://storage.googleapis.com/open-buildings-data/v3/polygons_s2_level_4_gzip/967_buildings.csv.gz).
- VERIFIED-ROWS: HTTP 206, first 1,048,576 compressed bytes of a 949,864,757-byte tile. Fields: `latitude`, `longitude`, `area_in_meters`, `confidence`, `geometry`, `full_plus_code`. One record near central Providencia: -33.43275888, -70.61874208, area 214.0222 m², confidence 0.9223. Coordinates indicate location; exact boundary membership not tested. No height or floor fields. Total cannot be inferred from the sample. Imagery date varies by location; do not assert a uniform 2023 date. Role: INPUT.

## OSM buildings and activity places

- [Geofabrik Chile](https://download.geofabrik.de/south-america/chile.html); [building levels definition](https://wiki.openstreetmap.org/wiki/Key:building:levels).
- UNVERIFIED: bounded Overpass queries did not return. The national PBF is listed at 336 MB, over the 100 MB limit. Neither building counts nor activity rows were read. `building:levels` excludes roof and underground floors. Count the union of height and levels tags once per object; OSM objects are not necessarily unique buildings. Role: INPUT.

## Overture Buildings

- [Buildings guide](https://docs.overturemaps.org/guides/buildings/).
- VERIFIED-BYTES: documentation opened HTTP 200; no Providencia records read. The guide says OSM has priority, Google and Microsoft fill coverage, and matched height attributes can be merged. Local coverage, values, release date and count remain unverified. Role: INPUT.

## Overture Places

- [Places guide](https://docs.overturemaps.org/guides/places/).
- VERIFIED-BYTES: documentation opened HTTP 200; no Providencia records read. It describes point records with name, taxonomy, geometry and confidence; the 2026 schema uses `taxonomy` and `basic_category`. Places does not include OSM data and does not provide visits. Local count and currentness are unverified. Role: INPUT.

## Santiago residential exposure model

- [Model and documentation](https://aleaicr.github.io/data/exposureModel_ch_2026/); [release ZIP](https://github.com/aleaicv/ExposureModels/releases/download/santiago_chile_ch_2025/exposure_santiago.zip).
- VERIFIED-ROWS: HTTP 200, 1,549,365 bytes. The DBF has 9,793 records; 1,617 have `Comuna=providencia`. The first Providencia record has `height=42.9`, `stories=13`, `year=1985`; columns include `height`, `stories`, `dwellings`, `population`, `year`, `Comuna`. Model reference year is 2023. Role: INPUT.
- Scope is residential buildings of at least five stories. These are model attributes, not independently measured heights. Check methodology, license and footprint matching before treating them as ground truth or merging them with Overture.

## Google Open Buildings 2.5D Temporal

- [Official description](https://sites.research.google/gr/open-buildings/temporal/).
- UNVERIFIED: no Santiago raster cells read. The published 2016–2023 annual height values are predictions, not measured building heights; the stated evaluation does not establish accuracy in Chile. Its roughly 4 m grid is not a building footprint. Role: INPUT.

## Critique

1. Only Microsoft has a bounded footprint count; its approximate boundary test can misassign edge buildings.
2. OSM and Overture requests did not yield local rows or counts. No defensible cross-source ranking by coverage or height is available.
3. Activity places and predicted-height rasters lack local samples. The residential model's values and license were not independently validated.
