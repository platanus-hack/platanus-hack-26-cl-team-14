# The MVP can run locally with a native interface

- Status: Accepted
- Date: 2026-11-21
- Source: [Juan's direction](https://discord.com/channels/1573514576400752640/1573561159955648553/1573580844839215145).

## Decision

The MVP does not have to be a web page. It can run locally, entirely in Rust, if that serves the demo better.
A public website or deployment is not required.

## Why

Juan said: "No tiene que ser una página si es que es mejor que esté 100% en rust  y corra local".
This permits a native interface; it does not require a rewrite of the current scene.

## What it means for agents

Keep the simulation core in Rust. Keep the agreed local Rust API and browser scene as the current implementation choice.
Do not treat that browser choice as a human requirement. Do not start a native UI rewrite from this permission alone.
The mutable conditions, moving 3D scene, and shared urban engine remain in scope.
