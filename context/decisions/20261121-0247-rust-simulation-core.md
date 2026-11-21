# Use Rust for the simulation core

- Status: Accepted
- Date: 2026-11-21
- Source: Juan’s [Rust direction](https://discord.com/channels/1573514576400752640/1573561159955648553/1573575465258717349) and [request to add it to the guidelines](https://discord.com/channels/1573514576400752640/1573561159955648553/1573575557361180817).

## Decision

Implement the simulation core in Rust. Add this rule to AGENTS.md.

## Why

Juan chose Rust for efficiency. This is a technology direction, not a measured speedup or benchmark result.

## What it means for agents

Use Rust for simulation execution. Existing data research and download scripts can remain in their current languages.
The pitch still determines which mechanisms to build.
