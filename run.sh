#!/bin/sh
# Run the demo: Rust engine on 8787 and the page server on 5173.
# One measured first build took 75 s; build time depends on the host and cache.
set -e
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release --manifest-path engine/Cargo.toml
engine_pid=
web_pid=
cleanup() {
    trap - 0 INT TERM
    for pid in $engine_pid $web_pid; do
        kill "$pid" 2>/dev/null || true
    done
    wait 2>/dev/null || true
}
trap cleanup 0
trap 'exit 130' INT
trap 'exit 143' TERM
engine/target/release/santiago-scenario-core data/mvp 8787 &
engine_pid=$!
python3 -m http.server 5173 --bind 127.0.0.1 &
web_pid=$!
sleep 1
if kill -0 "$engine_pid" 2>/dev/null && kill -0 "$web_pid" 2>/dev/null; then
    echo "Demo: http://127.0.0.1:5173/web/"
fi
while kill -0 "$engine_pid" 2>/dev/null && kill -0 "$web_pid" 2>/dev/null; do
    sleep 1
done
echo "A demo server stopped; stopping this launcher's other server." >&2
exit 1
