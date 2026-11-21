#!/usr/bin/env bash
set -eu
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

repo=$PWD
platform=$(uname -s)
case $platform in
    Darwin) source "$repo/tools/city-terminal-mac.sh" ;;
    Linux)  source "$repo/tools/city-terminal-linux.sh" ;;
    *) echo "The city launcher supports macOS and Linux." >&2; exit 1 ;;
esac

presentation=${1:-}
if [[ $presentation == --presentation && $platform != Darwin ]]; then
    echo 'La entrada de presentación está disponible en macOS.' >&2
    exit 1
fi

port=${PORT:-8787}
api="http://127.0.0.1:$port"
if curl --silent --max-time 1 --output /dev/null "$api/api/scenario-latest"; then
    echo "Port $port is already in use. Choose a free port with PORT." >&2
    exit 1
fi

target=${CARGO_TARGET_DIR:-engine/target}
engine="$target/release/santiago-scenario-core"
viewer=(cargo run --locked --release --manifest-path viz/Cargo.toml)

# Separate process groups let cleanup stop Cargo and its window together.
set -m
engine_pid=
window_pid=
welcome_pid=
terminal_pid=
terminal_layout_pid=
runtime=$(mktemp -d "${TMPDIR:-/tmp}/santiago-launch.XXXXXX")
engine_log="$runtime/engine.log"
cleanup() {
    trap - EXIT INT TERM
    for pid in $terminal_layout_pid $terminal_pid $window_pid $welcome_pid $engine_pid; do
        kill -TERM -- "-$pid" 2>/dev/null || true
    done
    if declare -F cleanup_city_terminal_mac >/dev/null; then cleanup_city_terminal_mac || true; fi
    wait 2>/dev/null || true
    rm -f "$engine_log" "$runtime/engine-starting" "$runtime/ready"
    rmdir "$runtime"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

if [[ $presentation == --presentation ]]; then
    swift tools/presentation-welcome.swift pitch/santiago-dia-noche-web.mp4 "$runtime" &
    welcome_pid=$!
fi

if [[ $presentation == --presentation && ( -n ${SANTIAGO_ENGINE_BIN:-} || -n ${SANTIAGO_VIEWER_BIN:-} ) ]]; then
    if [[ ! -x ${SANTIAGO_ENGINE_BIN:-} || ! -x ${SANTIAGO_VIEWER_BIN:-} ]]; then
        echo 'Indica el motor y el visor ejecutables para esta presentación.' >&2
        exit 1
    fi
    engine=$SANTIAGO_ENGINE_BIN
    viewer=("$SANTIAGO_VIEWER_BIN")
else
    cargo build --locked --release --manifest-path engine/Cargo.toml \
        --bin santiago-scenario-core --target-dir "$target"
    if [[ $presentation == --presentation ]]; then
        cargo build --locked --release --manifest-path viz/Cargo.toml
    fi
fi

if [[ $presentation == --presentation ]]; then
    touch "$runtime/engine-starting"
fi
touch "$engine_log"
"$engine" data/mvp "$port" 2> >(tee "$engine_log" >&2) &
engine_pid=$!
until grep -Fxq "Scenario core listening on $api" "$engine_log"; do
    kill -0 "$engine_pid" 2>/dev/null || { wait "$engine_pid"; exit 1; }
    sleep 0.2
done
curl --fail --silent --show-error --output /dev/null \
    --header 'Content-Type: application/json' --data '{"changes":[]}' "$api/api/scenario"
echo "SANTIAGO_API_URL=$api"
export SANTIAGO_ENGINE="$api"
if [[ $presentation == --presentation ]]; then
    touch "$runtime/ready"
    wait "$welcome_pid"
    welcome_pid=
    export SANTIAGO_DISTANCE="${SANTIAGO_DISTANCE:-38000}"
fi
if declare -F open_city_terminal_mac >/dev/null; then
    open_city_terminal_mac
else
    open_city_terminal_linux
fi
export SANTIAGO_WINDOW_RECT="$city_x $city_y $city_width $city_height"
"${viewer[@]}" &
window_pid=$!
wait "$window_pid"
