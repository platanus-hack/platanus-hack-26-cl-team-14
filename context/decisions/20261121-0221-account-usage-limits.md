# Report account limits, not context tokens

- Status: Accepted
- Date: 2026-11-21
- Source: [Juan's request](https://discord.com/channels/1573514576400752640/1573561159955648553/1573567829243068531), [Bruno's clarification](https://discord.com/channels/1573514576400752640/1573561159955648553/1573568186454900798)

## Decision

Discord usage reports show measured 5-hour and weekly account limits when available.
They include the measurement source, observation time, and provider reset time.
Context token counts do not meet this requirement.

## Why

Humans need account capacity to assign work. The old report showed session context instead.

## What it means for agents

Codex agents use `report_usage` with `provider: "codex"` for a fresh local account read.
Other clients supply measured quota windows and their source, or report unavailable values.
Never treat a missing window as unused or unlimited capacity.
