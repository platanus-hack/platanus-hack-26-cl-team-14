# Experiment history and cached replay

Vicente asked for direct map navigation, a confirmation before new experiments,
and a left panel that can replay saved experiments. V3 owns the saved result
and playback. V2 owns the panel and map interaction. V1 owns computation and
the result fields. The first slice below is a proposal for those owners.
[Request](https://discord.com/channels/1573514576400752640/1573514577109581907/1573885614204653710) · [Verticals](../verticals.md)

## First slice

Show the six existing saved results as a list in the left panel. Use their
saved titles. On selection, show the start, duration and analysis from the
saved metadata. Use one **Reproducir** action per result. To compare two results,
play A, wait until it appears, and then select B. Navigation leaves the saved
list available. The chat keeps the current pitch flow.

| Step | Input and output |
| --- | --- |
| List | Call `list_replays` once on open. The returned metadata has `id`, `title`, `scene.half_hour`, `duration_half_hours`, and `analyze`. The panel keeps the ID internally. A half-hour equals 30 minutes of city time. |
| Play | Send the selected ID to `run_replay`. This tool adds the saved camera cues, resolves the chart, and posts one saved snapshot to the current engine. It does not run the model. |
| Show | The current viewer receives the snapshot on the scenario stream. It uses the saved camera, captions and chart timing. A replay receipt confirms publication. Check the visible frame before playing the next shot. |

A viewer implementation can send one JSON-RPC line to the existing Node MCP
process from a worker, using the same engine origin as the viewer. It can also
reuse a shared replay adapter if the owners choose a different transport.
Keep chart resolution in the replay path, because a chart can refer to another
saved replay. The side-by-side relay omits charts and serves static tiles.
[MCP source](../../tools/santiago-mcp/server.mjs) · [Viewer stream](../../viz/src/net.rs) · [Shot order](v3-ordered-replay-shots.md) · [Bevy buttons](https://bevy.org/examples/ui-user-interface/button/)

## Acceptance

In an isolated checkout and on a private engine port, open the panel and select
**Muelle MAP07 · con ferry**. Check the 07:00 start, Pío Nono camera and chart
after 17 seconds. The chart should show 3,470.2 modeled eastbound weekday work
trips before rounding. Check a single `POST /api/replay` and zero
`POST /api/scenario` calls. Check that the six pitch IDs still appear in chat.

The current MCP path passed those publication checks on port 43173. It listed
six entries and published Mapocho with revision 1, half-hour 14, and cues at
0, 9 and 17 seconds. Panel interaction and the visible frame remain to be checked.

## New experiments

If the user asks to save a new experiment, the chat asks for confirmation
of its start, duration and outcome. After confirmation, `create_replay` runs
the model once and writes a new replay file. The next `list_replays` call
includes its metadata. Reload the catalog when the panel opens so it includes
the latest saved results. Play the new entry through the same replay path.
[Launcher](../../tools/city-chat.sh) · [Confirmed authoring fixture](https://discord.com/channels/1573514576400752640/1573889710286180484/1573897224927060061)

`simulate_city` does not save its result. Saving an already-computed run
requires a separate write path that keeps the confirmed inputs, result and
scene revision together.
