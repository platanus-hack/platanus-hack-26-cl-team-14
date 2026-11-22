# V6 activity replay

Status: proposed for the first CicloRecreoVía replay. The [V6 brief](https://discord.com/channels/1573514576400752640/1573903663821226187/1573917586834596072) calls for a saved comparison with the same city before and during the activity.

## What to save

- Keep the [replay file](../../data/replays/README.md) with `meta` and `result`. Save the confirmed activity input in `meta.activity` with the initial city edits in `meta.changes`. Save the engine's ordered activity steps in `result.activity`.
- Keep the engine's local minute, route segment IDs, and counts per step. V1 defines the exact fields. Store the ordered route geometry once alongside the steps. Match each segment ID to its polyline in WGS84 longitude and latitude. Use geometry with a license that permits redistribution, such as the repo's attributed OpenStreetMap road pack, or get written permission from the rights holder. Also save the participant mode and any motor access rule. Show queues, riders in motion, and visits with their units. Segment counts describe groups. They cannot identify a rider across steps or establish a detour for ordinary cars.
- Use the same city edits and initial camera for the control. Save its result without the activity. Mark timed arrivals, capacity, and shop visits as assumptions until measured. The organizer's estimate of 30,000 people per weekend covers the Santiago network. It does not set entry counts for Pocuro. Report counts and time near shops. Sales require separate evidence.

## How to play it

- After confirmation, authoring runs the engine once per saved case. Listing returns replay metadata. Playback sends the full saved result once to `POST /api/replay`. The viewer receives the result through the existing scenario stream. It reads the saved steps locally. The stream can keep only the newest revision at a frame boundary, so it cannot carry one revision per step reliably.
- Give the activity one simulated clock. Start from the engine's exact local minute, including times such as 08:15. Use the saved minutes to select the current step. A camera-only move keeps the current step. Opening the replay again starts from its first step. Show the saved end boundary before stopping the playback. The existing `duration_half_hours` is a coarse cue for the old replays. It does not set the activity's step length or end. Camera and chart cues use elapsed presentation seconds.
- Keep the step list in the saved result and viewer. Give the chat a short summary. Reopening or repeating the replay reuses the same trace.

## First acceptance check

1. Save the control and activity from the same starting city. Confirm both are listed. Play each and observe one replay request and zero simulation requests per play.
2. On the activity, match the opening, movement, a visit or queue, and the closing boundary to saved step minutes. Repeat playback and compare the displayed steps. Check that the last step accounts for everyone who entered, exited, or remains.
3. Change one dwell or capacity input. The engine receipt must show which later segment or interval changed. Use a separately saved result for that run. A strip of captions alone cannot prove the underlying steps changed.

[SUMO Floating Car Data](https://sumo.dlr.de/docs/Simulation/Output/FCDOutput.html) stores time-stamped positions by simulation step. [deck.gl TripsLayer](https://deck.gl/docs/api-reference/geo-layers/trips-layer#gettimestamps) advances a separate time cursor over saved times. Both support storing the simulation result before playback. V6 starts with the engine's segment counts, so the first version needs no individual trajectory format.
