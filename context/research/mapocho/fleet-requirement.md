# Fleet implied by the Mapocho replay

The draft uses 10 hypothetical piers, 17 km/h, one minute at each intermediate pier, and departures every 10 minutes in each direction. The sum of the nine straight pier-to-pier distances is 21.86 km. The one-way run takes 77.14 minutes in motion and eight minutes at intermediate piers. The two-way run takes 170.27 minutes before terminal time. [Pier coordinates](piers.json) · [Distance and dwell method](../../../tools/pack/mapocho_boat_trips.py)

Divide the round-trip cycle time by the headway and round up. Include terminal time in the cycle. This assumes vessels can turn at both ends and alternate directions. [TCRP Transit Scheduling Manual, Part B](https://onlinepubs.trb.org/Onlinepubs/tcrp/tcrp_rpt_30-b.pdf)

| Headway in each direction | Active vessels with zero terminal time | Active vessels with five minutes at each terminal |
| --- | ---: | ---: |
| 5 minutes | 35 | 37 |
| 10 minutes | 18 | 19 |
| 15 minutes | 12 | 13 |

At a 10-minute headway, 18 vessels allow 9.73 minutes of terminal time per round trip. Five minutes at each terminal is a sensitivity case. The replay can show the 18-vessel lower bound if it also gives the zero-terminal-time assumption. At 162 passenger places per vessel, six departures per hour supply up to 972 places per hour across any one segment in each direction. Total boardings across all piers depend on where riders get off.

The estimate excludes spare vessels and uses straight segments. A river-following route, safe speeds, docking and recovery can change the cycle time. A service window also needs a start-of-service timetable or an explicit assumption that the fleet is already in position. [Mapocho navigability evidence](../mapocho-navegable.md)
