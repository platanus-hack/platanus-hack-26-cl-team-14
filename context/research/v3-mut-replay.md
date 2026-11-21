# Replay MUT → Monumental

Borrador para el primer caso de V3. Propongo un máximo de 55 segundos para el caso y la comparación final. El pitch completo dura hasta tres minutos. [Dirección de V3](../verticals.md)

> ¿Qué cambia si Colo-Colo juega donde está el MUT y movemos los empleos modelados de esa manzana a Macul?

| Tiempo | Qué se ve | Qué se dice |
| --- | --- | --- |
| 0:00–0:10 | El presentador escribe la pregunta en la terminal del mapa. Aparecen los supuestos. | «Comparamos el mismo partido en dos lugares, con 39.222 asistentes». |
| 0:10–0:22 | La terminal se oculta. La cámara muestra el estadio actual en Macul y luego el MUT junto a Tobalaba. | «Este es el partido en el Monumental. Ahora lo llevamos a Tobalaba». |
| 0:22–0:40 | La cámara muestra la escena B con el mismo encuadre de Tobalaba. Vuelve a Macul con el mismo encuadre de la escena A. | «Intercambiamos los empleos modelados de estas dos manzanas. La hinchada conserva sus comunas de origen». |
| 0:40–0:55 | Dos barras apiladas comparan transporte público, auto y otros modos. La terminal vuelve al terminar el replay. | Lee una diferencia del gráfico. Di «asistentes estimados por el modelo». |

## Preparar las dos escenas

- Parte de la base para cada llamada. Usa `simulate_city` antes del pitch. A agrega el partido a `monumental`. B fija 53 empleos en la manzana `13114131009033`, 4.679 en `13118031003002` y agrega el partido a `mut`. Los conteos corresponden a manzanas. Comprueba los valores en la base que se use para grabar. [Decisión del intercambio](../decisions/20261121-1315-mut-monumental-swap.md) · [Ensayo A/B por MCP](https://discord.com/channels/1573514576400752640/1573734041541288053/1573752258351341739)
- Declara 39.222 asistentes, la misma hinchada de Colo-Colo y las 17:00 como hora supuesta de término. La cifra de asistencia viene del Superclásico del 31 de agosto de 2025. El modelo actual usa tiempos de viaje por comuna. La hora de término no altera la oferta de transporte en el cálculo. [Asistencia histórica](monumental-match-attendance.md) · [Contrato MCP](../../tools/santiago-mcp/server.mjs)
- Usa como referencia los centros del catálogo: MUT `(-70.601132, -33.417280)` y Monumental `(-70.605966, -33.506537)`. Empieza a unos 780 m. Guarda la cámara y el reloj de cada plano. Comprueba en el visor que el cambio se entienda a la misma distancia. [Catálogo de lugares](../../data/mvp/named_places.csv) · [Comparación nativa de manzanas](https://discord.com/channels/1573514576400752640/1573734041541288053/1573782542459605133)

```json
{"changes":[{"kind":"event","target":{"type":"place","id":"monumental"},"op":"set","value":39222,"assumed_end_half_hour":34,"audience":"colo_colo"}]}
```

```json
{"changes":[{"kind":"jobs","target":{"type":"block","id":"13114131009033"},"op":"set","value":53},{"kind":"jobs","target":{"type":"block","id":"13118031003002"},"op":"set","value":4679},{"kind":"event","target":{"type":"place","id":"mut"},"op":"set","value":39222,"assumed_end_half_hour":34,"audience":"colo_colo"}]}
```

## Qué puede mostrar el gráfico

- Usa `events[0].fan_base.by_mode` para comparar el **viaje al partido**. La prueba de #323 comparó el evento en ambos recintos y estimó 23.688 → 27.790 asistentes en transporte público. B en esa prueba cambió solo el recinto. Vuelve a calcular el gráfico con el intercambio completo antes de fijar las cifras. [Prueba de chat de Teodoro](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/323#issuecomment-7068321956) · [Cálculo por origen](../../engine/src/fans.rs)
- Si se necesita una cifra de día laboral, solicita `TOBALABA`, `PEDRERO` y `MIRADOR` en `stations` en ambas llamadas. Lee `asked_stations[].scenario_daily` como entradas al Metro en un día laboral. Muestra cada estación por separado. El motor no identifica un trasvase entre estaciones ni incluye los regresos del partido en esas cifras. [Contrato MCP](../../tools/santiago-mcp/server.mjs)

## Evidencia antes de grabar

- Eugenio comprobó los cambios de altura en ambas manzanas en el visor real. Ese ensayo usó el perfil anterior. Confirma que la nueva toma muestre las dos revisiones del cálculo con hinchada fija. Confirma también qué puede mostrar el visor del cambio de recinto y del público. [Ensayo de Eugenio](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/issues/265#issuecomment-7068380726)
- Guarda los dos resultados, los supuestos, la revisión del motor y los planos con sus tiempos. Abre la toma desde la caché con el motor apagado y confirma que el gráfico usa los mismos resultados que la imagen. Mide el tiempo de la toma y revisa un fotograma de cada plano a tamaño de proyector.
- La separación entre cálculo y reproducción ya se usa en simulación de transporte. SUMO genera gráficos desde la salida de trayectorias, y el lector OTFVis de MATSim abre una secuencia de estados guardados por tiempo. Para este caso basta que la caché entregue las escenas y cifras que usa el guion. [SUMO](https://sumo.dlr.de/docs/Tools/Visualization.html) · [Salida FCD](https://eclipse.dev/sumo/docs/Simulation/Output/FCDOutput.html) · [Escritor OTFVis](https://github.com/matsim-org/matsim-libs/blob/main/contribs/otfvis/src/main/java/org/matsim/vis/otfvis/OTFFileWriter.java) · [Lector OTFVis](https://github.com/matsim-org/matsim-libs/blob/main/contribs/otfvis/src/main/java/org/matsim/vis/otfvis/data/fileio/OTFFileReader.java)
