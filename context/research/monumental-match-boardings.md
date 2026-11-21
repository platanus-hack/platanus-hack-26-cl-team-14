# Metro boardings after a Monumental match

The stadium demo shows people leaving after a match. A public DTPM workbook gives a local check on the timing and scale of station use. The ANFP match report confirms that Colo-Colo played Coquimbo Unido at the Monumental on Saturday 26 April 2025. The match started on time at 16:00. The report leaves attendance blank. AS reports 17,756 spectators. [ANFP match report, pp. 1 and 4](https://www.campeonatochileno.cl/wp-content/uploads/2025/04/Informe-Arbitro-COLO-COLO-vs-COQUIMBO.pdf) · [AS match results](https://chile.as.com/resultados/futbol/chile/2025/jornada/regular_a_9/)

| Station | Sat 26 Apr 18:00 | 18:30 | Sat hour total | Sun 27 Apr 18:00 | 18:30 | Sun hour total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Pedrero | 1,675 | 945 | 2,620 | 83 | 102 | 185 |
| Mirador | 854 | 742 | 1,596 | 308 | 272 | 580 |
| San Joaquín | 234 | 329 | 563 | 57 | 48 | 105 |
| Bellavista de La Florida | 496 | 547 | 1,043 | 394 | 426 | 820 |

Each column shows entries for a local half-hour starting at the stated time. These are Metro boardings from payment validations. They exclude fare evasion. The Sunday columns give context. Travel patterns and service can differ by day type. The Saturday counts include all riders at these stations. They do not identify match attendees, crowd size, car or bus use, or home locations. [DTPM methodology and publication dates](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) · [April 2025 workbook](https://www.dtpm.cl/descargas/modelos_y_matrices/Subida_Paradero_Estacion_2025.04.xlsb)

For the pitch, use the observed 18:00–19:00 station counts as local context for the departure animation. A model check needs **total Saturday station entries** for the same hour and stations. Estimating excess demand requires several comparable Saturdays without a match and a check of station closures and service changes. The current April 2026 score uses a different week.

## Reproduce the table

Use the April 2025 workbook with SHA-256 `570d643a01c677386a2814e678433562f3c7e6bf3302001c7e78e8dc33989999`. The `FICHA_DATOS` sheet assigns `SABADO` to 26 April and `DOMINGO` to 27 April. In `SUBIDAS_2025_04`, select `Modo = Metro` and the four station names in `Paradero`. Select `Media_hora = 0.75` and `37/48` (Excel fractions of a day). Read `Subidas_Promedio` for each day type and add the two half-hours.

## A second match on 8 November 2025

Colo-Colo Femenino played Palestino at the Monumental at 20:00 on Saturday 8 November 2025. The dated DTPM stages show a peak at Pedrero at 22:00. The four other stations in this table each had fewer recorded boardings in that half-hour than in the preceding half-hour. The counts give a second reference for the location and timing of a station peak. Match attendance, the final whistle time and a comparable Saturday without a match remain unverified. [ANFP match record](https://www.campeonatochileno.cl/match/colo-colo-femenino-palestino-femenino-2025-11-08/) · [DTPM stages](https://www.dtpm.cl/descargas/modelos_y_matrices/Etapas_Nov_2025.zip)

| Station | 21:30 | 22:00 | 22:30 |
| --- | ---: | ---: | ---: |
| Pedrero | 144 | 814 | 128 |
| Mirador | 297 | 212 | 94 |
| Macul | 276 | 213 | 167 |
| San Joaquín | 89 | 73 | 50 |
| Camino Agrícola | 56 | 50 | 40 |

In the November stages ZIP, extract `Tabla de Etapas/2025-11-08.etapas.csv.gz` (SHA-256 `4a72cab5324db62c9824c84a84be6e893f8548547869d43392f47b189ba9c230`). Filter the pipe-delimited records to `tipo_transporte = Metro`, the five stations in `parada_subida`, and 8 November in `tiempo_subida`. Count rows by `parada_subida` and `media_hora_subida`. Across the five stations and all hours, 44,906 rows each had service expansion factor 1. No two rows shared station, boarding timestamp, operator, encrypted ID and both trip and stage counters.

The [DTPM catalog](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) lists 8–14 November 2025. The [companion workbook](https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2025.11.xlsx) gives 15–16 November for its weekend rows in `FICHA_DATOS`. Use the dated stages for 8 November. The workbook's Saturday rows have no date column and do not serve as a comparison here.
