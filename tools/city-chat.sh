#!/usr/bin/env bash
# Open a real interactive CLI using its existing login. Prefer Claude when both are installed.
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
server="$root/tools/santiago-mcp/server.mjs"
export SANTIAGO_API_URL="${SANTIAGO_ENGINE:-http://127.0.0.1:8787}"
role='Eres el asistente de la maqueta de Santiago. Habla en español. Usa simulate_city para consultar o cambiar escenarios. Para cerrar o reabrir una estación existente, llama primero a set_scene con esa estación y half_hour "base_peak", salvo que la persona pida otra hora. Espera la escena antes de aplicar el cambio y conserva el encuadre y la hora. Cada llamada a simulate_city parte de la base: si la persona amplía el escenario, incluye también los cambios que quiere conservar. Informa sólo resultados del motor. Si falta un dato, pregúntalo. En la demo, si la pregunta coincide con un replay guardado (list_replays), no calcules: muestra en pocas líneas sus supuestos (qué cambia, hora de inicio, duración, qué se analiza), luego llama a run_replay con el replay base, espera a que la persona confirme que lo ve y recién entonces llama a run_replay con el experimento. Nunca llames create_replay.'
provider=${SANTIAGO_CHAT_CLI:-auto}
if [[ $provider == auto ]]; then
    if command -v claude >/dev/null; then provider=claude; else provider=codex; fi
fi
case "$provider" in claude|codex) ;; *) echo 'SANTIAGO_CHAT_CLI acepta auto, claude o codex.' >&2; exit 2 ;; esac
command -v "$provider" >/dev/null || { echo "No se encontró $provider en PATH." >&2; exit 1; }

# All clones use the same private workspace. The presenter grants any trust in the CLI.
umask 077
case $(uname -s) in
    Darwin) state="$HOME/Library/Application Support/Santiaguito" ;;
    *) state="${XDG_DATA_HOME:-$HOME/.local/share}/santiaguito" ;;
esac
mkdir -p "$state/chat"
cd "$state/chat"
ready="$state/chat-ready-$provider"
case ${1:-} in
    --ensure-prepared) [[ ! -f $ready ]] || exit 0 ;;
    --prepare) ;;
    '') ;;
    *) echo 'Uso: city-chat.sh [--prepare | --ensure-prepared]' >&2; exit 2 ;;
esac

if [[ -n ${1:-} ]]; then
    [[ -t 0 && -t 1 ]] || { echo 'Abre una terminal y ejecuta ./run.sh --prepare-chat para preparar el chat.' >&2; exit 1; }
    rm -f "$ready"
    printf 'Prepara %s en esta carpeta de la maqueta: %s\n' "$provider" "$PWD"
    printf 'Completa el inicio de sesión y revisa la confianza de la carpeta. Cuando veas dónde escribir tu pregunta, sal con /exit.\n'
fi

if [[ $provider == claude ]]; then
    config=$(node -p 'JSON.stringify({mcpServers:{santiago:{command:"node",args:[process.argv[1]],env:{SANTIAGO_API_URL:process.env.SANTIAGO_API_URL}}}})' "$server")
    claude --strict-mcp-config --mcp-config "$config" --setting-sources '' \
        --settings '{"disableAllHooks":true}' --tools '' --permission-mode dontAsk \
        --allowedTools mcp__santiago__simulate_city mcp__santiago__set_scene mcp__santiago__list_replays mcp__santiago__run_replay --system-prompt "$role"
else
    args=$(node -p 'JSON.stringify([process.argv[1]])' "$server")
    api=$(node -p 'JSON.stringify(process.env.SANTIAGO_API_URL)')
    instructions=$(node -p 'JSON.stringify(process.argv[1])' "$role")
    codex --sandbox read-only --ask-for-approval on-request \
        -c "developer_instructions=$instructions" \
        -c 'mcp_servers.santiago.command="node"' -c "mcp_servers.santiago.args=$args" \
        -c "mcp_servers.santiago.env.SANTIAGO_API_URL=$api" \
        -c 'mcp_servers.santiago.enabled=true' -c 'mcp_servers.santiago.required=true'
fi

if [[ -z ${1:-} ]]; then exit 0; fi
read -r -p '¿Viste el chat listo para escribir una pregunta? [s/N] ' answer
case "$answer" in
    s|S) ;;
    *) echo 'Preparación pendiente. Vuelve a ejecutar ./run.sh --prepare-chat.' >&2; exit 1 ;;
esac
touch "$ready"
printf 'Chat de %s preparado.\n' "$provider"
