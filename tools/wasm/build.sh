#!/bin/sh
# Build the browser version: the engine (wasm32-wasip1) and the viewer (wasm32-unknown-unknown, WebGPU).
# Needs: rustup target add wasm32-wasip1 wasm32-unknown-unknown
#        cargo install wasm-bindgen-cli --version 0.2.130 --locked   (the version in viz/Cargo.lock)
#        Node.js 22 or later
# Then serve the repository root and open /web/wasm/ in a browser with WebGPU:
#        python3 tools/wasm/serve.py 8961
set -e
cd "$(dirname "$0")/../.."
export PATH="$HOME/.cargo/bin:$PATH"
target=${CARGO_TARGET_DIR:-target/wasm}
cargo build --release --manifest-path engine/Cargo.toml --bin santiago-scenario-core --target wasm32-wasip1 --target-dir "$target"
cp "$target/wasm32-wasip1/release/santiago-scenario-core.wasm" web/wasm/engine.wasm
cargo build --release --manifest-path viz/Cargo.toml --target wasm32-unknown-unknown --target-dir "$target"
# The name section is half of the file; the browser does not need it.
wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
    --out-name viewer --out-dir web/wasm "$target/wasm32-unknown-unknown/release/santiago-miniature.wasm"
python3 tools/wasm/pack_startup_polygons.py
topology=web/wasm/car-topology.bin.gz
if [ ! -f "$topology" ] || [ web/data/local_car_graph.json.gz -nt "$topology" ] || [ tools/wasm/pack_topology.py -nt "$topology" ]; then
    python3 tools/wasm/pack_topology.py
fi
# A static site cannot list a folder. The MCP server lists these two.
python3 -c 'import json, os; print(json.dumps({d: sorted(os.listdir(d)) for d in ("data/replays/", "web/data/")}))' > web/wasm/folders.json
streets=web/wasm/road-surface.bin.gz
if [ ! -f "$streets" ] || [ ! -f web/wasm/road-surface-deferred.bin ] || [ web/data/road_surface.geojson.gz -nt "$streets" ] || [ tools/wasm/pack_streets.py -nt "$streets" ]; then
    python3 tools/wasm/pack_streets.py
fi
node tools/wasm/baseline.mjs
ls -l web/wasm/engine.wasm web/wasm/viewer_bg.wasm web/wasm/baseline.json
