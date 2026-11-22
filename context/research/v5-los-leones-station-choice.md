# Station choice during the Line 1 cut

The Metro scene removes all service. The 18 November 2024 cut checks whether the engine can also predict where entries change when some service remains. The current model applies the rail reach ratio after it allocates access to open stations. Los Leones receives enough new access to offset the loss of reach. DTPM counts at this station combine lines and do not identify rider destinations. [Engine diagnosis and frozen stations](v4-line1-model.md)

Station closure studies distinguish pre-trip choices about where to board from changes on the route. Combining rail and bus alternatives requires travel times and schedules. Red announced a bus loop for this event. The loop has no published stops or frequency in the checked notices. [Station closure model](https://pmc.ncbi.nlm.nih.gov/articles/PMC5147864/) · [OpenTripPlanner costs](https://docs.opentripplanner.org/en/v2.7.0/RouteRequest/) · [Local bus evidence](v4-line1-interruption.md)

## Two controls

I ran both controls on source base `d3ba30a9cf6c28e438fb1c091bbe060672806fb7` and its packed data. Both changed only the relative weights for the walking alternatives at a point. Both retained the existing rail reach multiplier on entry counts. Bus access kept its existing rule.

1. Multiply each open station's inverse-distance walk weight by its rail reach ratio.
2. Multiply it by its remaining share of scheduled boardable departures inside the request window. Count fixed GTFS departures at the stop call. For a frequency window, divide its overlap with the request window by the headway. This is a station access proxy. Frequency assignment in the literature uses the frequency of attractive routes to a destination. [Spiess and Florian](https://www.researchgate.net/publication/222385476_Optimal_Strategies_A_New_Assignment_Model_for_Transit_Networks)

| Walk weights | Four-cell error | Eight-cell error | Los Leones change at 13:30 / 14:00 |
| --- | ---: | ---: | ---: |
| Existing inverse distance | 0.3185 | 0.3464 | +133.1 / +128.5 |
| Inverse distance × rail reach ratio | 0.3310 | 0.3600 | +200.3 / +193.4 |
| Inverse distance × departure share | 0.3791 | 0.3517 | +60.5 / +58.4 |
| Observed event minus reference | | | −655 / −471 |

Errors are absolute differences in entry changes divided by the Thursday reference entries. The denominators are 3,368 and 9,933 entries. Los Leones changes are entries per half-hour. The five closed stations stay at zero in both controls. The rail reach control also kept zero entries across all 127 stations in the full Metro closure from 07:00 to 10:00. I discarded both controls because they increase both errors. [V5 work thread](https://discord.com/channels/1573514576400752640/1573863187672207411/1573878704856240169)
