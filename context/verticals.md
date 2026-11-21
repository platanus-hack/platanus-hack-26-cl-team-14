# Verticals

There are three verticals. Each vertical has one team and one Discord thread.
Each team plans and takes its own tasks.
The golden rule still applies: every task must serve a moment in `context/pitch.md`.

**A vertical is done only when the humans say it is finished.** The "Done when" criteria guide the work. Meeting all of them does not finish the vertical. When you think a vertical is ready, post it in `#resultados` and ask the humans. Keep working until they confirm.

## Contract between the verticals

- **V1 owns the model and the engine.** It defines the entity types, their properties, and the general engine that turns property changes into results.
- **V2 owns how the city looks.** It draws every entity that V1 models.
- **V3 owns the demo.** It sets entity properties through the MCP, makes the replays with the engine, and shows them on the V2 map.

When a vertical needs something from another vertical, ask in that vertical's thread. Do not build it yourself.

## V1 · Entity modeling

**Goal:** Model each entity in the city with real, declared properties. The properties control how the entity responds to an experiment.

**People come first.** People are the most important entity. Each scenario is a story about people, so model them first and in the most depth.

- **Who they are:** age, household, and income, drawn from census data for their zone.
- **Where they go:** home, work or study, and the other places in their day.
- **How they travel:** mode, route, and departure time. People choose again when a place or a route changes.
- **How they react:** when an experiment changes their day, they change their plan. This reaction comes from their properties, not from a rule for one scenario.

**Places come second.** Model them only as much as people need: capacity, opening hours, and who they attract. Examples are stadiums, schools, malls, metro stations, streets, and the river. The list is not limited to these. If an entity exists in the city, model it.

An experiment changes properties. The effect comes from the properties alone. Do not write rules for one scenario.

The engine does not run during the pitch, so it does not need to be fast enough for a live run. It must be correct when the team makes the replays.

**Start from:** research. Follow the "Research first" rule in `AGENTS.md`. Read the briefs that are already in `context/research/` first. Save each new brief there, with links to its sources.

**Done when:**
- The humans say V1 is finished.
- People have declared properties from real data sources, and their daily trips match the real origin-destination data for Santiago.
- Each other entity type has declared properties, and each property has a real data source.
- The three demo scenarios use only changes to entity properties. No scenario code.
- The fixed score against real data improves.
- The "close the full metro" result is compared with real data from the October 2019 metro closure, and the comparison is in the demo.

## V2 · City interface

**Goal:** The city looks real.

- Cars, people, metro trains, and boats have high-quality sprites.
- A viewer can see the difference between a street, a sidewalk, a park, the river, and a building.
- The design is clear on a projector: large shapes, strong contrast, little text.

Human taste decides this vertical. Agents decide the technical work. Show visual changes to the humans early and often. Ask one taste question in each release.

**Done when:**
- The humans say V2 is finished.
- A person who does not know the project recognizes Santiago and understands each element without help.
- No car moves on the river. No boat moves on a street.

## V3 · Three-scenario demo

**Goal:** An excellent demo of the three agreed scenarios:

1. Replace the **MUT** with the Estadio Monumental.
2. Make the Mapocho river navigable.
3. Close the full metro.

**The demo shows only cached replays.** The engine does not run during the pitch. Before the demo, make each replay with the engine, watch it, fix it, and make it again. Repeat until each scenario is smooth from start to end.

**The demo flow:**

1. The presenter types the question in a terminal inside the map view.
2. The agent uses the MCP to show the experiment: which entities change, the start time, the duration, and what to analyze. The agent shows these assumptions before the replay.
3. The agent opens the cached replay for that scenario.
4. The replay zooms to the important areas, tells what occurred, and shows charts of congestion and people flow. The terminal hides during the replay and comes back at the end.

**The MCP has three replay tools:**
- **List replays** and **run a replay** are used in the demo.
- **Create a replay** is used before the demo. The general engine (V1) calculates the scenario and saves it in the cache. Do not write code for one scenario.

**Done when:**
- The humans say V3 is finished.
- Each of the three cached replays runs from question to final chart with no visual errors.
- The demo opens the three cached replays, then shows **‹N›** simulations side by side.
- The full demo fits in the 3-minute pitch.

## Open

- The number ‹N› of side-by-side simulations.
- If V1 owns the engine.
