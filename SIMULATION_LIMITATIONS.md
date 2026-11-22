# Simulation limitations

What the santiaguito.cl engine can answer today, and what it cannot.
`tools/city-chat.sh` adds this file to the chat agent's prompt. Update it in the same PR when the engine gains or loses an output.
Sources: the Discord thread "Qué mide el motor" (1573907253805977710) and the engine code at main.

## How a run works

- One run models one typical weekday of Santiago, in 48 half-hours. There is no calendar date and no sequence of days.
- People are synthetic and match the Census 2024 margins. They are not microdata.
- An experiment changes properties (a building, a street, a line, a station, an event). The engine recomputes the whole day from the baseline.
- A saved replay plays a stored result. It does not recompute.
- The scene time only sets where the viewer starts. Only a temporary Metro closure uses a time window in the calculation.

## What the chat can offer as results

- Metro entries by station and half-hour, baseline and scenario (126 stations).
- Work trips between comunas: baseline, scenario and change (up to 200 changed pairs).
- Jobs, working residents and public transport share by zone.
- Mode change of workers by age band and household size.
- Daily round-trip cost change by household income band, for workers who change mode and have a price before and after (Red fare or approximate fuel; no parking, tolls or value of time).
- Street edits: free-flow car minutes between fixed comuna points, before and after (up to five pairs per edit in the chat summary; no traffic, turns or local access).
- Transit service edits: public transport minutes between comunas, before and after.
- Shopping and optional visits: trips that start or stop, and their destination comuna.
- Study trips to higher-education campuses and health trips to DEIS health centres.
- Events (stadium, concert): attendance as an input, attendees by origin and return comuna, arrival mode and arrival times.
- Mapocho ferry: work trips by direction.
- One synthetic person: whether the time and money allow an optional activity at place A or B.

## What the engine cannot answer today

- Road congestion, car travel time with traffic, or bus crowding.
- Emissions, noise, accidents, shop sales, rents or property prices.
- Visitors of one specific building, unless the change gives it jobs or an event with an audience.
- How many people take more than N minutes door to door, as a share of travellers (needs weighted travel-time output).
- Effects across several days.
- Ferry riders do not leave the other modes; the ferry estimate is a separate output.
- Moving a shape alone creates no visitors or jobs.

## How the chat must use this

- Before a new experiment, offer only results from "What the chat can offer" that fit the change.
- If the person asks for something in "What the engine cannot answer", say so in one line and offer the nearest result that the engine can give.
- Never invent a number. Every number comes from a tool result.
