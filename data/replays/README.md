# Cached replays

The presenter can use the six checked-in replay files from a clean checkout.
Start the viewer with `SANTIAGO_PLAYBACK_RATE=240 ./run.sh` for the live MUT
comparison. Press Enter on the welcome screen. In the chat, ask:

> ¿Qué pasaría si intercambiamos de lugar el MUT y el Estadio Monumental de Colo-Colo?

The chat opens the saved MUT experiment in the same turn. To see the baseline,
ask for it explicitly. Both MUT replays start at 14:00. The camera and subtitles
show each point of the saved answer in order.
[Direct playback decision](../../context/decisions/20261122-0156-play-cached-demo-directly.md)

| Scenario | Baseline | Experiment | Saved result |
| --- | --- | --- | --- |
| MUT and Monumental | `mut-monumental-routes-a` | `mut-monumental-routes-b` | [Itinerary pair](https://discord.com/channels/1573514576400752640/1573809606277337280/1573870047825236010) |
| Mapocho ferry | `mapocho-muelle-base` | `mapocho-muelle-ferry` | [Mapocho pair](https://discord.com/channels/1573514576400752640/1573809606277337280/1573846221167263784) |
| Metro closure | `metro-window-baseline` | `metro-window-closure` | [Metro pair](https://discord.com/channels/1573514576400752640/1573809606277337280/1573855157962407936) |

Run `shasum -a 256 -c data/replays/SHA256SUMS` from the repository root to
verify the files. To open the two experiments directly with Bun installed, run
`tools/side-by-side.sh mapocho-muelle-ferry metro-window-closure`.

## Replay tools

- `create_replay` runs a confirmed new experiment once and writes `<id>.json`.
- `list_replays` returns the `meta` of each file.
- `run_replay` posts `result` to `/api/replay`. The engine relays it to the viewer.

File: `{ meta: { id, title, question, scene, changes, stations, duration_half_hours, analyze, places_used, created_at }, result: <full /api/scenario result> }`.
