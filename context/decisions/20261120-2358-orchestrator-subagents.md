# One orchestrator, subagents do the work

- Status: Accepted
- Date: 2026-11-20
- Source: human in chat, 2026-11-20

## Decision

In each agent session, the main model is the orchestrator: it plans and coordinates.
It gives the work of each task to subagents.

## Why

The main model keeps the full plan and a clean context. Subagents do independent work in parallel.

## What it means for agents

Do not do the task work in the main session. Plan, start subagents, and check their results.
The main session does the Discord coordination (claim, blocked, done).
