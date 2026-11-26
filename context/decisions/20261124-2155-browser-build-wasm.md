# Build the engine and the viewer for the browser with WebAssembly

- Status: Accepted
- Date: 2026-11-24
- Source: Emerson, in his agent session: "queremos la compilación a wasm".

## Decision

Compile the Rust engine and the Rust viewer to WebAssembly. Both run in one browser tab.
The engine keeps the paths and JSON of its HTTP API. The page calls them through a worker.

## Why

A page cannot open a server port. With the engine in the tab, the public site needs only static files.
The agent connection (MCP) is a separate step: a relay or WebMCP can call the same paths later.

## Update, 2026-11-25

Emerson, in his agent session: there will be no more native demo. The browser build is the target.
Shared code (the viewer panels, the MCP result cards) can change for the browser.

## What it means for agents

- Engine: target `wasm32-wasip1`. `web/wasm/wasi.js` gives it read-only files. The HTTP server is native only.
- Viewer: target `wasm32-unknown-unknown` with WebGPU. WebGL2 cannot run the ink pass, which reads scene depth.
- Viewer data files go through `viz/src/data.rs`. Add each new file to `web/wasm/viewer-files.json`.
- The engine does not start other programs. It unpacks gzip files with `flate2`.
- Build with `tools/wasm/build.sh`.
