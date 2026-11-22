# Embedded pitch terminal

The presenter needs the real interactive Codex terminal inside the native city
window. The panel must hide during a replay and return with the same session.
The first slice is macOS. V2 owns the view and focus. V3 owns the replay flow.
[Pitch](../pitch.md) · [Replay terminal lifecycle](v3-replay-terminal-lifecycle.md)

- Wry embeds a child webview in a winit window and sets its bounds in logical points. Wry 0.55 fixed the reported child-view crash on macOS 26.3.1. [Example](https://github.com/tauri-apps/wry/blob/wry-v0.57.3/examples/winit.rs) · [Issue](https://github.com/tauri-apps/wry/issues/1705#issuecomment-4148905169)
- VS Code separates terminal visibility and layout from process ownership. We use one xterm view and one PTY for the app lifetime. [VS Code source](https://github.com/microsoft/vscode/blob/main/src/vs/workbench/contrib/terminal/browser/terminalInstance.ts) · [WezTerm PTY API](https://docs.rs/portable-pty/0.9.0/portable_pty/)

Keep the child view mounted at button size while the panel is closed. Reuse
`tools/city-chat.sh` for the interactive CLI and its engine connection. Keep
xterm.js 6.0.0 and its fit add-on 0.11.0 local. Check focus, transparency, and conversation
continuity in the native window.

The closed view uses one button. The left edge of the open view uses pointer
capture while dragging. The right edge stays fixed. The existing terminal fits
to the new bounds and updates the same PTY.
[Pointer capture](https://developer.mozilla.org/en-US/docs/Web/API/Element/setPointerCapture)
[Webview bounds](https://docs.rs/wry/0.57.3/wry/struct.WebView.html#method.set_bounds)
