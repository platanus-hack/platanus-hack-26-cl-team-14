# Replay del Monumental en el MUT

La propuesta de ensayo dedica 105 segundos al MUT. En el plano fijo se ve el sitio antes y después del traslado. Luego se muestra cómo llegaría la hinchada. El guion del pitch define el orden y el tiempo. [Guion #438](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/438)

## Abrir los replays

Extrae `mut-itinerary-ab.zip` del hilo V3 y copia `mut-monumental-routes-a.json` y `mut-monumental-routes-b.json` a `data/replays/`. Mantén el visor y el MCP conectados al mismo motor. Inicia el visor con `SANTIAGO_PLAYBACK_RATE=240` para la comparación. Reproduce A, confirma que se vea el sitio del MUT y reproduce B. Las dos escenas parten a las 13:30 con la misma cámara a 650 m. El gráfico aparece 18 segundos después de iniciar B. Si el reloj avanza a la velocidad indicada, eso corresponde a las 14:42 en la ciudad.

El paquete incluye solicitudes para regenerar con `create_replay`, respuestas, sumas SHA-256 y el registro de reproducción. La corrida en `f8ed126094be47bf8d3e0d3b1c7cc2c0b74bc289` registró dos llamadas al relay y cero cálculos durante la reproducción. Una revisión independiente cotejó los grupos y los hashes. [Paquete A/B](https://discord.com/channels/1573514576400752640/1573809606277337280/1573870047825236010) · [Revisión](https://discord.com/channels/1573514576400752640/1573809606277337280/1573870480417624136)

- **A.** El partido se juega en el Monumental, con la ciudad de referencia.
- **B.** El motor acepta las 35 ediciones físicas. Traslada `mesh:152:317` al MUT y mantiene el anillo de tribunas y la cancha interior. Retira 28 edificios dentro del sitio, incluidos ocho dentro del anillo. Cierra seis vías que quedan enteras dentro del perímetro. Conserva las 11 vías que continúan fuera y la torre vecina `mesh:232:57`. [Perímetro y vías](https://discord.com/channels/1573514576400752640/1573809606277337280/1573843035643453634) · [Decisión de cierre](https://discord.com/channels/1573514576400752640/1573809606277337280/1573844486126178349)
- **Empleos en B.** El escenario fija 53 empleos en la manzana `13114131009033` de Tobalaba y 4.679 en `13118031003002` de Macul. Son supuestos para las dos manzanas. [Decisión del intercambio](../decisions/20261121-1315-mut-monumental-swap.md)
- **Partido en A y B.** Cada caso usa 39.222 asistentes del mismo perfil de Colo-Colo, con los mismos conteos de origen en 52 comunas y las 17:00 como hora supuesta de término. El motor estima los modos a partir de viajes laborales entre comunas. La oferta de fin de semana y el estacionamiento quedan fuera del cálculo. [Asistencia histórica](monumental-match-attendance.md) · [Supuestos del perfil](../../engine/src/fans.rs)

## Cifras para el pitch

| Medida | Resultado estimado | Campo en la caché |
| --- | --- | --- |
| Hinchas en transporte público | 23.687,5 → 27.789,9 | `result.events[0].fan_base.by_mode.public` |
| Hinchas en auto | 12.758,9 → 9.397,8 | `result.events[0].fan_base.by_mode.car` |
| Hinchas en otros modos | 2.775,6 → 2.034,3 | `result.events[0].fan_base.by_mode.other` |
| Entradas al Metro en un día laboral, B menos A | Tobalaba −2.598,5, Pedrero +2.249,9, Mirador +1.826,5 | `result.stations[].scenario_daily` para `TB-TOB`, `PE` y `MA` |

Las cifras de entradas al Metro excluyen el partido. El motor estima entradas por estación. No identifica trasvases entre estaciones. Los seis cierres de vías agregan de 5,3 a 31,2 segundos de viaje en auto a nueve pares dirigidos de comunas, en flujo libre. Los nueve pares conservan conexión. [Recibos del motor](https://discord.com/channels/1573514576400752640/1573809606277337280/1573844937198411918)

## Cobertura de los itinerarios

En ambas sedes tienen ruta los 52 grupos de auto y 16 de los 52 grupos de transporte público. Las rutas públicas cubren 9.496,7 hinchas estimados en A (40,1 %) y 11.409,6 en B (41,1 %). El gráfico incluye también los grupos sin ruta. Las opciones de transporte público usan horarios de referencia de un día hábil. La llegada entre 14:00 y 15:00 es un supuesto del escenario. Todos los caminos trazados terminan en el portón exterior. [Contrato](v1-fan-itinerary-contract.md) · [Paquete y cobertura](https://discord.com/channels/1573514576400752640/1573809606277337280/1573870047825236010)

## Toma para la presentación

Las tomas de referencia a las 17:00 muestran la cancha, las tribunas y la torre vecina en el mismo encuadre. Tagua pudo leer el traslado y el gráfico en el ensayo técnico con el par fijo anterior. Falta revisar en el visor el nuevo par de las 13:30 con itinerarios y medir el ensayo narrado en proyector. [Tomas a las 17:00](https://discord.com/channels/1573514576400752640/1573809836699820163/1573844947705004155) · [Ensayo técnico](https://discord.com/channels/1573514576400752640/1573809606277337280/1573857623898329213)
