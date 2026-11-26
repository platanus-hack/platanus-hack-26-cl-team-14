# Replay links and the Chrome assistant panel

- Status: Accepted
- Date: 2026-11-26
- Source: Emerson, in the browser build session.

## Decision

Keep the landing page at the site root. Link to the simulator at `/web/wasm/` and to saved examples with `?replay=<id>`. The simulator opens an example through its existing `run_replay` tool. The engine reads the saved result and does not run the experiment again.

The Chrome companion reports when its side panel opens or closes. The page uses that signal to show the changes panel. The Changes button also opens the panel. A WebMCP tool call opens it when the browser exposes WebMCP.

## Why

Chrome gives each extension the state of its own side panel. WebMCP registers page tools in compatible browsers and reports a tool call when an assistant uses one. [Chrome Side Panel API](https://developer.chrome.com/docs/extensions/reference/api/sidePanel), [Chrome WebMCP tool guide](https://developer.chrome.com/docs/modern-web-guidance/guides/webmcp/agentic-javascript-tools)

## Static host

Serve `landing/public/` at the root of the same origin as `/web/wasm/`. Keep the viewer data, replay JSON, MCP modules, and fonts at their current paths. The current landing-only asset directory contains the home page and the gallery. The deployment also needs a route for the simulator and its files. The local preview serves both at `http://127.0.0.1:8961/`.
