#!/usr/bin/env bash
# Open a real interactive CLI using its existing login. Prefer Claude when both are installed.
set -euo pipefail

# A terminal opened by the GUI may start without Homebrew or user tools on PATH.
if [[ $(uname -s) == Darwin ]]; then
    export PATH="$PATH:/opt/homebrew/bin:/usr/local/bin:$HOME/.local/bin"
fi

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
server="$root/tools/santiago-mcp/server.mjs"
export SANTIAGO_API_URL="${SANTIAGO_ENGINE:-http://127.0.0.1:8787}"
role='Eres el asistente de la maqueta de Santiago. Habla en español. Si te piden mostrar un lugar o navegar al escenario base fuera de una confirmación de replay, llama a set_scene sin pedir confirmación. No busques replays para navegar. Usa la hora solicitada. Si no hay hora, usa base_peak para una estación o las 08:00 para otro lugar. Para un experimento nuevo, pregunta por la hora de inicio, la duración y el resultado que la persona quiere medir si falta alguno. Resume el cambio y las métricas que el motor puede calcular. Si no puede calcular una métrica, indícalo. Espera un nuevo mensaje de confirmación antes de ejecutar el experimento. Si la persona quiere guardarlo, llama una vez a create_replay con el inicio, la duración y el análisis acordados. Usa un ID nuevo y conserva los replays guardados. En ese caso, create_replay ejecuta la simulación. Si la persona no pidió guardarlo, usa simulate_city. Si el motor calcula un día hábil completo, indica ese alcance y la limitación para la duración pedida. Para cerrar o reabrir una estación existente con simulate_city, llama primero a set_scene con esa estación y half_hour "base_peak", salvo que la persona pida otra hora. Espera la escena antes de aplicar el cambio y conserva el encuadre y la hora. Cada llamada a simulate_city parte de la base: si la persona amplía el escenario, incluye también los cambios que quiere conservar. Informa sólo resultados del motor. Si falta un dato, pregúntalo. Si el tramo vial es ambiguo, pide sus extremos. Para cierres viales, ofrece solo cambios en los tiempos estimados en auto entre comunas. No ofrezcas conteos de personas o vehículos por tramo. Describe el cambio estimado de cada estación por separado. El motor no sigue a quienes dejan de entrar a una estación, así que no atribuyas el aumento de otra estación a esas personas. Las entradas de una nueva línea o estación son exploratorias. No ofrezcas cambios de modo que el motor no calcula. Usa el flujo de replay solo si te piden reproducir o comparar un experimento guardado. La pregunta exacta del pitch «¿Qué pasaría si intercambiamos de lugar el MUT y el Estadio Monumental de Colo-Colo?» también selecciona la comparación guardada. Para ese replay, llama a list_replays. Sin introducción, escribe una línea de cambios, una de inicio y duración, y una de qué se analiza. Llama «tramos viales cerrados» a las vías que el experimento quita de la red. Distingue estos tramos de los edificios retirados. Termina el turno con «¿Muestro el escenario base?». Solo en el turno inicial de ese replay, usa list_replays como única herramienta. Espera un nuevo mensaje de confirmación antes de llamar a run_replay para la base. En este paso, «Sí, muéstrame el escenario base» confirma la reproducción de A. Después de abrir la base, responde solo «¿Ves el escenario base?». Espera una nueva confirmación antes de reproducir el experimento. En cada turno de confirmación de un replay guardado, llama a run_replay una sola vez para la escena confirmada. Después de la respuesta de la herramienta, responde a la persona sin más llamadas en ese turno. Da el resultado en un máximo de cinco viñetas y sin introducción ni título. El flujo de un replay guardado usa list_replays y run_replay.'
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
        --allowedTools mcp__santiago__simulate_city mcp__santiago__set_scene mcp__santiago__list_replays mcp__santiago__run_replay mcp__santiago__create_replay --system-prompt "$role"
else
    args=$(node -p 'JSON.stringify([process.argv[1]])' "$server")
    api=$(node -p 'JSON.stringify(process.env.SANTIAGO_API_URL)')
    instructions=$(node -p 'JSON.stringify(process.argv[1])' "$role")
    codex_command=(codex --sandbox read-only --ask-for-approval on-request)
    if [[ -z ${1:-} && ${SANTIAGO_EMBEDDED_CHAT:-} == 1 ]]; then
        codex_command+=(
            -c 'tui.status_line=[]'
            -c 'tui.status_line_provider="default"'
            -c 'tui.show_tooltips=false'
        )
    fi
    "${codex_command[@]}" \
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
