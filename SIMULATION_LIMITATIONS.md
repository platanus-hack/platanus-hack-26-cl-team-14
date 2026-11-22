# Simulation limitations

What the santiaguito.cl engine can answer today, and what it cannot.
`tools/city-chat.sh` adds this file to the chat agent's prompt. Update it in the same PR when the engine gains or loses an output.
Sources: the Discord thread "Qué mide el motor" (1573907253805977710) and the engine code at main.

## How the chat must use this

- First ask what the person wants to measure, and offer to run the change. Do not start an answer with a limit.
- Do not say that the engine cannot do a change that its tools accept: buildings, streets, lines, stations, jobs, residents and events.
- Before a new experiment, offer only engine outputs that fit the change and that the tool result will contain.
- For a route activity, confirm whether the person wants initial edits, the activity, or both. Confirm the route, local start and end, arrivals, capacity, any stops, and what to measure. State what uses an assumption. Run the confirmed case with `simulate_city`. Use `create_replay` if the person asks to save it.
- If the person asks for a measure in "What the engine cannot answer", say so in one line and offer the nearest result that the engine can give.
- If `query_result` does not return a measure that a run returns (for example the daily cost by income band), offer to run the experiment with `simulate_city`.
- Never invent a number. Every number comes from a tool result.
- Say when a value is missing from the result (no coverage). Do not report it as zero.

## How a run works

- The base city models one typical weekday of Santiago in 48 half-hours. A route activity uses its own local start and end within one day. The model has no calendar date or sequence of days.
- People are synthetic and match the Census 2024 margins. They are not microdata.
- An experiment can apply initial city edits, run a route activity, or do both. Initial edits recompute the base city for that weekday. A route activity advances declared arrivals, travel, queues and visits in steps.
- A saved replay plays a stored result. It does not recompute.
- The scene time sets where the viewer starts. A temporary transit line edit uses its own `half_hours` window. A route activity uses its own minute schedule. Its motor access label describes the phase. It does not change ordinary car routes.

## Engine outputs

The engine produces these outputs. The chat sees a summary of them through `simulate_city`, and can compute over the listed fields with `query_result`. Offer a figure only if the tool result you received contains it.


- Metro entries by station and half-hour, baseline and scenario (126 stations). These are modeled entries, not unique people, alightings or riders of one line. Station bins carry no origin comuna and no split between lines.
- Work trips between comunas: baseline, scenario and change. Only up to 200 changed pairs come back. A missing pair is not zero, and a total over those rows is not a city-wide total.
- Jobs, working residents and public transport share by zone.
- Mode change of workers by age band and household size.
- Daily round-trip cost change by household income band, for workers who change mode and have a price before and after (Red fare or approximate fuel; no parking, tolls or value of time).
- Street edits: free-flow car minutes between fixed comuna points, before and after (no traffic, turns or local access). The chat can name a street section by its two cross streets; `road_sections` gives one before and after for the whole section. Before/after describes that one edit, possibly applied after earlier edits, not the original city against the final scenario.
- Transit service edits: public transport minutes between comunas, before and after, with the same per-edit meaning.
- `query_result` (#591): counts, totals, means, shares, percentiles and histograms over Metro entries, reported changed work pairs, changed shopping destinations and the times of one road or transit edit, for the current result or a saved replay. Pairs can have equal weights. Census 2024 public commuter weights apply only to transit pairs that match observed counts, the same for base and scenario; they are not traveller weights for road results. Each answer reports its unit, scope and coverage.
- Shopping and optional visits: trips that start or stop, and their destination comuna. A new shop point gives only a rough estimate of outbound trips to it.
- Study trips to higher-education campuses and health trips to DEIS health centres.
- Events (stadium, concert): attendance is an input, and origin shares can be supplied. Arrival mode, return comuna and timed routes exist only for events with supported audience and itinerary data (the MUT and Monumental case), not for any event.
- Mapocho ferry: work trips by direction.
- One synthetic person: whether the time and money allow an optional activity at place A or B.
- Route activity: modeled arrivals, exits and participants still present. The activity receipt also reports visit starts and person-minutes spent waiting or visiting. These counts follow the declared route rules. They cannot establish individual paths, ordinary store visits or sales.

## What the engine cannot answer today

- Road congestion, car travel time with traffic, or bus crowding.
- Emissions, noise, accidents, shop sales, rents or property prices.
- Observed attendance or ordinary visitor counts for one building. Jobs added to a building are modeled work stock.
- Door-to-door journey time of individual people. Route percentiles cover only matched changed pairs, not every traveller.
- Filters that join separate dimensions, for example shopping trips by origin AND mode AND hour. Each comes as a separate total.
- Effects across several days.
- Ferry riders do not leave the other modes; the ferry estimate is a separate output.
- Moving a shape alone creates no visitors or jobs.
