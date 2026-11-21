# Santiago responde

Propuesta de los agentes. El equipo humano decide el guion en [context/pitch.md](../pitch.md). La [versión de ensayo fdd9cff](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/tree/fdd9cfffb1dda57d24f73dfe9f41a4273617a5e8) conserva los casos anteriores. Esta propuesta usa la dirección de [un motor compartido](../decisions/20261121-1200-urban-dynamics-engine-scope.md) y [las personas como estado de Santiago](https://discord.com/channels/1573514576400752640/1573561159955648553/1573667009340772443).

## Pregunta y guion de 3 minutos

| Tiempo | Qué decimos | Qué mostramos |
| --- | --- | --- |
| 0:00–0:35 | «Tu viaje al trabajo empieza antes de salir de casa. Empieza cuando la ciudad decide dónde se puede vivir y dónde habrá trabajo. Esas decisiones se toman en comunas distintas. Tú haces el viaje completo». | Abrimos el mapa de comunas de Santiago (`/web/?v=s`). La vista muestra los flujos habituales entre casa y trabajo. |
| 0:35–1:05 | «¿Cómo cambiarían los viajes al trabajo en Santiago si Estación Central tuviera la mitad más de viviendas y Las Condes la mitad más de empleos?» | Escribimos la pregunta en el agente local. El agente envía ambos cambios en una llamada a `simulate_city`. El aumento de la mitad en cada comuna es un supuesto de la pregunta, no un dato observado. |
| 1:05–2:00 | «La ciudad responde. Miremos de dónde salen los viajes y dónde terminan. Fíjense en las comunas que no tocamos. La decisión llega hasta allá». | La misma ejecución devuelve los cambios al agente y actualiza la vista. La sorpresa es un cambio en una comuna que nadie nombró. El presentador lee un cambio que aparece en el resultado de esa ejecución y selecciona la comuna de Santiago en el mapa. |
| 2:00–2:35 | «Partimos de los viajes habituales al trabajo del Censo. En esta versión, el motor reparte esos viajes por comuna según las viviendas y los empleos que cambiamos. Es una estimación de cómo se moverían los viajes bajo esos supuestos». | El panel muestra los totales modelados de la comuna elegida. El agente dice cuánto cambian los trabajadores con destino a esa comuna. |
| 2:35–3:00 | «Cuando decidimos dónde construir, cambiamos el viaje de alguien que vive lejos. Hagamos la pregunta antes de tomar la decisión». | Abrimos la cámara sobre Santiago y dejamos el resultado visible. |

## Qué sostiene la historia

- El Censo cuenta lugares habituales de trabajo. Los flujos del guion representan trabajadores por origen y destino. Los resultados se expresan como cambios modelados de esos flujos. [Flujos y cobertura](../../data/mvp/README.md).
- El motor de comunas aplica vivienda y empleo juntos. El supuesto de la demo se traduce a `housing_multiplier: 1.5` para Estación Central y `jobs_multiplier: 1.5` para Las Condes. El otro multiplicador de cada comuna es `1.0`. [Contrato de cambios](../../tools/santiago-mcp/server.mjs), [reglas en Rust](../../engine/src/city.rs), [códigos de comuna](../../data/mvp/communes.csv).
- El guion no fija un resultado numérico. Se dice el del mismo identificador de ejecución que muestra la ciudad. En una llamada directa a la herramienta local en la revisión de referencia, el centro de Santiago perdió trabajadores asignados como destino. Antes de presentar, correr la pregunta completa con el agente y la vista abierta en el commit que servirá la demo.
- **Not built** en la revisión de referencia. Un estado sintético por persona, consistente con los totales del Censo, que permita elegir destino, modo y horario.
- **Not built** en la revisión de referencia. Ver los flujos de toda la ciudad y sus cambios en la vista detallada que abre por defecto. Esa vista usa un fragmento de Tobalaba. Para el guion se usa el mapa de comunas. [Vista](../../web/index.html).
- **Not built** en la revisión de referencia. Cambios en una dirección, estación o tramo de red por la misma herramienta. Tampoco hay resultados de congestión, capacidad ni demanda por estación para esta pregunta.
- **Not built** en la revisión de referencia. Validación de los efectos ante cambios reales. Un ajuste a los viajes del Censo sirve para comprobar el punto de partida. Para comprobar una intervención hacen falta observaciones comparables de antes y después.
- **Not built** en la revisión de referencia. Respuestas de simulación para cualquier pregunta de urbanismo, construcción, movilidad o transporte. El agente debe indicar qué cambios admite el motor cuando una pregunta excede ese alcance.

La referencia de implementación es [78a85a0](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/tree/78a85a0e6370a9eb278f214747822a6cd1dcf51a). Los tiempos del guion son un presupuesto. Falta medir un ensayo de punta a punta.
