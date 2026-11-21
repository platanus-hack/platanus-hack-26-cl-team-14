#!/bin/sh
# Run the demo: Rust engine on 8787 and the page server on 5173.
# The first build takes about 75 s with no output; later runs start in seconds.
set -e
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
trap 'kill 0' INT TERM
cargo build --release --manifest-path engine/Cargo.toml
engine/target/release/santiago-scenario-core data/mvp 8787 &
python3 -m http.server 5173 --bind 127.0.0.1 >/dev/null 2>&1 &
echo "Demo: http://127.0.0.1:5173/web/"
wait
