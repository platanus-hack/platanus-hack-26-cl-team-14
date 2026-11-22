# Play cached demo cases directly

- Status: Accepted
- Date: 2026-11-22
- Source: [Mac’s direction in the pitch thread](https://discord.com/channels/1573514576400752640/1573924666668752927/1573925156206682303)

## Decision

When the presenter asks for a prepared demo case, play the saved simulation directly. Minimize the terminal panel so the audience sees the simulation.

## Why

Use the live question to open the prepared scene. Show the scene without a separate assumptions exchange.

## What it means for agents

- Use the replay named in the pitch cues. Playback reads the saved result and does not calculate a new scenario.
- Skip the chat assumptions and the baseline and experiment confirmation prompts for the prepared demo cases. This replaces the cached playback instructions in the [live opening decision](20261121-2045-live-pitch-opening.md).
- For a new experiment, continue to ask for the missing inputs and the measure. Confirm the experiment before running it.
- Check the presenter question, the panel minimizing, and the visible saved result in the same rehearsal.
