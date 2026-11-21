# Santiago miniature

Run this command from the repository root. The engine must serve the latest scenario and event stream on port 8790.

```sh
CARGO_TARGET_DIR=/tmp/viz-target cargo run --manifest-path viz/Cargo.toml
```

Drag to pan. Scroll to zoom. Set `SANTIAGO_ENGINE` to use another engine. Set `SANTIAGO_PIXEL=1` for the pixel-art view.

The two building packs contain 279,385 mapped footprints and 375,281 shapes generated inside census blocks. Camera distance selects the oriented boxes or the full polygons. Roof forms, window patterns and tree crowns are visual approximations.

The window reads each full engine result. Roof height encodes changes in jobs or residents. The engine supplies before and after values for relative changes, or a net change for direction. The view displays relative changes between 0.35 and 3 times the original height. For a signed change without a baseline, it displays 0.75 or 1.25 times the original height.

Building and tree positions use OpenStreetMap data, © OpenStreetMap contributors. Terrain is produced using Copernicus WorldDEM-90 © DLR e.V. 2010–2014 and © Airbus Defence and Space GmbH 2014–2018 provided under COPERNICUS by the European Union and ESA. All rights reserved. See [the terrain source and license](../web/data/terrain.json).
