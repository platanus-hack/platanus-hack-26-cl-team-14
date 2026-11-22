# Replay del Monumental en el MUT

La propuesta de ensayo dedica 105 segundos al MUT. En el plano fijo se ve el sitio antes y después del traslado. Luego se muestra cómo llegaría la hinchada. El guion del pitch define el orden y el tiempo. [Guion #438](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/438)

## Abrir los replays

Extrae `mut-venue-ab.zip` del hilo V3 y copia `mut-monumental-a.json` y `mut-monumental-b.json` a `data/replays/`. Llama a `run_replay` con `mut-monumental-a` y luego con `mut-monumental-b`. Mantén el visor y el MCP conectados al mismo motor. Usa una revisión que incluya #440 para el traslado y #441 para las leyendas. El paquete incluye las dos solicitudes completas para regenerar con `create_replay`, las respuestas, las sumas SHA-256 y el registro de reproducción. La corrida publicó dos llamadas al relay y cero llamadas al cálculo durante la reproducción. [Paquete A/B](https://discord.com/channels/1573514576400752640/1573809606277337280/1573844937198411918) · [Revisión independiente](https://discord.com/channels/1573514576400752640/1573809606277337280/1573845336772841493)

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
| Entradas al Metro en un día laboral, B menos A | Tobalaba −2.584,3, Pedrero +2.278,2, Mirador +1.843,5 | `result.stations[].scenario_daily` para `TB-TOB`, `PE` y `MA` |

Las cifras de entradas al Metro excluyen el partido. El motor estima entradas por estación. No identifica trasvases entre estaciones. Los seis cierres de vías agregan de 5,3 a 31,2 segundos de viaje en auto a nueve pares dirigidos de comunas, en flujo libre. Los nueve pares conservan conexión. [Recibos del motor](https://discord.com/channels/1573514576400752640/1573809606277337280/1573844937198411918)

## Toma para la presentación

Tinta mostró el MUT a las 17:00 con una cámara fija a 650 m. La toma B muestra la cancha, las tribunas y la torre vecina. Maitén y Tinta pueden usar las cachés verificadas con ese encuadre. Las claves de cámara del paquete conservan los tiempos del ensayo anterior. El ensayo narrado y la legibilidad en proyector siguen pendientes. [Tomas nativas](https://discord.com/channels/1573514576400752640/1573809836699820163/1573844947705004155) · [Revisión de las tomas](https://discord.com/channels/1573514576400752640/1573809606277337280/1573845749089706046)
