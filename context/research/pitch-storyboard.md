# ¿Y si el estadio de Colo-Colo estuviera en el MUT?

Propuesta de tres minutos según la dirección de Mac. El [guion de Estación Central y Las Condes](pitch-proposal.md) conserva el primer ensayo. [Dirección de Mac](https://discord.com/channels/1573514576400752640/1573688250797658172/1573716600329343077).

| Tiempo | Qué vemos | Qué dice quien presenta |
| --- | --- | --- |
| 0:00–0:20 | La Tierra. Santiago. Un auto rumbo a Tobalaba. La cámara se pone a su altura y lo sigue. | «Miren ese auto. Viene al MUT. Para quienes van adentro, este viaje es parte de un día cualquiera». |
| 0:20–0:40 | El auto se detiene. Bajan personas y entran al MUT. Corte al chat. | «Pensemos en ese mismo lugar en otra noche. ¿Y si en vez del MUT estuviera el estadio de Colo-Colo?» |
| 0:40–1:30 | Escribimos la pregunta preparada. La cámara abre el plano. Aparece el resultado de ese escenario en el chat. | «¿Qué pasa con los viajes de quienes hoy trabajan ahí? ¿Y cómo volvería la gente después de un partido?» |
| 1:30–1:50 | La maqueta muestra el cambio del escenario. En el chat se lee una cifra de viajes y su unidad. | «Cambiamos un lugar. Veamos hasta dónde llega el cambio». Lee el efecto que devolvió esa ejecución. |
| 1:50–2:45 | Abrimos el plano a tres paneles. Quedan el estadio, el escenario de Estación Central y Las Condes, y el espacio para la pregunta del jurado. | «También podemos cambiar dónde vive y trabaja la gente. Ahora elijan ustedes qué cambiar de Santiago». Escribe la pregunta y lee el resultado. |
| 2:45–3:00 | Quedan los tres escenarios a la vista. | «Antes de cambiar un lugar, hagamos la pregunta y miremos quién más tendría que cambiar su viaje». |

## Pregunta preparada

> ¿Qué pasaría con los viajes al trabajo y el regreso del público si reemplazamos el MUT por el estadio de Colo-Colo y un partido recibe [asistencia supuesta] personas? Resume el cambio en una cifra por día laboral y una del evento, con sus unidades.

Para el ensayo, fija y declara la asistencia. Comprueba en `applied` que la llamada cambió los empleos modelados en la manzana del MUT. El caso de referencia usa esa manzana. El motor estima por separado los viajes de regreso para el período completo del evento. El chat puede leer sus totales por modo y comuna. Deja la cifra de Tobalaba y la de regreso del público como espacios por llenar hasta correr esta pregunta. [Caso de referencia del MUT](../../tools/perf/scenarios.py), [contrato del motor](../../engine/README.md#general-scenario-http-routes).

Para el segundo panel, usa la pregunta de Estación Central y Las Condes, que ya pasó por el chat y la maqueta. Para el tercero, usa la pregunta del jurado. Si el jurado no propone una, el cierre de estaciones de Quilicura es otra pregunta que admite el motor. [Primer ensayo](https://discord.com/channels/1573514576400752640/1573688250797658172/1573712370340204634), [casos del motor](../../tools/perf/scenarios.py).

## Qué falta para mostrarlo

- Grabar la bajada desde la Tierra y el seguimiento del mismo auto. Falta la escena donde los pasajeros bajan y entran al MUT.
- Ensayar la pregunta del estadio en el chat. Falta mostrar el edificio del estadio y la salida del público en la maqueta. Para el público, habla de viajes estimados y sus modos. La congestión vial, el tiempo de viaje en auto y la ocupación quedan fuera del cálculo actual. [Alcance](../../engine/README.md#general-scenario-http-routes).
- Montar los tres paneles con capturas de ejecuciones reales. La prueba que tenemos del MUT cambió alturas de edificios en Tobalaba al quitar empleos. [Prueba de la ventana](https://discord.com/channels/1573514576400752640/1573688247572238336/1573704493281972224).
- Grabar el recorrido sin cortes con la maqueta aceptada. Registra cuándo cambia la ventana y cuándo termina cada respuesta. Cronometra desde la primera frase hasta el cierre. [Guía del ensayo](handoff-humanos.md).
