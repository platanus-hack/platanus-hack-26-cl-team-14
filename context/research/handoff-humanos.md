# Cómo presentar ¿Y si Santiago…?

Abre el [guion de tres minutos](pitch-proposal.md). Escribe esta pregunta desde el chat configurado para la demo:

> ¿Cómo cambiarían los viajes al trabajo en Santiago si Estación Central tuviera la mitad más de residentes y Las Condes la mitad más de empleos?

Lee del chat el cambio que estima el motor en Universidad de Chile, en subidas por día laboral. Muestra esa misma ejecución en la maqueta nativa. Luego escribe una pregunta del jurado. Ambas preguntas usan `simulate_city`.

## Qué puedes probar hoy

El motor en `main` cambia residentes, empleos y tiempos de viaje. Puede cerrar o reabrir estaciones existentes. Comienza cada pregunta desde el escenario base. El chat recibe la estimación. El motor publica el mismo resultado para la vista. El motor estima la demanda de retorno de eventos por separado. El cálculo de eventos aplica al período completo de retorno. Crear una estación en otro punto queda fuera del alcance actual. [Configuración y alcance](../../README.md#run-a-scenario-from-your-local-agent), [PR #137](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/137).

Necesitas Rust para el motor y Node.js 22 o posterior para el servidor MCP. Desde la raíz del repositorio, inicia el motor en un puerto libre:

```sh
cargo run --release --manifest-path engine/Cargo.toml --bin santiago-scenario-core -- data/mvp 8790
```

Abre el cliente desde ese repositorio. Configura el servidor MCP `santiago` con `SANTIAGO_API_URL=http://127.0.0.1:8790`. En la ventana nativa, usa `SANTIAGO_ENGINE=http://127.0.0.1:8790`. Los tres componentes deben usar la misma dirección del motor.

La maqueta nativa en Bevy está en `feat/view-city`. Para compilarla, usa `rustc` 1.97.1 o posterior. Ejecuta el comando de la vista desde la raíz del repositorio. La vista necesita los archivos de datos de esa revisión. Sigue pendiente fijar la revisión aceptada para presentar. [Comando de la vista candidata](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/92503cdac25aa451b76ea5089eb12282f653cbe7/viz/README.md).

### Permiso para la herramienta en el cliente

En Codex CLI, la prueba con el cliente usó este permiso específico y la llamada a la herramienta siguió sin una pausa de aprobación:

```sh
codex -c 'mcp_servers.santiago.tools.simulate_city.approval_mode="approve"'
```

Claude Code ejecutó la pregunta preparada en el ensayo. El permiso documentado es `--allowedTools mcp__santiago__simulate_city`. Falta confirmar ese ajuste específico. [Codex](https://developers.openai.com/codex/config-reference), [Claude Code](https://code.claude.com/docs/en/permissions#match-mcp-tools).

## Antes de presentar y entregar

- El ensayo de la pregunta preparada mostró menos subidas en Universidad de Chile y un cambio de alturas en la misma ejecución. Lee la cifra de la estación y su unidad. Las capturas muestran Tobalaba. Quedan pendientes el plano de Universidad de Chile y la grabación continua. [Traza y capturas del ensayo](https://discord.com/channels/1573514576400752640/1573688250797658172/1573712370340204634).
- Ensaya una pregunta inédita con la maqueta aceptada. Para cada pregunta, anota cuándo cambia la ventana y cuándo termina de responder el chat. Queda pendiente aceptar el detalle en la periferia y las personas que caminan en esa misma revisión.
- El equipo decide el nombre final y la dirección de entrega. «¿Y si Santiago…?» es provisional. El campo `deploy-url` abre el repositorio. Falta el enlace público a la aplicación o a una descarga utilizable. [Ficha](../../platanus-hack-project.jsonc).
