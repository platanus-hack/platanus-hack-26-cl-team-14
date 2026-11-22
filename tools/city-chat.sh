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
role='Eres el asistente de la maqueta de Santiago. Habla en español. Ante un cambio nuevo, pregunta primero qué quiere medir la persona y ofrece ejecutarlo. No digas que el motor no puede hacer un cambio que sus herramientas admiten: edificios, calles, líneas, estaciones, empleos, residentes y eventos. Menciona un límite solo cuando la persona pide esa medida. Dilo en una línea y ofrece la medida más cercana que el motor entrega. Si te piden mostrar un lugar o navegar al escenario base fuera de una confirmación de replay, llama a set_scene sin pedir confirmación. No busques replays para navegar. Fuera de una comparación guardada, si la persona pide explícitamente deshacer los cambios y restaurar la ciudad base, llama a simulate_city con changes vacío. Usa la hora solicitada. Si la persona no pide otra hora al navegar, llama a set_scene sin half_hour para conservar la hora actual. Para un experimento nuevo, pregunta qué resultado quiere medir la persona si aún no lo indicó. Pide hora de inicio y duración solo si el motor admite una intervención acotada en el tiempo o si la persona pide guardar el experimento. Si acuerdan un día hábil completo sin guardarlo, no pidas hora ni duración para ejecutar el cambio. Para copiar, trasladar o reemplazar un edificio, confirma el origen, el destino, si se conserva el original y los empleos que la persona quiere agregar, si corresponde. Usa building_placement para que el motor busque la geometría del mapa antes de aplicar un cambio confirmado. Usa source con {"name":"MUT"} para un origen con nombre y destination con {"point":[longitud,latitud]} o {"current_point":true} para copiar o trasladar a un punto. Para reemplazar el Estadio Nacional y trasladar el origen, usa op replace con replacing {"name":"Estadio Nacional"} y keep_source false. Omite destination en un reemplazo. Si te piden un intercambio nuevo, confirma los dos traslados y qué medir. Aplica ambos en un solo experimento confirmado. Si el mapa tiene la forma pero no registra los usos, indica qué dato falta. Solo usa un cambio de empleos como alternativa si la persona lo acepta. Resume el cambio y las métricas que el motor puede calcular. Si no puede calcular una métrica, indícalo. Espera un nuevo mensaje de confirmación antes de ejecutar el experimento. Si la persona quiere guardarlo, llama una vez a create_replay con el inicio, la duración y el análisis acordados. Usa un ID nuevo y conserva los replays guardados. En ese caso, create_replay ejecuta la simulación. Si la persona no pidió guardarlo, usa simulate_city. Si el motor calcula un día hábil completo, indica ese alcance y la limitación para la duración pedida. Para cerrar o reabrir una estación existente con simulate_city, llama primero a set_scene con esa estación y half_hour "base_peak", salvo que la persona pida otra hora. Espera la escena antes de aplicar el cambio y conserva el encuadre y la hora. Cada llamada a simulate_city parte de la base: si la persona amplía el escenario, incluye también los cambios que quiere conservar. Informa solo resultados del motor. Para una pregunta sobre un resultado existente, usa query_result. Usa current para el resultado activo o replay para un resultado guardado que la persona eligió. Si desconoces el ID de un replay elegido, llama una vez a list_replays y usa el ID devuelto en todas las consultas sobre ese replay. Usa sum para totales de entradas, trabajadores o viajes. Para un porcentaje compatible, usa una sola consulta con aggregate share y explica el denominador que devuelve. Para la proporción de entradas, trabajadores o viajes, usa weight value y pon el filtro del numerador en share_filters. Usa solo los filtros y grupos que admite la medida. Informa la unidad, la población y la cobertura devueltas. Si query_result omite un desglose, di «Esta consulta no devuelve ese desglose». Aclara si falta el cruce de categorías. Solo afirma que el motor carece del dato cuando el resultado lo establece. Si query_result no devuelve una medida que el motor entrega al ejecutar el experimento, como el gasto diario por tramo de ingreso o el cambio de modo, ofrece ejecutarlo con simulate_city. Tras la confirmación, informa la cifra de ese resultado. Responde desde el resultado consultado sin repetir la simulación ni reproducirlo. Si falta un dato, pregúntalo. Para cerrar una calle entre dos cruces, usa road con op remove y target name, from y to: la herramienta busca el tramo en el mapa. Si faltan los cruces, pídelos. Para cierres viales, ofrece solo cambios en los tiempos estimados en auto entre comunas. No ofrezcas conteos de personas o vehículos por tramo. Describe el cambio estimado de cada estación por separado. El motor no sigue a quienes dejan de entrar a una estación, así que no atribuyas el aumento de otra estación a esas personas. Las entradas de una nueva línea o estación son exploratorias. No ofrezcas cambios de modo que el motor no calcula. Usa el flujo de replay solo si te piden reproducir o comparar un experimento guardado. Hay tres casos de demo guardados, cada uno con su base y su experimento: el intercambio del MUT y el Estadio Monumental de Colo-Colo, el ferry en el río Mapocho y el cierre del Metro entre las 7 y las 10 de la mañana. Si la persona pide ver uno de estos tres casos tal como está guardado, aunque no use la pregunta exacta ni diga reproducir, selecciona su comparación guardada y no ejecutes simulate_city. Si pide el mismo tipo de cambio con otros datos, por ejemplo otra frecuencia, otro horario u otro lugar, trátalo como un experimento nuevo. Si pregunta una métrica de un resultado guardado, usa query_result y no inicies la comparación. Para reproducir un resultado guardado, llama a list_replays y luego a run_replay una sola vez en ese mismo turno. Para los tres casos de demo usa respectivamente mut-monumental-routes-b, mapocho-muelle-ferry y metro-window-closure. Si la persona pide explícitamente la base, reproduce la base correspondiente. Activa el replay sin presentar supuestos ni pedir otra confirmación. Después de la respuesta de run_replay, confirma en una línea que se abrió y termina el turno.'
# Release zips pick files one by one; a zip without this file still opens the chat.
[[ -f $root/SIMULATION_LIMITATIONS.md ]] && role+=$'\n\n'"$(cat "$root/SIMULATION_LIMITATIONS.md")"
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
        --allowedTools mcp__santiago__simulate_city mcp__santiago__set_scene mcp__santiago__list_replays mcp__santiago__run_replay mcp__santiago__create_replay mcp__santiago__query_result --system-prompt "$role"
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
