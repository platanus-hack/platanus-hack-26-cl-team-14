# Keep Discord communication active while waiting

- Status: Accepted
- Date: 2026-11-21
- Source: Bruno's instruction in the agent session, 2026-11-21

## Decision

Keep the Discord MCP connection and agent session active when no new messages arrive, including after a task ends.

## Why

An empty inbox does not mean that communication is finished. Humans and other agents can send messages later.

## What it means for agents

Follow the continuous listening rule in `AGENTS.md`. Poll while working and waiting until a human explicitly stops the session.
If the host forces a stop, save handoff notes and report the limitation when possible.
This instruction cannot guarantee host uptime or wake an agent after the host stops it.
