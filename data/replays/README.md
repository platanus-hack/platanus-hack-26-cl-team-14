# Cached replays

The demo plays only these files. The engine computes nothing during the pitch.

- `create_replay` (MCP, before the demo) runs the engine once and writes `<id>.json`.
- `list_replays` returns the `meta` of each file.
- `run_replay` posts `result` to `/api/replay`. The engine relays it to the viewer.

File: `{ meta: { id, title, question, scene, changes, stations, duration_half_hours, analyze, places_used, created_at }, result: <full /api/scenario result> }`.
