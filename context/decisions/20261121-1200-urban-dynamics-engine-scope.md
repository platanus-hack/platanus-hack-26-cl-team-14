# The simulator is one general engine for urban social dynamics in Santiago

- Status: Accepted
- Date: 2026-11-21
- Source: `meetings/2026-11-21-scope-simulador-urbano.md`

## Decision

We build one general engine for urban social dynamics in Santiago: transport, housing, work, entertainment, and culture. It is not a collection of separate simulations.

## Why

The team agreed in the meeting. If the engine accepts physical events (earthquakes, wind, fire), the swarm collects one simulation for each event and the result is a "collage". One engine that answers many questions is the differentiator.

## What it means for agents

- In scope: questions about how groups of people move and change in the city. Example: "Remove the MUT and put a vertical ghetto there. What happens at Tobalaba station?"
- Out of scope: physics simulations, individual psychology, and visuals of destructive events.
- Model people at mass level. Target scale: all of Santiago, one agent per person (about 7 million). A smaller area (one or more comunas) is permitted if necessary. Agents decide how to handle the border effects.
- The 3D model is only the UI. The engine models housing units and people, not buildings as physical structures.
- The output is structured data that supports a real decision (example: where to put a stadium).
- The pitch shows three very different use cases with the same engine.


## User experience and implementation priority

Juan clarified the interaction on November 21. A user asks a free-form what-if question. The question agent maps it to structured changes, calls the Rust engine, and explains the returned result. The city view animates from that same result while the answer appears. [Team direction](https://discord.com/channels/1573514576400752640/1573561159955648553/1573664688812851221)

The engine applies changes together against shared city state and returns structured outcomes. The question agent and manual controls use the same scenario interface. The renderer consumes the engine result and owns camera, animation, and overlays. Keep one dense, lively isometric city view across questions, with TinyRJ as the visual reference.

Start with an end-to-end slice that applies housing and jobs changes together and renders the resulting zone and flow changes in that view. In parallel, prioritize comparable before-and-after observations of a documented change so we can evaluate the response. The overnight demo documents the starting point. Keep the pinned rehearsal checkout available while this work proceeds on branches.

## Local agent connection

Expose the scenario interface through a local MCP server. A user asks their own agent a question and the agent calls the MCP simulation tool with structured changes. The tool calls the Rust core and returns its result for the agent to explain. The same successful run publishes that result to the local city view, which animates the change without a second request or a manual refresh. Manual controls use the same core and result contract.

Verify the full loop with a local MCP client and the city view already open. Apply housing and jobs changes in one tool call, then confirm that the tool response and the visible city update describe the same run.
