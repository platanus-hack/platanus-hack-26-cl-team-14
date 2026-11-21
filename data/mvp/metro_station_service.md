# Station entries per scheduled place

This proposal compares station entries with the places on trains that a rider could board at that station. The station model supplies the entries. No passenger routing, occupancy estimate or saturation threshold is part of the rule. [Proposal recorded before calculation](https://discord.com/channels/1573514576400752640/1573688249287708682/1573696074084778066)

## Compute the indicator

1. Select the station and local half hour in `metro_station_service.csv`. A row gives the mean number of departures that allow boarding, by line and direction. Sum across both directions and all lines served by the station when the numerator covers the whole station.
2. For each row, multiply `trains` by `places_per_train` for that line from `metro_capacity.csv`. Sum the products. Use an unknown result if any service in the sum lacks a capacity or uses an incompatible crowding standard. A half hour with no service also has no ratio.
3. Divide entries by scheduled places. Label the result “station entries per scheduled place.” Mark whether the numerator is observed or modeled. A missing observation gives no ratio. For a scenario that changes the timetable, calculate the denominator for that timetable before reporting the indicator.

A train may arrive with passengers already on board. The station entry count misses riders who transfer within the paid area. Riders can also enter in one interval and board in the next. The ratio cannot tell whether a train or platform is crowded. The engine would need passenger paths, alightings and loads by train segment to estimate onboard crowding.

## Station service

The source covers the five weekdays of April 13–17, 2026. GTFS service L ran all five days, LJ ran four and V ran on Friday. The calendar has no exceptions on these dates. Each trip contributes one call per station where riders can board and continue. The builder uses platform departure time, the platform's parent station, and the trip's actual stop sequence. It excludes calls with pickup prohibited and the trip's last call. Line departures at the starting station alone would put some trains in the wrong half hour and would count short trips at stations they never visit. [DTPM March GTFS](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip)

The table covers all 126 station IDs in the observation file. It contains only positive scheduled counts. Within this feed, an absent station, line, direction and time combination has zero boardable departures. Train counts can be fractional because they average five days. Counts describe scheduled service. The feed gives no cancellation or delay observations.

Run `python3 tools/pack/metro_station_service.py` from the repository root. Pass the path to the original feed as the first argument to read it from another checkout.
