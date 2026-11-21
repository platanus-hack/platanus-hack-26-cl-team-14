# Santiago responde

El equipo humano decide el guion en [context/pitch.md](../pitch.md). La [propuesta de tres minutos](pitch-proposal.md) muestra una ciudad y una pregunta:

> ¿Cómo cambiarían los viajes al trabajo en Santiago si Estación Central tuviera la mitad más de viviendas y Las Condes la mitad más de empleos?

El momento que buscamos es ver un cambio en una comuna que la pregunta no nombró.

## Qué funciona en main

En [78a85a0](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/tree/78a85a0e6370a9eb278f214747822a6cd1dcf51a), el motor en Rust aplica cambios de viviendas y empleos por comuna en una sola ejecución. Usa los flujos habituales de casa al trabajo del [Censo](../../data/mvp/work_od.csv).

La herramienta local `simulate_city` recibe los cambios, devuelve los totales y los cambios por comuna al agente y publica el resultado para la vista de ciudad. El mapa de comunas recibe también los cambios de flujos. La vista detallada que abre por defecto usa un fragmento de Tobalaba. [Herramienta](../../tools/santiago-mcp/server.mjs), [modelo](../../engine/src/city.rs), [vista](../../web/index.html).

Pendiente de construir en esa revisión: personas que elijan destino, modo y horario, cambios por dirección o estación, efectos en el Metro desde la consulta general y validación de efectos ante cambios reales. El alcance de la herramienta actual es la redistribución por comuna.

## Cómo verlo

En un clon del repositorio, usa la revisión indicada y ejecuta `./run.sh`. Requiere Rust y Python. Abre `http://127.0.0.1:5173/web/?v=s`. Abre Codex o Claude desde ese clon con Node instalado y usa la pregunta de arriba. La [guía del repositorio](../../README.md) explica cómo cargar la herramienta local. Usa un equipo donde los puertos estén libres.

## Qué queda para el ensayo

La versión reservada para el ensayo es [fdd9cff](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/tree/fdd9cfffb1dda57d24f73dfe9f41a4273617a5e8). Tiene Tobalaba, Santiago, Puntaje y Evento. Los casos del MUT, la redistribución por comuna y el evento se calculan por separado en el mismo servicio. En Evento, «patrón» significa el exceso positivo de subidas de Parque O’Higgins por sobre su línea. [Regla del ensayo](../decisions/20261121-0845-presentation-freeze.md), [guion de esa revisión](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/fdd9cfffb1dda57d24f73dfe9f41a4273617a5e8/context/research/pitch-proposal.md).

Una llamada directa a la herramienta devolvió un cambio en la comuna de Santiago. Para presentar el nuevo guion, falta ensayar la pregunta completa con el agente y el mapa abierto. El ensayo debe confirmar qué cambio se ve fuera de las comunas nombradas y cuánto dura la historia.
