# Use mouse navigation in the native city

- Status: Accepted
- Date: 2026-11-21
- Source: [Emerson's decision in agent-chat](https://discord.com/channels/1573514576400752640/1573561159955648553/1573724692135809218)

## Decision

Enable drag to pan and wheel to zoom when the city starts. The presenter can inspect street-level detail during a scenario. Pressing M switches between the surface and the subway at the same camera position.

## What it means for agents

The chat sends scenario changes to the engine. An engine result can focus the camera on the affected place. Mouse navigation only changes the view. For matched captures, set `SANTIAGO_LOCK_CAMERA=1`, `SANTIAGO_FOCUS` and `SANTIAGO_DISTANCE`.
