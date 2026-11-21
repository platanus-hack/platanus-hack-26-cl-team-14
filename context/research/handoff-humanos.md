# Cómo presentar santiaguito.cl

Usa esta pregunta del primer ensayo para comprobar el arranque del chat y de la maqueta. El caso principal ahora es el intercambio MUT ↔ Monumental. [Decisión de presentación](https://discord.com/channels/1573514576400752640/1573688250797658172/1573733563491291157).

> ¿Cómo cambiarían los viajes al trabajo en Santiago si Estación Central tuviera la mitad más de residentes y Las Condes la mitad más de empleos?

Lee del chat el cambio que estima el motor en Universidad de Chile, en subidas por día laboral. Muestra esa misma ejecución en la maqueta nativa. Luego escribe una pregunta del jurado. Ambas preguntas usan `simulate_city`.

## Descarga pública

Descarga el ZIP de macOS con Apple Silicon o Linux x86_64. Descomprímelo y ejecuta `./run.sh` desde esa carpeta. El lanzador requiere curl. En Linux, la maqueta requiere glibc 2.44 o posterior. Para usar el chat, instala Node.js 22 o posterior y sigue `README.txt` del ZIP. [santiaguito.cl v0.1.0](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/releases/tag/v0.1.0).

## Qué puedes probar hoy

El motor en `main` cambia residentes, empleos y tiempos de viaje. Puede cerrar o reabrir estaciones existentes. Comienza cada pregunta desde el escenario base. El chat recibe la estimación. El motor publica el mismo resultado para la vista. El motor estima la demanda de retorno de eventos por separado. El cálculo de eventos aplica al período completo de retorno. El chat ya puede estimar entradas en nuevos puntos de Metro. Esta función estima demanda del modelo. [PR #193](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/193), [configuración y alcance](../../README.md#ask-a-question-from-your-local-agent), [PR #137](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/137).

Instala Rust 1.97.1 o posterior con Cargo, curl y Node.js 22 o posterior. Desde la raíz del repositorio, inicia el motor y la maqueta:

```sh
./run.sh
```

Si el puerto 8787 está ocupado, usa `PORT=8790 ./run.sh`. Abre el chat desde el mismo repositorio. Configura el servidor MCP `santiago` con el valor de `SANTIAGO_API_URL` que imprime el lanzador. Usa Ctrl+C en la terminal del lanzador para cerrar el motor y la ventana.

La tecla M cambia la maqueta entre superficie y subterráneo. La vista lee los archivos de `web/data/`. Sigue pendiente fijar la revisión aceptada para presentar. [Lanzador](../../README.md#run-the-native-santiago-demo), [maqueta](../../viz/README.md).

### Permiso para la herramienta en el cliente

En Codex CLI, la prueba con el cliente usó este permiso específico y la llamada a la herramienta siguió sin una pausa de aprobación:

```sh
codex -c 'mcp_servers.santiago.tools.simulate_city.approval_mode="approve"'
```

Claude Code ejecutó la pregunta preparada en el ensayo. El permiso documentado es `--allowedTools mcp__santiago__simulate_city`. Falta confirmar ese ajuste específico. [Codex](https://developers.openai.com/codex/config-reference), [Claude Code](https://code.claude.com/docs/en/permissions#match-mcp-tools).

## Antes de presentar y entregar

- El ensayo de la pregunta preparada mostró menos subidas en Universidad de Chile y un cambio de alturas en la misma ejecución. Lee la cifra de la estación y su unidad. Las capturas muestran Tobalaba. Quedan pendientes el plano de Universidad de Chile y la grabación continua. [Traza y capturas del ensayo](https://discord.com/channels/1573514576400752640/1573688250797658172/1573712370340204634).
- Ensaya una pregunta inédita con la maqueta aceptada. Para cada pregunta, anota cuándo cambia la ventana y cuándo termina de responder el chat. Queda pendiente aceptar el detalle en la periferia y las personas que caminan en esa misma revisión.
- El nombre es «santiaguito.cl». El campo `deploy-url` abre el repositorio. La descarga pública está disponible. El equipo decide qué dirección usar en la ficha. [Ficha](../../platanus-hack-project.jsonc).
