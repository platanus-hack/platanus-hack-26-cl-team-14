# Road surface and centerline generation

Generation builder: `1f0b82f8717d13492c5717a592ce71714f4a11b2`. Publication base: `2a396275de6253e311e7d2083a31843afc144225`. The surface and centerline builder files are byte-identical between these revisions. The later revision changes the report. This publication keeps both generated gzip files unchanged. No regeneration, tests or native builds were done for publication.

## Reproduction and source

AX downloaded `https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf`, 352,392,606 bytes. MD5 `fef09ad7ee6f28c7afc925ef68c737a4` matches the supplied value.

`osmium extract -s smart -b -70.85,-33.65,-70.45,-33.30 data/raw/osm-source/chile-261120.osm.pbf -o data/raw/osm-source/urban.osm.pbf`

Urban SHA256 `4a1cfdb3ffdfdb58cbc6247ab36cac38ad2dc35509b5c1e102a7e53461234cb3` matches the supplied value. Tools: osmium-tool 1.19.1; DuckDB 2.0.1 with Spatial. DuckDB builder uses 3 threads and 2 GB memory limit. Download/extract/build/report completed without process errors. Runtime was a few minutes including tools/source preparation; no formal peak-memory measurement.

Commands:

```sh
python3 -B tools/pack/road_surface.py data/raw/osm-source/urban.osm.pbf
python3 -B tools/pack/road_centerlines.py
python3 -B tools/pack/road_surface_report.py /tmp/vicente-road-cameras.csv
```

## Outputs

- `web/data/road_surface.geojson.gz`: 22,280 polygons; 105,164,999 inflated bytes; 24,068,787 gzip bytes; SHA256 `e4b546ae0e41e4ecd2e5e15b234fc820a431760f25bd3fc45ba75b6a6fc85509`.
- `web/data/road_centerlines.geojson.gz`: 148,202 motor ways; 148,216 parts; 61,284,083 inflated bytes; 7,685,681 gzip bytes; SHA256 `64bbe638386e60ce268464796daebcc2511f6b19e9ae567f50262009e9dd1c76`.
- Raw source directory uses 383 MiB; road scratch uses 368 MiB (disk allocations from du, not transfer sizes).
- Width source counts across 194,119 source ways: class_default 104,619 / 10,901.295 km; lanes 88,547 / 12,691.548 km; explicit width 953 / 215.651 km. Lengths use complete retained source ways, not only city-clipped geometry.
- Centerline inspection: all 148,216 `(osm_way_id, part)` keys unique; zero non-LineStrings, fewer-than-two-coordinate lines, unequal half-width fields, or vertices outside the city rectangle (1e-9 degree tolerance). This is structural data inspection, not source geometry validation.

## Findings and limits

Published surface: 359 occupied tiles, 71,112 holes. Zero invalid polygons, other geometry types, outside-tile geometry, or bad tile IDs. **51.561 unmatched seam metres** in the initial boundary metric. Later inspection classified all 24 line parts (51.560846 m) as contacts on matching rounded source-ribbon boundaries. These are one-sided contacts, not demonstrated crossing cracks. The largest is 39.757466 m at tiles 339/359, ground/foot/layer 0, latitude -33.31, longitude -70.454137 to -70.453710. This method does not prove all topology or routing connectivity correct.

Route report shows regressions: Baquedano car outside candidate 54.31% versus baseline 0%; Baquedano bus 20.51% versus 17.28%; Vespucio bus 17.44% versus 7.77%, car 3.66% versus 0.58%; Tobalaba bus 5.36% versus 4.77%. Full mode/camera metrics follow. Do not treat these data as accepted rendering.

All sampled raw same-level shared coordinates intersect their matching candidate level: zero outside across Baquedano ground 208, Maipu ground 93, Quilicura ground 24, Tobalaba ground128/tunnel1, Vespucio bridge8/ground37. Coincidence is not a verified routing edge.

No route samples exist for Maipu bicycle/walk; Quilicura car/bicycle/walk; Tobalaba bicycle; Vespucio bicycle/walk. Absence of samples is not evidence of zero error. Tobalaba camera coordinates are provisional and not verified against a Mac capture. Camera radii are 225 m.

