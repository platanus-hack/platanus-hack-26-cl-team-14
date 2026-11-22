#!/usr/bin/env bash
# Record one cached replay to a 1920x1080 video: the city view only, no chat.
# The viewer renders with a hidden window, so the screen stays free.
# Usage: tools/record-replay.sh mapocho-muelle-ferry [out.mp4]
# Needs bun and ffmpeg.
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
id=$1
out=${2:-pitch/media/$id.mp4}
port=8911

cargo build --locked --release --manifest-path viz/Cargo.toml --target-dir viz/target
bun tools/replay-relay.mjs "$port" "$id" &
relay=$!
trap 'kill $relay' EXIT
until curl --silent --fail --output /dev/null "http://127.0.0.1:$port/api/scenario-latest"; do sleep 0.2; done
# A hidden X11 window keeps the monitor scale. Scale 1 gives 1920x1080 pixels.
WINIT_X11_SCALE_FACTOR=1 SANTIAGO_ENGINE="http://127.0.0.1:$port" SANTIAGO_RECORD="$out" \
    viz/target/release/santiago-miniature
