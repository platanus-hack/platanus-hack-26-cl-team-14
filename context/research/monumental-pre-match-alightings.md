# Estimated Metro alightings before a Monumental match

Colo-Colo Femenino played Palestino at the Monumental on 8 November 2025 at 20:00. Pedrero recorded 643 estimated alightings from 19:00 to 20:00. This was above its preceding hour (406). Mirador fell from 578 to 444. The table counts all riders with an estimated alighting. It gives a station reference for the timing of arrival, without identifying the riders' destination after they left Metro. [ANFP match record](https://www.campeonatochileno.cl/match/colo-colo-femenino-palestino-femenino-2025-11-08/) · [DTPM source](https://www.dtpm.cl/descargas/modelos_y_matrices/Etapas_Nov_2025.zip)

| Station | 17:30 | 18:00 | 18:30 | 19:00 | 19:30 | 20:00 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Pedrero | 140 | 155 | 251 | 341 | 302 | 163 |
| Mirador | 301 | 301 | 277 | 221 | 223 | 200 |
| Macul | 352 | 347 | 355 | 308 | 337 | 370 |
| San Joaquín | 54 | 69 | 57 | 66 | 67 | 58 |
| Camino Agrícola | 128 | 119 | 138 | 125 | 119 | 112 |

Times are local half-hour starts. Use the dated `Tabla de Etapas/2025-11-08.etapas.csv.gz` member, SHA-256 `4a72cab5324db62c9824c84a84be6e893f8548547869d43392f47b189ba9c230`. In the pipe-delimited file select `tipo_transporte = METRO` and `tiene_bajada = 1`. Select 8 November in `tiempo_bajada`. Normalize station names in `parada_bajada` by removing accents and ignoring case. Keep Pedrero, Mirador, Macul, San Joaquín and Camino Agrícola. Keep the six `media_hora_bajada` values from `17:30:00` through `20:00:00` inclusive. Count rows by station and half-hour. All 6,056 selected rows had expansion factor 1. No rows duplicated the operator, encrypted ID, trip and stage counters, boarding and alighting times, and alighting station.

DTPM infers alightings from fare records. It cannot infer them for all stages. The observed pre-match pattern alone gives no match effect. A comparison needs another Saturday with comparable service and no match. Keep the arrival count separate from the [post-match entries](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/228). [DTPM methods](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) · [November file structure](https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura%20Tabla%20de%20Etapas%20Nov25.txt)
