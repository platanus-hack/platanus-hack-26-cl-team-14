# Simulation limitations

What the santiaguito.cl engine can answer today, and what it cannot.
`tools/city-chat.sh` adds this file to the chat agent's prompt. Update it in the same PR when the engine gains or loses an output.
Sources: the Discord thread "Qué mide el motor" (1573907253805977710) and the engine code at main.

## How a run works

- One run models one typical weekday of Santiago, in 48 half-hours. There is no calendar date and no sequence of days.
- People are synthetic and match the Census 2024 margins. They are not microdata.
- An experiment changes properties (a building, a street, a line, a station, an event). The engine recomputes the whole day from the baseline.
- A saved replay plays a stored result. It does not recompute.
- The scene time only sets where the viewer starts. Only temporary transit line edits (Metro, bus and other lines) use a time window (`half_hours`) in the calculation.

## Engine outputs

The engine produces these outputs. The chat sees a summary of them through `simulate_city`, and can compute over the listed fields with `query_result`. Offer a figure only if the tool result you received contains it.


- Metro entries by station and half-hour, baseline and scenario (126 stations). These are modeled entries, not unique people, alightings or riders of one line. Station bins carry no origin comuna and no split between lines.
- Work trips between comunas: baseline, scenario and change. Only up to 200 changed pairs come back. A missing pair is not zero, and a total over those rows is not a city-wide total.
- Jobs, working residents and public transport share by zone.
- Mode change of workers by age band and household size.
- Daily round-trip cost change by household income band, for workers who change mode and have a price before and after (Red fare or approximate fuel; no parking, tolls or value of time).
- Street edits: free-flow car minutes between fixed comuna points, before and after (no traffic, turns or local access). Before/after describes that one edit, possibly applied after earlier edits, not the original city against the final scenario.
- Transit service edits: public transport minutes between comunas, before and after, with the same per-edit meaning.
- `query_result` (#591): counts, totals, means, shares, percentiles and histograms over Metro entries, reported changed work pairs, changed shopping destinations and the times of one road or transit edit, for the current result or a saved replay. Pairs can have equal weights. Census 2024 public commuter weights apply only to transit pairs that match observed counts, the same for base and scenario; they are not traveller weights for road results. Each answer reports its unit, scope and coverage.
- Shopping and optional visits: trips that start or stop, and their destination comuna. A new shop point gives only a rough estimate of outbound trips to it.
- Study trips to higher-education campuses and health trips to DEIS health centres.
- Events (stadium, concert): attendance is an input, and origin shares can be supplied. Arrival mode, return comuna and timed routes exist only for events with supported audience and itinerary data (the MUT and Monumental case), not for any event.
- Mapocho ferry: work trips by direction.
- One synthetic person: whether the time and money allow an optional activity at place A or B.

## What the engine cannot answer today

- Road congestion, car travel time with traffic, or bus crowding.
- Emissions, noise, accidents, shop sales, rents or property prices.
- Visitors of one specific building. Jobs added to a building are modeled work stock, not visitors.
- Door-to-door journey time of individual people. Route percentiles cover only matched changed pairs, not every traveller.
- Filters that join separate dimensions, for example shopping trips by origin AND mode AND hour. Each comes as a separate total.
- Effects across several days.
- Ferry riders do not leave the other modes; the ferry estimate is a separate output.
- Moving a shape alone creates no visitors or jobs.

## How the chat must use this

- Before a new experiment, offer only engine outputs that fit the change and that the tool result will contain.
- If the person asks for something in "What the engine cannot answer", say so in one line and offer the nearest result that the engine can give.
- Never invent a number. Every number comes from a tool result.
- Say when a value is missing from the result (no coverage). Do not report it as zero.
