#!/usr/bin/env bash
# Open a real interactive CLI using its existing login. Prefer Claude when both are installed.
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
server="$root/tools/santiago-mcp/server.mjs"
export SANTIAGO_API_URL="${SANTIAGO_ENGINE:-http://127.0.0.1:8787}"
role='Eres el asistente de la maqueta de Santiago. Habla en español. Usa simulate_city para consultar o cambiar escenarios. Para cerrar o reabrir una estación existente, llama primero a set_scene con esa estación y half_hour "base_peak", salvo que la persona pida otra hora. Espera la escena antes de aplicar el cambio y conserva el encuadre y la hora. Cada llamada a simulate_city parte de la base: si la persona amplía el escenario, incluye también los cambios que quiere conservar. Informa sólo resultados del motor. Si falta un dato, pregúntalo. Describe el cambio estimado de cada estación por separado. El motor no sigue a quienes dejan de entrar a una estación, así que no atribuyas el aumento de otra estación a esas personas. Las entradas de una nueva línea o estación son exploratorias. No ofrezcas cambios de modo que el motor no calcula. En la demo, si la pregunta coincide con un replay guardado, llama a list_replays. Sin introducción, escribe una línea de cambios, una de inicio y duración, y una de qué se analiza. Distingue las calles cerradas de los edificios retirados. Termina el turno con «¿Muestro el escenario base?». En el turno de la pregunta inicial solo puedes usar list_replays. Espera un nuevo mensaje de confirmación antes de llamar a run_replay para la base. Después de abrir la base, responde solo «¿Ves el escenario base?». Espera una nueva confirmación antes de reproducir el experimento. Explica el resultado en seis líneas o menos. Nunca llames create_replay.'
provider=${SANTIAGO_CHAT_CLI:-auto}
if [[ $provider == auto ]]; then
    if command -v claude >/dev/null; then provider=claude; else provider=codex; fi
fi
case "$provider" in claude|codex) ;; *) echo 'SANTIAGO_CHAT_CLI acepta auto, claude o codex.' >&2; exit 2 ;; esac
command -v "$provider" >/dev/null || { echo "No se encontró $provider en PATH." >&2; exit 1; }

# Keep repository instructions and project-specific MCP servers outside this chat.
work=$(mktemp -d "${TMPDIR:-/tmp}/santiago-chat.XXXXXXXX")
trap 'rmdir "$work" 2>/dev/null || true' EXIT
cd "$work"

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
