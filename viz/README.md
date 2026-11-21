# Santiago miniature

Run `./run.sh` from the repository root to open the city and a terminal beside it. The terminal starts the installed Claude Code or Codex CLI with the Santiago MCP connected to this engine. Sign in to the CLI first. When both are installed, the launcher chooses Claude. Use `SANTIAGO_CHAT_CLI=codex ./run.sh` or `SANTIAGO_CHAT_CLI=claude ./run.sh` to choose one.

The terminal uses the right 35% of the screen and displays text and icons in 17-point JetBrainsMono Nerd Font Mono. On Linux, xterm reads the bundled font through a launch-specific Fontconfig file. On macOS, Terminal uses the active font. If it is missing, the launcher prints the destination and copies the font to `~/Library/Fonts/JetBrainsMonoNerdFontMono-Regular.ttf`. If the font is disabled or unavailable, the demo opens with the Terminal profile font at 17 pt. You can activate or remove the bundled copy in Font Book. [Bundled font and SIL Open Font License](../tools/terminal-font/OFL.txt)

To open only the viewer, run this command from the repository root with Rust 1.97.1 or later. The window reads the latest scenario and event stream on port 8787 by default.

```sh
CARGO_TARGET_DIR=/tmp/viz-target cargo run --manifest-path viz/Cargo.toml
```

Drag to pan. Scroll to zoom. Press M to switch between the surface and the subway at the same camera position. Set `SANTIAGO_ENGINE` to use another engine. The default ink view uses four samples per art pixel. An ordered 4×4 dither selects the final paper and ink tones. The replay clock switches to night at 20:00 and day at 06:00. Night light in windows is decorative. Set `SANTIAGO_PIXEL=0` for the full-resolution camera with depth of field. [Pixel rendering reference](https://github.com/cobanov/pixelart-gaussiansplat)

The two building packs contain 279,385 mapped footprints and 459,845 shapes generated inside census blocks. Camera distance selects the oriented boxes or the full polygons. Roof forms, window patterns and tree crowns are visual approximations.

The window reads each full engine result. Roof height encodes changes in jobs or residents. The engine supplies before and after values for relative changes, or a net change for direction. The view displays relative changes between 0.35 and 3 times the original height. For a signed change without a baseline, it displays 0.75 or 1.25 times the original height.

Building and tree positions use OpenStreetMap data, © OpenStreetMap contributors. Terrain is produced using Copernicus WorldDEM-90 © DLR e.V. 2010–2014 and © Airbus Defence and Space GmbH 2014–2018 provided under COPERNICUS by the European Union and ESA. All rights reserved. See [the terrain source and license](../web/data/terrain.json).

The window uses satellite height estimates for 65,857 mapped roofs from Google Research, Open Buildings 2.5D Temporal (2023). The derived height data use ODbL 1.0. [Height sources and accuracy](../context/research/santiago-roof-heights.md)

The road surface follows mapped OSM ways and the terrain grid. At Encomenderos, both branches use a width estimated from two lanes. The outline of the physical island is unverified. At Vespucio Norte, OSM supplies the crossing order. The renderer uses a small vertical offset for that order. The bridge deck elevation is unknown. Some published bus paths extend outside the mapped pavement. [Road widths and map sources](../data/mvp/road_surface.md)
