# Editable Metro and bus services

Use the catalog to select a service by its GTFS route, direction and stop pattern. A change to one pattern leaves parallel services and other patterns on the same line available. The source is Red Metropolitana feed V161.20260314, valid from 21 March to 31 December 2026. It covers seven operating Metro routes and 416 bus routes. The two rail services coded as tram in this feed are outside this catalog. Future Metro projects have a separate [planned network](metro_planned.md).

## Inputs

The GTFS trips table links each trip to a route, direction, shape and service calendar. The stop times table orders its calls. Use the next arrival minus the previous departure for in-vehicle segment time. Use departure minus arrival at the same stop for dwell time. An express segment can pass several intermediate stations. Bus frequencies give start and end times and the number of seconds between departures. All bus frequency rows in this feed have `exact_times=0`, so they describe intervals. Times after 24:00 belong to the preceding service day. [GTFS Schedule Reference](https://gtfs.org/documentation/schedule/reference/) · [DTPM source ZIP](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip)

The Metro platform parent IDs match all 126 stations in the engine. For example, both Line 1 and Line 4 serve Tobalaba, parent ID `TB-TOB`. The catalog preserves separate directional patterns for local, express and short services. The feed describes scheduled service. It gives no observed waits, delays, passenger loads or ridership response to an edit.

## Reproduce the file

Download the DTPM ZIP once and keep it outside the repository. Its SHA-256 is `f10945e80beebc39ea2ac70de997d3d69714897b575de6fbf4e3ca643013b77b`.

From the repository root, run:

```sh
python3 tools/pack/editable_transit.py --gtfs /path/to/GTFS_20260321_v3.zip
```

## Read a pattern

Look up the edit target in `lines`. Each key combines the GTFS route ID, direction, shape and a hash of the ordered stops and boarding rules. The hash distinguishes two trips that serve the same stops with different pickup restrictions. Each stop ID resolves in `stops`. Each shape ID resolves in `shapes` to ordered WGS84 longitude and latitude pairs. A short trip can share the full route shape. Use its first and last calls to identify the portion it serves.

| Feed records | Bus | Metro |
| --- | ---: | ---: |
| Routes | 416 | 7 |
| Editable patterns | 839 | 389 |
| Distinct stops or parent stations | 12,060 | 126 |
| Frequency windows | 14,436 | 0 |
| Fixed departures | 0 | 11,528 |

`segment_minutes` has one entry per consecutive pair of calls. `dwell_minutes`, `pickup_type` and `drop_off_type` have one entry per stop. The builder selects a representative trip with the lower median scheduled end-to-end time among the pattern’s trip records. The first trip ID in lexical order breaks a tie. Each template counts once. These times describe that single trip.

`headway_minutes` summarizes the hour from 08:00 inclusive to 09:00 exclusive on Monday 13 April 2026, at the first stop. The denominator counts fixed departures and expected starts from each active frequency window. A window contributes its overlap in seconds divided by its headway in seconds. A null value means zero starts in that hour. The calendar, date exceptions, fixed departures and frequency windows support other dates and times. The JSON `definitions` field gives the units and tuple layouts.

For example, bus pattern `101:0:101I:bca9b5d49c23a375` starts at PB1. Its first segment to PB2 takes 1.517 scheduled minutes on the representative trip. Its mean departure interval for the reference hour is 12.85 minutes.
