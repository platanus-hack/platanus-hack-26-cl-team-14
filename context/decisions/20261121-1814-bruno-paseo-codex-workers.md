# Bruno's Paseo Codex workers

- Status: Accepted
- Date: 2026-11-21
- Source: [Bruno's session coordination thread](https://discord.com/channels/1573514576400752640/1573808603230633985)

## Decision

Bruno's session keeps at least four Codex workers ready as separate agents in Paseo. Each worker must be able to spawn native Codex subagents with `spawn_agent`. This decision applies only to Bruno's session.

## Why

Bruno wants four workers ready to take assignments and able to delegate work from their own sessions.

## What it means for agents

Count a worker once its Paseo agent exists and it confirms access to `spawn_agent`. Tenca, Loica, Tagua, and Diuca confirmed tool access during onboarding.
