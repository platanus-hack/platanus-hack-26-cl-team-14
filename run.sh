#!/usr/bin/env bash
set -eu
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

repo=$PWD
case $(uname -s) in
    Darwin)
        export SANTIAGO_EMBEDDED_CHAT=1
        if [[ -z ${SANTIAGO_CHAT_CLI:-} ]] && command -v codex >/dev/null; then
            export SANTIAGO_CHAT_CLI=codex
        fi
        ;;
    Linux)  source "$repo/tools/city-terminal-linux.sh" ;;
    *) echo "The city launcher supports macOS and Linux." >&2; exit 1 ;;
esac

if [[ ${1:-} == --prepare-chat ]]; then
    exec "$repo/tools/city-chat.sh" --prepare
fi

port=${PORT:-8787}
api="http://127.0.0.1:$port"
if curl --silent --max-time 1 --output /dev/null "$api/api/scenario-latest"; then
    echo "Port $port is already in use. Choose a free port with PORT." >&2
    exit 1
fi
"$repo/tools/city-chat.sh" --ensure-prepared

target=${CARGO_TARGET_DIR:-engine/target}
cargo build --locked --release --manifest-path engine/Cargo.toml \
    --bin santiago-scenario-core --target-dir "$target"

# Separate process groups let cleanup stop Cargo and its window together.
set -m
engine_pid=
window_pid=
terminal_pid=
terminal_layout_pid=
launch_dir=$(mktemp -d "${TMPDIR:-/tmp}/santiago-boot.XXXXXX")
engine_log="$launch_dir/engine.log"
export SANTIAGO_BOOT_SIGNAL="$launch_dir/enter"
cleanup() {
    trap - EXIT INT TERM
    for pid in $terminal_layout_pid $terminal_pid $window_pid $engine_pid; do
        kill -TERM -- "-$pid" 2>/dev/null || true
    done
    wait 2>/dev/null || true
    rm -rf "$launch_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

touch "$engine_log"
"$target/release/santiago-scenario-core" data/mvp "$port" 2> >(tee "$engine_log" >&2) &
engine_pid=$!
until grep -Fxq "Scenario core listening on $api" "$engine_log"; do
    kill -0 "$engine_pid" 2>/dev/null || { wait "$engine_pid"; exit 1; }
    sleep 0.2
done
curl --fail --silent --show-error --output /dev/null \
    --header 'Content-Type: application/json' --data '{"changes":[]}' "$api/api/scenario"
echo "SANTIAGO_API_URL=$api"
export SANTIAGO_ENGINE="$api"
if declare -F open_city_terminal_linux >/dev/null; then
    export SANTIAGO_CITY_WINDOW_CLASS="SantiagoCity$$"
fi
cargo run --locked --release --manifest-path viz/Cargo.toml &
window_pid=$!
if declare -F open_city_terminal_linux >/dev/null; then
    while [[ ! -e "$SANTIAGO_BOOT_SIGNAL" ]] && kill -0 "$window_pid" 2>/dev/null; do
        sleep 0.1
    done
    if [[ -e "$SANTIAGO_BOOT_SIGNAL" ]] && kill -0 "$window_pid" 2>/dev/null; then
        open_city_terminal_linux
        printf '%s %s %s %s\n' "$city_x" "$city_y" "$city_width" "$city_height" > "$launch_dir/layout.tmp"
        mv "$launch_dir/layout.tmp" "$SANTIAGO_BOOT_SIGNAL.rect"
    fi
fi
wait "$window_pid"
