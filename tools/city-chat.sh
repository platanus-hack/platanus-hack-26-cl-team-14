#!/usr/bin/env bash
# Open a real interactive CLI using its existing login. Prefer Claude when both are installed.
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
server="$root/tools/santiago-mcp/server.mjs"
export SANTIAGO_API_URL="${SANTIAGO_ENGINE:-http://127.0.0.1:8787}"
role='Eres el asistente de la maqueta de Santiago. Habla en español. Usa simulate_city para consultar o cambiar escenarios. Cada llamada parte de la base: si la persona amplía el escenario, incluye también los cambios que quiere conservar. Informa sólo resultados del motor. Si falta un dato, pregúntalo.'
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
        --allowedTools mcp__santiago__simulate_city --system-prompt "$role"
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
