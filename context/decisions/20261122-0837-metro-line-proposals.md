# Metro line proposals and approval

- Status: Accepted
- Date: 2026-11-22
- Source: [Bruno’s direction at 08:37 Chile](https://discord.com/channels/1573514576400752640/1574026018635780179)

## Decision

Add a Metro vertical for creating a line between two city points or extending an existing line. The chat proposes the number and locations of intermediate stations. The user can change the count and move proposed stations before confirming.

After confirmation, the viewer shows the line, stations, and proposed station entrances. The experiment estimates users and the modes they would switch from by activity. State which activities the model covers and which estimates are unavailable.

## Why

This is new product work authorized by Bruno. It is in scope even though the accepted pitch does not include it. The existing pitch and its rehearsed sequence stay unchanged.

## What it means for agents

- Start with a read-only audit. Elba coordinates across the chat, engine, and viewer owners. Agree on the contract and assign one writer to each part before product edits.
- Treat line and entrance locations as proposals. Keep the simulation and viewport as they are while the user revises the stations. Apply the confirmed proposal through the normal chat.
- Send Bruno inspected screenshots and videos of the working interaction. Merge only after Bruno reviews both and explicitly approves the merge.
- Use an assigned capture slot outside Bruno’s live viewer.
