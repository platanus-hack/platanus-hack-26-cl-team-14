# Ocio y compras en Santiago: cuánto, cuándo, cómo, qué tan lejos

**Pregunta:** ¿los viajes a un partido o a un mall se parecen a los viajes al trabajo de día laboral?

EOD 2012 = encuesta SECTRA, mirror U. de Chile (`users.dcc.uchile.cl/~egraells/gds-data/eod2012.tgz`). Peso = `viajes.Factor<Día>Normal × personas.Factor_<Día>Normal`, igual que `tools/pack/work_departures.py`. Temporada normal. Propósito (`Tablas_parametros/Proposito.csv`): 1 trabajo, 11 compras, 13 recreación, 6 visita, 7 volver a casa. Modo `ModoAgregado`: auto 1; público 2–7, 9–12; caminata 17; bici 18. Script: `tools/pack/eod_leisure.py`. Distancia = mediana ponderada de `DistanciaViaje.DistEuclidiana`, sin las imputadas. "% sin volver" = viajes del propósito ÷ viajes que no son "volver a casa".

| Dato | Valor | Población/año | Fuente y cálculo | Conf. |
|---|---|---|---|---|
| Viajes totales por día | Laboral 18,46 M · Sábado 14,79 M · Domingo 12,50 M | Gran Santiago 2012 | EOD 2012, calculado | A |
| Compras, % sin volver | Laboral 16 % · Sáb 34 % · Dom 38 % | 2012 | EOD, propósito 11 | A |
| Recreación, % sin volver | Laboral 3,5 % · Sáb 15 % · Dom 16 % | 2012 | EOD, propósito 13 | A |
| Ocio total (compras+recreación+visita+comer), % sin volver | Laboral 26 % · Sáb 61 % · Dom 68 % | 2012 | EOD, propósitos 6, 9, 11, 13 | A |
| Trabajo, % sin volver | Laboral 30 % · Sáb 17 % · Dom 12 % | 2012 | EOD, propósito 1 | A |
| Viajes de recreación | Laboral 0,35 M · Sáb 1,18 M · Dom 1,08 M | 2012 | EOD (muestra: 1.232 / 564 / 504 viajes) | A |
| Modo, trabajo laboral | auto 31 · público 42 · caminata 16 · bici 5 % | 2012 | EOD | A |
| Modo, recreación sábado | auto 40 · público 17 · caminata 39 · bici 3 % | 2012 | EOD | A |
| Modo, recreación domingo | auto 42 · público 23 · caminata 28 · bici 5 % | 2012 | EOD | A |
| Modo, compras sábado | auto 23 · público 14 · caminata 61 % | 2012 | EOD | A |
| Distancia mediana (línea recta) | Trabajo 7,1 km · Recreación sáb 3,4 / dom 5,1 km · Compras sáb 1,5 km | 2012 | EOD | A |
| Tiempo mediano de viaje | Trabajo 40 min · Recreación sáb 20 / dom 25 min · Compras 14 min | 2012 | EOD, `TiempoViaje` | A |
| Hora de inicio, recreación | Sáb: 32 % entre 10–14 h, 34 % entre 15–20 h. Dom: 36 % / 41 % | 2012 | EOD, `HoraIni` | A |
| Hora de inicio, compras sábado | Pico 11 h (19 %); 56 % entre 10–14 h | 2012 | EOD | A |
| Viajes en transporte público | Sáb/laboral 0,65 · Dom/laboral 0,49 | 2012 | EOD, modo público, todos los propósitos | A |
| Entradas Metro, total red | Laboral 2,47 M · Sáb 1,41 M (**0,57**) · Dom 0,78 M (**0,32**) | 13–17, 11 y 12 abril 2026 | `data/mvp/metro_observations.json`, suma de las medias horas de las 126 estaciones | A |
| Hora pico Metro | Laboral 07:30 · Sábado 13:00 · Domingo 18:30 | abril 2026 | mismo archivo | A |
| Sáb y dom ÷ laboral por estación | Tobalaba 0,57 / 0,33 · Pedrero 0,62 / 0,33 · Mirador 0,79 / 0,50 | abril 2026 | mismo archivo | A |
| Tiempo en ocio y vida social, RM | 5,78 h día de semana · 7,95 h fin de semana | 12+ años, 2015 | [INE, La dimensión personal del tiempo, Tabla 8](https://www.ipsuss.cl/ipsuss/site/docs/20180307/20180307102511/la_dimensi__n____enut.pdf) | B |
| Asistencia a eventos (cultura, deporte, etc.), Chile | Participa 4,2 % día de semana · 11,9 % fin de semana; 3,05 h en fin de semana | 12+ años, 2015 | Mismo informe, Tabla 12 | B |
| Asistencia a eventos culturales, de entretención y deportivos, Chile | Participa 2,9 % día de semana · 6,1 % fin de semana; 3:08 h en fin de semana | 12+ años, 2023 | [INE, Informe principales resultados II ENUT, Tablas 38–39](https://www.ine.gob.cl/docs/default-source/uso-del-tiempo-tiempo-libre/publicaciones-y-anuarios/ii-enut/informe-de-principales-resultados-ii-enut-2023.pdf) | B |
| Ocio y vida social, Chile | Participa 77,7 % (3:14 h) día de semana · 82,0 % (5:00 h) fin de semana | 12+ años, 2023 | Mismo informe | B |
| Compras para el hogar, Chile | Participa 29,2 % (1:00 h) día de semana · 26,6 % (1:17 h) fin de semana | 12+ años, 2023 | Mismo informe, Tablas 17–18 | B |
| Ocio en RM, ENUT 2023 | no encontrado en texto (solo en gráfico) | — | — | — |
| Partidos y malls como propósito separado en la EOD | no encontrado (la EOD solo tiene "recreación" y "compras") | — | — | — |

## Qué significa para el motor

- **El fin de semana es de ocio.** El sábado, 61 % de los viajes que no son "volver a casa" son compras, recreación, visitas o comer. El domingo es 68 %. El día laboral es 26 %. Un sábado modelado con viajes al trabajo describe mal la ciudad.
- **El modo de ocio difiere del modo al trabajo (comparación proxy).** Recreación de fin de semana: 40–42 % auto y 17–23 % público. Trabajo de día laboral: 31 % auto y 42 % público. «Público» usa los códigos 2–7 y 9–12; los códigos mixtos 8 y 13–15 quedan en «otro». Compras (todas, propósito 11): más de 60 % a pie, mediana 1,5 km. El script no selecciona malls ni el estadio, así que estas cifras no miden el acceso al MUT ni el error del modelo de hinchas. Para el pitch, el reparto de recreación sirve como segunda hipótesis: comparar ambos resultados con el mismo público, orígenes y horario.
- **Distancia.** Recreación: 3,4 km (sáb) y 5,1 km (dom). Trabajo: 7,1 km. La distancia específica de viajes a un mall no se conoce: la EOD no los separa.
- **Fin de semana observado en Metro:** el sábado 11 de abril de 2026 tuvo 0,57 de las entradas del promedio laboral del 13 al 17 de abril; el domingo 12, 0,32. Pedrero 0,62 y Mirador 0,79. El pico pasa de 07:30 a 13:00 el sábado. Son descripciones de ese fin de semana. Incluyen las 31 estaciones reservadas por `tools/pack/check_score.py`, así que no sirven como multiplicador del motor. Un multiplicador debe salir solo de las estaciones de entrenamiento o de otra fuente, y declararse como supuesto.
- **Cuidado con el tamaño de muestra.** La EOD tiene solo 564 viajes de recreación en sábado y 504 en domingo. Úselos para shares, no para matrices origen-destino. El fin de semana de Metro es un solo sábado y un solo domingo (11 y 12 de abril de 2026). No revisamos si ese fin de semana hubo un partido.
