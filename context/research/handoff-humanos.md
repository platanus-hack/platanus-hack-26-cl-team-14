# Santiago responde

El equipo humano decide el guion en [context/pitch.md](../pitch.md). La [propuesta de tres minutos](pitch-proposal.md) muestra una ciudad y una pregunta:

> ¿Cómo cambiarían los viajes al trabajo en Santiago si Estación Central tuviera la mitad más de viviendas y Las Condes la mitad más de empleos?

El momento que buscamos es ver un cambio en una comuna que la pregunta no nombró.

## Qué funciona en main

En [91df8bc](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/tree/91df8bcd3183f7e26fb7e714a946c851c8522bbc), el motor en Rust aplica cambios de viviendas y empleos por comuna en una sola ejecución. Usa los flujos habituales de casa al trabajo del [Censo](../../data/mvp/work_od.csv).

La herramienta local `simulate_city` recibe los cambios, devuelve los totales y los cambios por comuna al agente y publica el resultado para la vista de ciudad. El mapa de comunas recibe también los cambios de flujos. La vista detallada que abre por defecto usa un fragmento de Tobalaba. [Herramienta](../../tools/santiago-mcp/server.mjs), [modelo](../../engine/src/city.rs), [vista](../../web/index.html).

El servidor ya puede enviar el resultado por una conexión continua. El navegador de esta revisión todavía consulta por cada actualización. La proyección de subidas al Metro ya corre como cálculo aparte desde la terminal. El equipo también incorporó datos de llegadas al Metro por propósito de viaje. [Resultados en vivo](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/108), [proyección](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/114), [datos](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/115).

El repositorio ya contiene una población sintética que conserva los totales por comuna del Censo. También contiene reglas para elegir destino y modo. La consulta del demo aún usa el motor de comunas. Falta conectar esas piezas, incorporar decisiones de horario y comprobar el efecto de cambios reales. [Población](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/109), [reglas de elección](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/112).

## Cómo verlo

En un clon del repositorio, usa la revisión indicada y ejecuta `./run.sh`. Requiere Rust y Python. Abre `http://127.0.0.1:5173/web/?v=s`. Abre Codex o Claude desde ese clon con Node instalado y usa la pregunta de arriba. La [guía del repositorio](../../README.md) explica cómo cargar la herramienta local.

### Antes de presentar

En Codex CLI, quien presenta puede autorizar solo esta herramienta con `-c 'mcp_servers.santiago.tools.simulate_city.approval_mode="approve"'`. En Claude Code, el permiso específico es `--allowedTools mcp__santiago__simulate_city`. [Codex](https://developers.openai.com/codex/config-reference), [Claude Code](https://code.claude.com/docs/en/permissions#match-mcp-tools).

## Qué queda para el ensayo

El ensayo fijó y verificó la revisión [fdd9cff](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/tree/fdd9cfffb1dda57d24f73dfe9f41a4273617a5e8). Tiene Tobalaba, Santiago, Puntaje y Evento. Los casos del MUT, la redistribución por comuna y el evento se calculan por separado en el mismo servicio. En Evento, «patrón» significa el exceso positivo de subidas de Parque O’Higgins por sobre su línea. [Verificación](https://discord.com/channels/1573514576400752640/1573579088684253235/1573677203466494159), [guion de esa revisión](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/fdd9cfffb1dda57d24f73dfe9f41a4273617a5e8/context/research/pitch-proposal.md).

En la revisión anterior `78a85a0`, una llamada directa a la herramienta y el mapa en Chrome mostraron el mismo cambio en la comuna de Santiago. El [guion](pitch-proposal.md#qué-sostiene-la-historia) registra el resultado. El clic para seleccionar otra comuna falló durante la revisión. El guion muestra el mapa completo. Codex CLI contestó la pregunta libre en una prueba aislada con el permiso específico de la herramienta. Falta ensayarla con el cliente de la presentación, el mapa abierto y el cronómetro.
