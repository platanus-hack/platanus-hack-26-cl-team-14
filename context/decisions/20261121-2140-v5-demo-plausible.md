# 20261121-2140 — V5 demo-related evals and engine improvement

- Status: Accepted
- Date: 2026-11-21
- Source: Francisco, in a Claude Code session

## Decision

Add a fifth vertical, V5. It starts when the humans say V1 is finished.

V5 improves the simulator so the demo scenes are plausible. It creates evals for real situations that are related to the demo scenes and that have public data. Then it improves the engine modules and the entities until those evals score well. See [verticals.md](../verticals.md).

## Why

The demo scenes are hypothetical, so nobody can measure them directly. Real situations that use the same mechanisms can be measured, for example the flow of people before and after a football match. If the engine reproduces these, the demo scenes are plausible. The current scores do not do this: they cover stations the demo does not show, and most V1 layers do not change them (#435 gave identical station bins).

## What it means for agents

- Agents choose the evals. Each eval is a real, plausible situation with data, related to a demo scene. Write which scene it supports and why.
- Situations not related to the demo are out of scope.
- The goal is a better engine, not a better report. Change modules and entities with research first. Fitting the engine to these evals is the intent.
- When a score goes on the pitch slide, label it as fitted to the data.
- Until V5 starts, V1 uses the April Metro score and the Line 1 cells, with their labels.