Source lines are OSM geometry; buffers are display estimates. Explicit width, lanes times 3.2 m, and class defaults differ in source quality. Half-width symmetry is an assumption. Layers are relative levels, not physical heights. No measured deck elevation exists. Tagged-only sidewalks/cycle lanes are not separately modeled. Plan-view route hits can match another height; tunnels are excluded from visible candidate hits; sampled paths do not prove route legality or native appearance. The initial data report included no held-out/source-position survey or native rendering review. Later native observations and source limits are recorded below.

## Later diagnosis and display limits

The initial metrics below are retained as measured. The report accepts ground and
bridge plan-view hits, excludes tunnels, and does not assign a level to route
samples. At Baquedano, 392 car samples / 7,708.011 m of the 7,787.111 missed metres
are inside published tunnel polygons. The other four samples / 79.100 m are near
tunnel ramp way `38620496`, at 1.860–2.135 m from its axis versus a modeled
half-width of 1.6 m. This does not establish missing OSM connectivity.

Vespucio bus has 119 missed samples / 2,303.887 m outside all structures. The
nearest retained motor-road axis is 1.863–9.887 m away (median 3.964 m). All samples
are outside that nearest way's modeled half-width. The later report classifies
82 samples / 1,596.1 m as legacy-width coverage on the same raw way, 26 / 495.0 m
outside both displays, seven / 136.6 m as legacy-path-only, and four / 76.2 m within
15 cm of the modeled edge. These are width/route-position evidence categories,
not proof of the intended bus route or traffic permissions. The original versus
simplified GTFS coverage comparison must not be read as a surveyed pavement check.

At Encomenderos, tile 232, ways `461206863` and `243043658` share node `15073543`
and diverge by 29.5 degrees. Both have two lanes and no measured width; the
builder assigns 6.4 m. At `(-70.601567932,-33.418116948)`, the point is inside the
modeled road and its exterior, 0.183 m from its border. A 1 m grid within ±3 m
has five uncovered points inside the exterior and 44 covered points. Green
polygon `586669549` is 12.4 m away and does not contain the marked point. These
results do not prove a physical island or a pavement defect. No polygon correction
was made without measured pavement evidence. The lane-derived width remains an
estimate.

A later native review found continuous Baquedano road ribbons and smooth Vespucio
loop outlines. Vespucio's central crossing still showed no clear height separation
at that camera distance. Its physical grade and deck elevation remain unknown.
These image observations are bounded: the renderer revision also changed terrain
and movers, and moving objects were not time-matched. They do not establish
source accuracy, timing performance or legal routing. Inferred widths, bus-route
misalignment and unknown physical grades remain publication limits.

## Camera input

```csv
camera,lon,lat,radius_m
tobalaba,-70.60148,-33.41816,225
baquedano,-70.6352016,-33.4371169,225
vespucio,-70.7005600,-33.3663627,225
maipu,-70.7707490,-33.5099623,225
quilicura,-70.6967351,-33.3353952,225
```

## Full report

