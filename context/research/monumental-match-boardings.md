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

Colo-Colo Femenino played Palestino at the Monumental on Saturday 8 November 2025. The ANFP lists the match at 20:00. DaleAlbo reported kickoff at 20:02, 3,746 controlled attendees and full time at 21:56. Use 22:00–23:00 for entries after full time. The 21:30 half-hour includes the final whistle. [ANFP match record](https://www.campeonatochileno.cl/match/colo-colo-femenino-palestino-femenino-2025-11-08/) · [DaleAlbo live log](https://dalealbo.cl/colocolo/colo-colo-femenino-vs-palestino-en-vivo-minuto-a-minuto-por-los-cuartos-de-final)

| Station | 8 Nov 2025 entries at 21:30 / 22:00 / 22:30 | 23 Nov 2024 entries at 21:30 / 22:00 / 22:30 | Difference at 22:00–23:00 |
| --- | ---: | ---: | ---: |
| Pedrero | 144 / 814 / 128 | 72 / 56 / 41 | +845 |
| Mirador | 297 / 212 / 94 | 211 / 118 / 86 | +102 |
| Macul | 276 / 213 / 167 | 212 / 179 / 142 | +59 |
| San Joaquín | 89 / 73 / 50 | 52 / 38 / 42 | +43 |
| Camino Agrícola | 56 / 50 / 40 | 81 / 43 / 39 | +8 |

The dated 2024 Saturday gives a descriptive comparison of all-rider entries at the same stations and times. At Pedrero, DTPM recorded 942 entries after full time in 2025 and 97 in the same hour in 2024. The 2024 date was the eve of the regional runoff. Rail service frequency, disruptions and local demand remain unverified. Keep the November score pending until a comparable Saturday reference and matched predictions are available. Use the separate alighting measure for arrivals. [DTPM November 2024 stages](https://www.dtpm.cl/descargas/modelos_y_matrices/Tabla-de-Etapas-%20Nov24.zip) · [Servel runoff regions](https://www.servel.cl/biblioteca-de-documentos/procesos-electorales/elecciones-2024/documentos-elecciones-2024/regiones-con-segunda-vuelta-de-gobernadores-regionales/) · [Pre-match alightings](monumental-pre-match-alightings.md)

In the November stages ZIP, extract `Tabla de Etapas/2025-11-08.etapas.csv.gz` (SHA-256 `4a72cab5324db62c9824c84a84be6e893f8548547869d43392f47b189ba9c230`). Filter the pipe-delimited records to `tipo_transporte = METRO`, the five stations in `parada_subida`, and 8 November in `tiempo_subida`. Count rows by `parada_subida` and `media_hora_subida`. Across the five stations and all hours, 44,906 rows each had service expansion factor 1. No two rows shared station, boarding timestamp, operator, encrypted ID and both trip and stage counters.

In the November 2024 ZIP, use the dated 23 November gzip member (SHA-256 `eade0af519cbd7d425dfcf177ce187c173e455ba69898f8e8c90482a71ed1525`). Apply the same filters for 23 November 2024. Mirtha checked the ZIP member CRC and reported 38,953 dated entry rows across the five stations, each with expansion factor 1. The extracted table and source notes are in the V4 handoff. [V4 source handoff](https://discord.com/channels/1573514576400752640/1573813174694846566/1573861223798407333)

The [DTPM catalog](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) lists 8–14 November 2025. The [companion workbook](https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2025.11.xlsx) gives 15–16 November for its weekend rows in `FICHA_DATOS`. Use the dated stages for 8 November. The workbook's Saturday rows carry day type only, so their date remains unverified. Contemporary reporting also identifies the Monumental as a polling site for that election. [Polling-site report](https://dalealbo.cl/colocolo/elecciones-presidenciales-modifican-el-plan-de-colo-colo-para-enfrentar-a-union-la-calera)
