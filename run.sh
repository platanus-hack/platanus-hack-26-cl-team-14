#!/usr/bin/env bash
set -eu
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

port=${PORT:-8787}
api="http://127.0.0.1:$port"
if curl --silent --max-time 1 --output /dev/null "$api/api/scenario-latest"; then
    echo "Port $port is already in use. Choose a free port with PORT." >&2
    exit 1
fi

target=${CARGO_TARGET_DIR:-engine/target}
cargo build --locked --release --manifest-path engine/Cargo.toml \
    --bin santiago-scenario-core --target-dir "$target"

# Separate process groups let cleanup stop Cargo and its window together.
set -m
engine_pid=
window_pid=
cleanup() {
    trap - EXIT INT TERM
    for pid in $window_pid $engine_pid; do
        kill -TERM -- "-$pid" 2>/dev/null || true
    done
    wait 2>/dev/null || true
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

"$target/release/santiago-scenario-core" data/mvp "$port" &
engine_pid=$!
until curl --fail --silent --max-time 1 --output /dev/null "$api/api/scenario-latest"; do
    kill -0 "$engine_pid" 2>/dev/null || { wait "$engine_pid"; exit 1; }
    sleep 0.2
done
echo "SANTIAGO_API_URL=$api"
SANTIAGO_ENGINE="$api" cargo run --locked --release --manifest-path viz/Cargo.toml &
window_pid=$!
wait "$window_pid"
