# Clear the stadium footprint at the MUT site

- Status: Accepted
- Date: 2026-11-21
- Source: [Juan's direction in Discord](https://discord.com/channels/1573514576400752640/1573561159955648553/1573841218553774224) and [Lautaro's scope in V1](https://discord.com/channels/1573514576400752640/1573809708047798364/1573841289223471125)

## Decision

In the stadium replay, remove each building that overlaps the relocated Monumental footprint at the MUT site. Then place the stadium there. Keep buildings outside the footprint.

## Why

The stadium needs the full area of its footprint. The current placement crosses 12 blocks, and the engine requires a moved building to fit inside one block. Clearing the site leaves the block boundaries in place. [Placement check](https://discord.com/channels/1573514576400752640/1573809606277337280/1573840662925676659)

## What it means for agents

- V1 needs to support placement across blocks and include the building removals in the scenario.
- Keep the two job edits, 39,222 attendees and the 17:00 event end in the replay inputs. Rebuild the experiment cache after the engine accepts the move.
- In the same camera view, show the relocated stadium, the cleared buildings and one retained neighbor outside the footprint. [Review direction](https://discord.com/channels/1573514576400752640/1573809708047798364/1573841341195223063)
