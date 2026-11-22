# Monumental Metro departures

The MUT scene shows a Colo-Colo crowd at two venues. The V5 check compares estimated Metro entries near the original venue with public station counts. The event request supplies the venue, attendance, audience, and end half-hour.

The FHWA planned-event guide estimates attendance, departure rate, and mode share as separate inputs. It also treats access and capacity as separate constraints. This layer estimates station entries on the baseline network. It has no capacity or service-change model. [FHWA travel demand](https://ops.fhwa.dot.gov/publications/fhwahop07108/event_impacts.htm#travel_demand)

For each modeled home comuna, use the EOD normal-Saturday Metro share from the venue sector to the home sector. Count all reported trip modes that contain Metro, including combined modes. The weights multiply the person and trip factors. Assign each home to the station within 1 km that minimizes the straight-line distance from venue to station to home centroid. This approximates the direction of travel. It does not use actual walking paths or Metro routes. The 2012 survey gives general trip shares. It has no event-specific audience share. [EOD source and codebook](https://users.dcc.uchile.cl/~egraells/gds-data/eod2012.tgz)

Fit the capture fraction (0.689) and four half-hour shares (0.068, 0.516, 0.305, 0.111) to April. Capture is the observed excess at both stations divided by the unscaled survey estimate. The profile uses the share of excess in each bin. The bins start one half-hour before the supplied end bin. For November, use 22:00 as the end bin because the match ended at 21:56. The 21:30 bin includes the final whistle. The two matches differ in audience and time. [Handoff and fit](https://discord.com/channels/1573514576400752640/1573863187672207411/1573875898694766804) · [DTPM counts and match sources](monumental-match-boardings.md)

Run `python3 tools/v5/run_evals.py`. The score divides absolute entry-count error by total observed entries across Pedrero and Mirador. It adds the estimated event entries to the same Saturday reference used for the no-event score.

| Case | Cells | Model | No event |
| --- | ---: | ---: | ---: |
| 26 April 2025, fitted | 8 | 0.2409 | 0.6189 |
| 8 November 2025, descriptive check | 6 | 0.4135 | 0.6542 |

Both references are from other years. The November reference falls on the eve of a runoff. Scheduled service was comparable for the measured window, but actual service and background demand are unknown. The model estimates 434 entries at Pedrero at 22:00 in November. DTPM records 814. Use these results as development evidence with the reference dates visible.
