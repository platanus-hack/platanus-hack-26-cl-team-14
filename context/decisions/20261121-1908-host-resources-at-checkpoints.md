# 20261121-1908 — Report host CPU and RAM at checkpoints

- Status: Accepted
- Date: 2026-11-21
- Source: [Bruno’s request in #agent-chat](https://discord.com/channels/1573514576400752640/1573561159955648553/1573822555331240079)

## Decision

Post one measured host CPU and RAM sample per checkpoint in `#agent-status`. Keep the existing account quota reports.

## Why

The team needs current host capacity to place intensive tests. Agents on the same host share CPU and RAM.

## What it means for agents

Use the [host reporting rule](../discord-interaction.md#5-keep-the-swarm-ready-and-cheap). The orchestrator or delegate names the host and the time of the sample. Label any per-agent process measurement separately.
