# River boat as an engine entity

Owner: bold-boi (V1). Thread: [V1 · Modelado de entidades](https://discord.com/channels/1573514576400752640/1573809708047798364/1573810024105517178).

## What the engine models

A boat line is a general transit line (`kind:"line"`, `op:"add"`) with `mode:"boat"`. The Mapocho scenario is only a property change: add two directed patterns. The engine has no Mapocho rule.

| Property | Value for the Mapocho run | Source |
| --- | --- | --- |
| Piers (`new_stops`) | MAP01–MAP10, one about every 2 km on the river axis | [piers.json](mapocho/piers.json). **Assumption**: no pier plan exists ([brief §1](mapocho-navegable.md#1-previous-projects)). |
| Speed → `segment_minutes` | 17 km/h on the distance along the river centerline (OSM relation 7548664, 22.79 km), 80.4 min end to end | Brisbane CityCat mean operating speed, Tanko & Burke 2019, as cited in [brief §7](mapocho-navegable.md#7-second-engine-test-the-boat-as-faster-travel-minutes). |
| `dwell_minutes` | 1 at each intermediate pier | Same assumption as `mapocho_boat_trips.py`. |
| `headway_minutes` | 10 | Thames Clippers runs a boat every 10–20 min ([brief §4](mapocho-navegable.md#4-comparable-river-buses)); 10 is the low end. |
| `vehicle_capacity` | 162 passengers | Brisbane CityCat 2nd and 3rd generation ([Wikipedia, RiverCity Ferries](https://en.wikipedia.org/wiki/RiverCity_Ferries)). |
| Season | not modeled | Brief §2 gives an assumed September–April season. A replay must state the month. |

Input file: [mapocho/boat_line.json](mapocho/boat_line.json) (two changes, east and west), written by `python3 -I tools/pack/mapocho_boat_line.py`. Each change carries a `geometry` that follows the river centerline; all 746 vertices are inside the mapped water polygons of `web/data/mapocho_detail.geojson`.

## How people choose it (`engine/src/line_riders.rs`)

For each synthetic worker, the engine finds the best boarding stop within 1 km of home and a later alighting stop within 1 km of the job (straight line, 80 m/min, the same walk rules as `editable_transit/routing.rs`). Door-to-door minutes = walk + headway/2 + scheduled ride + walk.

The added line is a fourth alternative in the fitted mode logit of `choice.rs`. It uses the public-transport minute coefficient and the public residual of the comuna pair, with no mode constant of its own. Thus at equal minutes it is as attractive as the existing public mode. A Sydney stated-preference study values a ferry over a bus at about 17 in-vehicle minutes (Daniels & Streeting 2002); this is not used, because it has no Santiago value.

Riders per half hour = demand × the weekday public departure share (`work_departures.csv`). The cap applies to the busiest segment: its on-board riders per half hour stay at or below departures × `vehicle_capacity`. All riders share one departure profile.

Limits:
- Workers only, trips to work only. Students and return trips are not counted.
- The comparator is the comuna-pair public time (`choice.minutes`), not a person-level public trip. This favors people who live next to a pier.
- Riders do not feed back into Metro station boardings or job choice.
- New stops are not in the comuna-pair transit graph, so the boat line changes no comuna-pair minutes.

## Check against real data

Run: engine at this branch, `POST /api/scenario` with the two changes in `boat_line.json` (weekday, data pack `data/mvp`).

| Measure | East (MAP01→MAP10) | West (MAP10→MAP01) |
| --- | ---: | ---: |
| Workers with a usable pier pair (1 km each end) | 11,094 | 4,173 |
| Share where the boat beats the comuna-pair public time | 7.7 % | 6.5 % |
| Demand, trips to work per weekday | 3,510 | 942 |
| Riders after the segment capacity cap | 3,470 | 932 |
| Taken from public / car / other | 2,439 / 451 / 580 | 416 / 245 / 271 |
| Busiest segment, modeled daily on board | MAP06→MAP07, 2,367 | MAP07→MAP06, 278 |
| Peak half hour (07:00–07:30), modeled demand on the busiest segment / seats | 404 / 486 = 83 % | 47 / 486 = 10 % |
| Busiest boarding / alighting pier | MAP06 1,159 / MAP08 1,476 | MAP08 212 / MAP06 242 |

EOD 2012 reference ([mapocho/boat_trips.md](mapocho/boat_trips.md), same 17 km/h and 10 min): the boat beats the reported time on **36.8 %** of 51,929 weekday trips between any two piers (800 m each end), and 45.4 % when the piers are 3 km or more apart.

Reading: the engine's faster share (6–8 %) is far below the EOD share. This does not prove the comparator is unbiased, because the two measures differ: EOD compares with the time each person reported for any mode and purpose; the engine compares with the modeled public time of the comuna pair, for workers only, and includes short one-pier hops. The engine result is a modeled demand, not a forecast: no Mapocho boat survey exists.

Story for the replay: about 4,400 trips to work per weekday. Modeled demand on the busiest eastbound segment (Santiago centro → Providencia) is 83 % of seats at 07:00–07:30; the westbound boat stays at 10 % or less.
