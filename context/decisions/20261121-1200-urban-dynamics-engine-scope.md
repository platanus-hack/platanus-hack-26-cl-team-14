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
