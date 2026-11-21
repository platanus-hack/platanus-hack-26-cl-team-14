# Compare the same spectators at two venues

The main pitch moves a Colo-Colo match between the Monumental and the MUT. Hold the spectator count, origin distribution, and event schedule fixed. Change the venue and recalculate travel options. This comparison answers how the same crowd would travel under the model.

## Prior work

- ActivitySim models whether people make non-work tours, where they go, when they travel, and how they travel in separate steps. Its destination code assigns a destination to a tour that already has a person. For a fixed match, supply the destination and attendance as scenario inputs. [ActivitySim frequency](https://activitysim.github.io/activitysim/v1.5.1/dev-guide/components/non_mandatory_tour_frequency.html) · [ActivitySim destination code](https://raw.githubusercontent.com/ActivitySim/activitysim/main/activitysim/abm/models/non_mandatory_destination.py) · [ActivitySim mode choice](https://activitysim.github.io/activitysim/v1.4.0/dev-guide/components/tour_mode_choice.html)
- MATSim can change the mode and route of an existing person's trip. A stadium application in Miami-Dade builds plans from event travel data. Its abstract describes an ongoing deployment. [MATSim replanning code](https://raw.githubusercontent.com/matsim-org/matsim-libs/main/matsim/src/main/java/org/matsim/core/replanning/strategies/ChangeTripMode.java) · [MATSim User Meeting 2026](https://matsim.org/conferences/mum2026/abstracts/MUM2026_paper_35.pdf)
- A Lisbon study used 15-minute counts from three Metro stations for 186 football events. It compared event and regular demand. This gives a precedent for measuring the station and time effects around matches. [Metro de Lisboa study](https://www.sciencedirect.com/science/article/pii/S2352146525002674)

## Smallest change for the pitch

Use one aggregate origin vector for the match. Prefer an event travel survey or aggregate ticket-holder origins. With population weights, call the origins an assumption. Preserve the same vector in both runs. Apply the available origin-to-venue mode and travel-time model, and state that it uses commuter preferences until event mode shares are available. Report differences by mode and comuna under those assumptions. [SECTRA Santiago EOD](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/) · [Current event contract](extra-event-contract.md)

For a check, first freeze the real match, the station set, the window around kick-off or the final whistle, and comparable dates without a match. Compare observed extra boardings with the model for the original venue. Report the observation coverage and error. The existing Parque O’Higgins check uses a different event and an assumed end time. It can guide the method. It does not validate the Monumental. [Project event comparison](event-attendance-april-2026.md)
