# V4 candidate · Line 1 interruption on 18 November 2024

This candidate uses a short, observed suspension for the V4 slide. DTPM records two complete half-hours inside the suspension and gives a weekday reference from the same release. The input and score below are proposed. No simulation score exists for this event.

## Observed service

At 13:10 on Monday 18 November, Metro reported Line 1 service only from San Pablo to Baquedano and from Escuela Militar to Los Dominicos. At 13:14 it named seven stations without Line 1 service. Transfers at Los Leones and Tobalaba were suspended. Red announced parallel bus service and a shuttle between the two open ends. Metro announced full service at 14:34. Use 13:30–14:30 for the two bins fully inside the reported suspension. [Metro notices and times](https://redgol.cl/tendencias/alerta-metro-de-santiago-reportan-cierre-de-varias-estaciones-de-la-linea-1-del-servicio) · [Red notice reproduced by Cooperativa](https://cooperativa.cl/noticias/pais/transportes/metro/metro-repone-el-servicio-en-toda-la-red-tras-suspension-de-linea-1/2024-11-18/134444.html)

## Recorded Metro boarding stages

| Station | Engine ID | Mon 18 Nov 13:30 | Mon 14:00 | Thu 21 Nov 13:30 | Thu 14:00 |
| --- | --- | ---: | ---: | ---: | ---: |
| Salvador | SA | 0 | 0 | 822 | 770 |
| Manuel Montt | MM | 0 | 0 | 1,104 | 1,074 |
| Pedro de Valdivia | PV | 0 | 0 | 1,080 | 1,067 |
| Los Leones | LE-LEN | 749 | 962 | 1,404 | 1,433 |
| Tobalaba | TB-TOB | 846 | 951 | 1,846 | 1,882 |
| El Golf | GO | 0 | 0 | 367 | 384 |
| Alcántara | AL | 0 | 0 | 291 | 286 |
| Baquedano | BA-BQ | 1,242 | 1,590 | 706 | 712 |
| Escuela Militar | EM | 175 | 215 | 909 | 1,041 |

Use `Tabla de Etapas/2024-11-18.etapas.csv.gz` and `Tabla de Etapas/2024-11-21.etapas.csv.gz` from the dated archive. The Thursday member has SHA-256 `4a8fef7747da7b57b6b30e0082a3b01fc923335f26098ae81efb767a3356e3e7`. See [the holiday brief](v4-holiday.md) for the Monday hash and extraction command. Filter to `METRO` and the local date in `tiempo_subida`. Group by `parada_subida` and `media_hora_subida`. Every selected row has service expansion factor 1. Each number counts recorded stages in the half-hour starting at the column time. The five single-line stations have zero entries during both bins. Shared station names combine boardings across lines. Thursday is a weekday reference. These counts cannot identify riders who switched routes or establish the effect of the suspension alone. [DTPM archive](https://www.dtpm.cl/descargas/modelos_y_matrices/Tabla-de-Etapas-%20Nov24.zip) · [Station IDs](../../data/mvp/metro_training.json)

## Proposed model check

Represent the interruption as a timed change to Line 1 service across the affected segment. Keep both outer sections and service on the other lines. Disable the two Line 1 transfers. Represent the announced bus shuttle if its timetable or a stated frequency assumption is available. The current line edit can change an individual directed pattern. Its routing comparison starts at 08:00 on a 2026 weekday. A citywide Metro removal or a station-only closure cannot represent this historical split service. [Transit edit and routing assumptions](../../engine/src/editable_transit.rs)

Check that the five single-line stations report zero in bins 27 and 28. Keep those known closure zeros out of the behavior score. For Baquedano and Escuela Militar, compare the predicted change from no edit to closure against the observed Monday-minus-Thursday change in each of the four cells. Sum the absolute errors and divide by the 3,368 Thursday stages in those cells. A prediction of no change scores 0.883 on this proposed scale. Lower is better. Freeze the cells before fitting, and keep the date and reference limits with the score.
