# Replay MUT → Monumental

Borrador para el primer caso de V3. La propuesta reserva 50 segundos dentro del pitch de tres minutos. Faltan la aprobación humana y el tiempo medido en el visor. [Dirección de V3](../verticals.md) · [Propuesta de pitch #354](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/354)

> ¿Qué cambia si Colo-Colo juega donde está el MUT y movemos los empleos modelados de esa manzana a Macul?

| Tiempo | Qué se ve | Qué se dice |
| --- | --- | --- |
| 0:00–0:08 | El presentador escribe la pregunta en la terminal del mapa. Aparecen los supuestos. | «Comparamos el mismo partido en dos lugares, con 39.222 asistentes». |
| 0:08–0:20 | La terminal se oculta. La escena A muestra el estadio actual en Macul y luego el MUT junto a Tobalaba. | «Este es el partido en el Monumental. Ahora lo llevamos a Tobalaba». |
| 0:20–0:38 | Cuando A ya muestra el MUT, el presentador pasa a B. La cámara repite los encuadres de Tobalaba y Macul. | «Intercambiamos los empleos modelados de estas dos manzanas. La hinchada conserva sus comunas de origen». |
| 0:38–0:44 | Dos barras apiladas comparan los modos de viaje de la hinchada: transporte público, auto y otros. | «En el MUT, el modelo estima unos 4.102 asistentes más en transporte público». |
| 0:44–0:50 | Un gráfico aparte muestra el cambio de entradas al Metro en un día laboral en Tobalaba, Pedrero y Mirador. Al terminar, vuelve la terminal. | «En un día laboral, el modelo estima menos entradas en Tobalaba y más en Pedrero y Mirador». |

[Solicitud provisional de planos y tiempos](https://discord.com/channels/1573514576400752640/1573809606277337280/1573817049661309078)

El JSON actual de los planos solo cubre el gráfico de la hinchada. Falta añadir un paso en el visor nativo para mostrar el gráfico del Metro con los mismos resultados A y B de la caché. Este paso no requiere otro cálculo de simulación.

## Preparar las dos escenas

- Parte de la base para cada llamada. Usa `simulate_city` antes del pitch. A agrega el partido a `monumental`. B fija 53 empleos en la manzana `13114131009033`, 4.679 en `13118031003002` y agrega el partido a `mut`. Los conteos corresponden a manzanas. Comprueba los valores en la base que se use para grabar. [Decisión del intercambio](../decisions/20261121-1315-mut-monumental-swap.md) · [Ensayo A/B por MCP](https://discord.com/channels/1573514576400752640/1573734041541288053/1573752258351341739)
- Declara 39.222 asistentes, la misma hinchada de Colo-Colo y las 17:00 como hora supuesta de término. La cifra de asistencia viene del Superclásico del 31 de agosto de 2025. La hora de término no altera la oferta de transporte en el cálculo. Muestra también los supuestos de `events[0].fan_base.assumptions` antes de reproducir. El modelo usa la educación de la jefatura del hogar como aproximación al grupo socioeconómico. Estima los modos con datos de viajes al trabajo entre comunas. El servicio de fin de semana y el estacionamiento quedan fuera del modelo. [Asistencia histórica](monumental-match-attendance.md) · [Supuestos de la hinchada](../../engine/src/fans.rs) · [Contrato MCP](../../tools/santiago-mcp/server.mjs)
- Usa como referencia los centros del catálogo: MUT `(-70.601132, -33.417280)` y Monumental `(-70.605966, -33.506537)`. Empieza a unos 780 m. Guarda la cámara y el reloj de cada plano. Comprueba en el visor que el cambio se entienda a la misma distancia. [Catálogo de lugares](../../data/mvp/named_places.csv) · [Comparación nativa de manzanas](https://discord.com/channels/1573514576400752640/1573734041541288053/1573782542459605133)

```json
{"changes":[{"kind":"event","target":{"type":"place","id":"monumental"},"op":"set","value":39222,"assumed_end_half_hour":34,"audience":"colo_colo"}]}
```

```json
{"changes":[{"kind":"jobs","target":{"type":"block","id":"13114131009033"},"op":"set","value":53},{"kind":"jobs","target":{"type":"block","id":"13118031003002"},"op":"set","value":4679},{"kind":"event","target":{"type":"place","id":"mut"},"op":"set","value":39222,"assumed_end_half_hour":34,"audience":"colo_colo"}]}
```

## Qué puede mostrar el gráfico

- Usa `events[0].fan_base.by_mode` para el **viaje al partido**. La repetición aislada del intercambio completo en `eee8cf8b60c35bba7810771d25c12cae9ce84370` mantuvo 39.222 asistentes y las mismas proporciones de origen en 52 comunas. A → B estimó transporte público 23.687,5 → 27.789,9, auto 12.758,9 → 9.397,8 y otros modos 2.775,6 → 2.034,3. [Corrida A/B](https://discord.com/channels/1573514576400752640/1573809606277337280/1573818751726387290) · [Cálculo por origen](../../engine/src/fans.rs)
- Para una comparación de **entradas al Metro en un día laboral**, usa la diferencia B menos A. Lee `asked_stations[].scenario_daily` en la respuesta del chat o `result.stations[].scenario_daily` en la caché. La misma corrida estimó Tobalaba −2.584,3 (`TB-TOB`), Pedrero +2.278,2 (`PE`) y Mirador +1.843,5 (`MA`). Estas cifras corresponden al día laboral. El motor no identifica un trasvase entre estaciones ni incluye los regresos del partido. [Corrida A/B](https://discord.com/channels/1573514576400752640/1573809606277337280/1573818751726387290) · [Contrato MCP](../../tools/santiago-mcp/server.mjs)

## Evidencia antes de grabar

- Los nuevos archivos de caché coincidieron con el paquete inicial salvo en `created_at` y `elapsed_ms`. La publicación de A y B registró dos llamadas al relay y cero al cálculo de simulación. Quedan pendientes la integración de los planos y los gráficos, la comprobación en proyector y la prueba de tiempo real. [Corrida A/B](https://discord.com/channels/1573514576400752640/1573809606277337280/1573818751726387290) · [#370](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/370) · [#377](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/377)
- Eugenio comprobó los cambios de altura en ambas manzanas en el visor real. Ese ensayo usó el perfil anterior. Confirma que la nueva toma muestre las dos revisiones del cálculo con hinchada fija. Confirma también qué puede mostrar el visor del cambio de recinto y del público. [Ensayo de Eugenio](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/issues/265#issuecomment-7068380726)
- Guarda los dos resultados, los supuestos, la revisión del motor y los planos con sus tiempos. Mantén activo el servicio elegido para entregar la caché al visor. Abre la toma desde la caché y comprueba que el registro muestre cero llamadas al cálculo de simulación durante la reproducción. Confirma que las escenas del visor y las cifras de ambos gráficos coincidan con la corrida guardada. Mide el tiempo de la toma y revisa un fotograma de cada plano a tamaño de proyector.
- La separación entre cálculo y reproducción ya se usa en simulación de transporte. SUMO genera gráficos desde la salida de trayectorias, y el lector OTFVis de MATSim abre una secuencia de estados guardados por tiempo. Para este caso basta que la caché entregue las escenas y cifras que usa el guion. [SUMO](https://sumo.dlr.de/docs/Tools/Visualization.html) · [Salida FCD](https://eclipse.dev/sumo/docs/Simulation/Output/FCDOutput.html) · [Escritor OTFVis](https://github.com/matsim-org/matsim-libs/blob/main/contribs/otfvis/src/main/java/org/matsim/vis/otfvis/OTFFileWriter.java) · [Lector OTFVis](https://github.com/matsim-org/matsim-libs/blob/main/contribs/otfvis/src/main/java/org/matsim/vis/otfvis/data/fileio/OTFFileReader.java)
