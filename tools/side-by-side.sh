#!/usr/bin/env bash
# Show N cached replays side by side, one viewer window each, with no Rust engine.
# Usage: tools/side-by-side.sh mut-monumental-b mapocho-ferry metro-window-closure
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
[ $# -gt 0 ] || { echo "Give one or more replay ids from data/replays." >&2; exit 2; }

target=${CARGO_TARGET_DIR:-viz/target}
cargo build --locked --release --manifest-path viz/Cargo.toml --target-dir "$target"

# Screen size in physical pixels, the unit of SANTIAGO_WINDOW_RECT.
if [ "$(uname -s)" = Darwin ]; then
    read -r width height < <(/usr/bin/osascript -l JavaScript -e \
        'const s = $.NSScreen.mainScreen; [s.frame.size.width * s.backingScaleFactor, s.frame.size.height * s.backingScaleFactor].map(Math.round).join(" ")')
else
    read -r width height <<<"${SIDE_BY_SIDE_SCREEN:-1920 1080}"
fi
n=$#
cols=$(( n <= 3 ? n : $(awk "BEGIN{print int(sqrt($n - 1)) + 1}") ))
rows=$(( (n + cols - 1) / cols ))
w=$(( width / cols ))
h=$(( height / rows ))

pids=()
trap 'kill "${pids[@]}" 2>/dev/null || true' EXIT
i=0
for id in "$@"; do
    port=$(( 8901 + i ))
    node tools/replay-relay.mjs "$port" "$id" &
    pids+=($!)
    until curl --silent --fail --output /dev/null "http://127.0.0.1:$port/api/scenario-latest"; do sleep 0.2; done
    SANTIAGO_ENGINE="http://127.0.0.1:$port" \
        SANTIAGO_WINDOW_RECT="$(( i % cols * w )) $(( i / cols * h )) $w $h" \
        "$target/release/santiago-miniature" &
    pids+=($!)
    i=$(( i + 1 ))
done
wait
