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
