# Experiment history in the city viewer

The presenter needs a left panel that opens saved results in the current map.
V2 owns the panel. V3 owns the replay and new-save contract.
[History contract](v3-experiment-history.md) · [Pitch](../pitch.md)

- Use native Bevy buttons and wheel scroll. The panel consumes the pointer so
  a scroll over its list does not change the map zoom.
  [Buttons](https://bevy.org/examples/ui-user-interface/button/)
  [Scroll](https://bevy.org/examples/ui-user-interface/scroll/)
- Run the existing MCP tools over newline-delimited JSON-RPC on a worker.
  Use the viewer's engine URL. This keeps camera cues and chart resolution in
  the same replay path the chat already uses.
  [MCP transport](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports)

Reload the catalogue on open. Play the saved MAP07 result and check its
camera and chart. Save one new experiment in a disposable checkout and
confirm that the next open shows it.
