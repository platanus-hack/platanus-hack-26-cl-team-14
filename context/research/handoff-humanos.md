# Santiago responde

El equipo humano decide el guion en [context/pitch.md](../pitch.md). La [propuesta de tres minutos](pitch-proposal.md) muestra una ciudad que responde. Primero el presentador muestra este ejemplo:

> ¿Cómo cambiarían los viajes al trabajo en Santiago si Estación Central tuviera la mitad más de residentes y Las Condes la mitad más de empleos?

El ejemplo cambia los viajes en una comuna que el presentador no nombró. Luego alguien del público hace una pregunta que el equipo no preparó. El presentador la escribe en el chat. Ambas ejecuciones usan la misma herramienta.

## Qué funciona en main

En [dea6e59](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/tree/dea6e591c47467d61f60ec551e2f1cb1ff0b20ee), el motor en Rust simula cambios de residentes, empleos y tiempos de viaje. También puede cerrar o reabrir estaciones existentes. Emplea una población sintética que conserva los totales por comuna del Censo. Los trabajadores vuelven a elegir destino y modo ante un cambio. [Motor](../../engine/src/scenario.rs), [datos](../../data/mvp/README.md).

Comprobamos que `simulate_city` aplica los dos cambios del ejemplo. El agente y la vista reciben el mismo identificador y las mismas cifras del resumen. La vista recibe además los grupos de viajes para dibujarlos. La vista nativa de Santiago está en una rama de trabajo. Falta probar juntos el chat y la vista de Santiago sin controles. [Cambio del motor](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/126).

El motor declara las preguntas que requieren una estación nueva, una línea nueva o un evento. Tampoco estima capacidad ni congestión. La versión actual carga el motor general al arrancar. Una revisión independiente comprobó que arranca y responde después de cambiar el identificador de la manzana del MUT. [Verificación del arranque](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/136).

## Cómo probar el motor

Abre la revisión indicada en un clon del repositorio. Necesitas Rust y Node. Desde la raíz del clon, inicia solo el motor en un puerto libre:

```sh
cargo run --release --manifest-path engine/Cargo.toml --bin santiago-scenario-core -- data/mvp 8790
```

Abre el cliente desde ese clon. Configura el servidor MCP `santiago` con `SANTIAGO_API_URL=http://127.0.0.1:8790` y haz la pregunta de arriba. La vista nativa debe apuntar al mismo puerto. Falta publicar la revisión de la vista aceptada y su comando de arranque.

### Antes de presentar

En Codex CLI, quien presenta puede autorizar solo esta herramienta con `-c 'mcp_servers.santiago.tools.simulate_city.approval_mode="approve"'`. En Claude Code, el permiso específico es `--allowedTools mcp__santiago__simulate_city`. [Codex](https://developers.openai.com/codex/config-reference), [Claude Code](https://code.claude.com/docs/en/permissions#match-mcp-tools).

## Qué queda para el ensayo

El ensayo fijó y verificó la revisión [fdd9cff](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/tree/fdd9cfffb1dda57d24f73dfe9f41a4273617a5e8). Tiene Tobalaba, Santiago, Puntaje y Evento. Los casos del MUT, la redistribución por comuna y el evento se calculan por separado en el mismo servicio. En Evento, «patrón» significa el exceso positivo de subidas de Parque O’Higgins por sobre su línea. [Verificación](https://discord.com/channels/1573514576400752640/1573579088684253235/1573677203466494159), [guion de esa revisión](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/fdd9cfffb1dda57d24f73dfe9f41a4273617a5e8/context/research/pitch-proposal.md).

Codex CLI contestó una pregunta libre sobre el motor anterior en una prueba aislada con el permiso específico de la herramienta. Falta ensayar una pregunta inédita con el motor actual, la vista sin controles y el cronómetro. La ficha de votación apunta al repositorio. Falta la URL pública del proyecto funcionando.