```csv
decoded_polygons,tiles,holes,invalid_polygons,other_geometry,outside_tile,bad_tile_ids
22280,359,71112,0,0,0,0
unmatched_seam_metres
51.561
camera,highway,width_source,ways,km_in_circle
baquedano,cycleway,class_default,11,0.981
baquedano,cycleway,width,5,0.436
baquedano,footway,class_default,84,3.613
baquedano,motorway,lanes,2,0.65
baquedano,motorway_link,lanes,3,0.23
baquedano,pedestrian,class_default,2,0.054
baquedano,primary,lanes,32,2.184
baquedano,residential,class_default,4,0.263
baquedano,residential,lanes,3,0.243
baquedano,secondary,lanes,14,0.753
baquedano,secondary_link,lanes,1,0.065
baquedano,service,class_default,6,0.119
baquedano,steps,class_default,9,0.082
maipu,busway,lanes,8,0.849
maipu,cycleway,class_default,2,0.395
maipu,footway,class_default,23,0.86
maipu,living_street,class_default,13,0.789
maipu,living_street,lanes,14,0.891
maipu,primary,lanes,8,0.825
maipu,primary_link,class_default,3,0.024
maipu,primary_link,lanes,1,0.048
maipu,residential,class_default,5,0.403
maipu,residential,lanes,2,0.393
maipu,secondary,lanes,6,0.33
maipu,service,class_default,9,0.251
maipu,tertiary,lanes,4,0.389
quilicura,secondary,lanes,2,0.451
quilicura,service,class_default,15,1.104
quilicura,tertiary,lanes,1,0.225
quilicura,unclassified,class_default,7,0.222
quilicura,unclassified,lanes,1,0.203
tobalaba,footway,class_default,73,4.571
tobalaba,pedestrian,class_default,3,0.18
tobalaba,primary,lanes,12,1.101
tobalaba,primary_link,lanes,2,0.103
tobalaba,residential,lanes,8,0.664
tobalaba,secondary,lanes,10,0.663
tobalaba,service,class_default,7,0.21
tobalaba,steps,class_default,3,0.018
tobalaba,tertiary,lanes,7,0.486
vespucio,footway,class_default,5,0.314
vespucio,motorway,lanes,16,1.8
vespucio,motorway_link,class_default,3,0.164
vespucio,motorway_link,lanes,19,2.823
vespucio,primary,lanes,5,0.459
vespucio,primary_link,lanes,3,0.226
vespucio,secondary,class_default,1,0.023
vespucio,secondary,lanes,12,1.208
vespucio,service,class_default,20,1.102
vespucio,unclassified,lanes,2,0.054
camera,mode,source,variant,samples,sampled_metres,metres_outside,pct_outside
baquedano,bicycle,local_osrm,baseline,45,876.1,507.4,57.91
baquedano,bicycle,local_osrm,candidate,45,876.1,409.3,46.72
baquedano,bus,gtfs,baseline,1056,20588.9,3557.2,17.28
baquedano,bus,gtfs,candidate,1056,20588.9,4222.8,20.51
baquedano,car,local_osrm,baseline,729,14339.3,0.0,0.0
baquedano,car,local_osrm,candidate,729,14339.3,7787.1,54.31
baquedano,walk,local_osrm,baseline,29,570.1,196.6,34.48
baquedano,walk,local_osrm,candidate,29,570.1,39.3,6.9
maipu,bus,gtfs,baseline,259,5121.3,3422.2,66.82
maipu,bus,gtfs,candidate,259,5121.3,2391.4,46.7
maipu,car,local_osrm,baseline,17,329.3,0.0,0.0
maipu,car,local_osrm,candidate,17,329.3,0.0,0.0
quilicura,bus,gtfs,baseline,96,1804.3,1326.9,73.54
quilicura,bus,gtfs,candidate,96,1804.3,45.8,2.54
tobalaba,bus,gtfs,baseline,504,9872.6,470.7,4.77
tobalaba,bus,gtfs,candidate,504,9872.6,529.6,5.36
tobalaba,car,local_osrm,baseline,477,9275.5,677.7,7.31
tobalaba,car,local_osrm,candidate,477,9275.5,0.0,0.0
tobalaba,walk,local_osrm,baseline,15,295.7,256.3,86.67
tobalaba,walk,local_osrm,candidate,15,295.7,39.4,13.33
vespucio,bus,gtfs,baseline,680,13210.5,1025.9,7.77
vespucio,bus,gtfs,candidate,680,13210.5,2303.9,17.44
vespucio,car,local_osrm,baseline,1011,19679.5,113.9,0.58
vespucio,car,local_osrm,candidate,1011,19679.5,719.3,3.66
camera,route_samples
baquedano,1859
maipu,276
quilicura,96
tobalaba,996
vespucio,1691
camera,structure,shared_coordinates,outside_baseline,outside_matching_candidate_level
baquedano,ground,208,117,0
maipu,ground,93,21,0
quilicura,ground,24,16,0
tobalaba,ground,128,72,0
tobalaba,tunnel,1,1,0
vespucio,bridge,8,0,0
vespucio,ground,37,10,0
```
