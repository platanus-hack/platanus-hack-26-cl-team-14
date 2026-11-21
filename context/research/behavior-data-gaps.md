# Datos para modelos de comportamiento: qué hay y qué falta

Pedido de Mac (16:27 UTC): que los what-if (mover un lugar, un modo nuevo, más teletrabajo) usen modelos existentes y todos los datos necesarios. Este archivo cubre solo el **eje de datos**. Cómo meten los modelos un modo nuevo o un lugar movido: [PR #213](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/213) (Marcia-44).

Las fuentes marcadas **verificada** se abrieron o descargaron. Las marcadas *sin verificar* no se pudieron abrir.

## Variables que piden los modelos de destino y modo, y estado en `data/mvp`

Los logit de destino y modo, ActivitySim, MATSim y ESTRAUS usan las mismas entradas: dónde vive y trabaja la gente, el tiempo y el costo por modo, el ingreso (que define el valor del tiempo), los autos del hogar, los atractores de ocio y compras, y quién no viaja.

| Variable | En `data/mvp` | Brecha | Mejor fuente |
| --- | --- | --- | --- |
| Hogar → trabajo (comuna) | `work_od.csv`, Censo 2024: 2.740.454 personas con destino conocido; el motor usa 2.688.814 RM → RM | Ninguna | Ya está |
| Modo al trabajo | `mode_share.csv`, `mode_od.csv`, Censo 2024 | El Metro va junto con el bus (código 2) | Ya está; EOD 2012 para separar |
| Empleo dentro de la comuna | `jobs.csv`, `jobs_2026.csv` | Es un modelo, no una medición | Ya está |
| Tiempos de viaje | `travel_times.csv`, `travel_times_blocks.csv` (GTFS) y `car_minutes.csv` (auto, por par de comunas) | Ninguna | Ya está |
| **Trabajo en casa (base de teletrabajo)** | [#221](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/221) trae la tabla por comuna | **517.018** trabajadores RM declaran como lugar de trabajo **habitual** su vivienda (P44 código 1). El denominador de viajes del motor ya los excluye P44 no distingue teletrabajo de otros oficios hechos en casa | Microdatos Censo 2024, el mismo `tools/scrape/censo2024-od.py`, contando `p44_lug_trab = 1` por comuna |
| Ingreso por residencia | `income.csv` (CASEN 2015, media por comuna) | Es de 2015 y trae estimación solo para 26 de 52 comunas. La planilla administrativa 2024 cubre las 52, pero mide otra cosa: ingreso imponible de asalariados formales por residencia probable ([ficha de Erwin-10](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/2b78604/context/research/behavior-data-joins.md)) | MDSF SAE 2022 por comuna (tasa de pobreza, no ingreso medio), **verificada**: https://observatorio.ministeriodesarrollosocial.gob.cl/pobreza-comunal-2022 |
| Salario en el destino | No hay | No hay fuente por lugar de trabajo | SII, Estadísticas de Empresas por Comuna y Rubro: trabajadores dependientes y renta neta, 2005–2025, **verificada**: https://www.sii.cl/sobre_el_sii/estadisticas_de_empresas.html. Solo como contexto de planillas por empleador: la comuna es la de la casa matriz, así que no sirve como salario del lugar de trabajo en una utilidad de destino |
| **Valor del tiempo** | No hay | El motor no convierte minutos en pesos | MDSF, Precios Sociales 2025, **verificado en el PDF**: urbano en vehículo **3.126**, espera y caminata **6.252**, viaje deportivo o recreativo **2.620** CLP/h-pasajero. Mezcla de propósitos 3,4 % trabajo / 27,2 % commute / 69,4 % otros (13 EOD de SECTRA). https://sni.gob.cl/wp-content/uploads/Precios_Sociales_2025.pdf |
| Autos del hogar | Hernán-10 (corte 5, `eod_car_mode.csv`, EOD 2012) | Es de 2012 | `vehicles.csv` no sirve como tenencia: el permiso se saca en otra comuna |
| Ocio y compras | `activity_points.csv` (16.697 POI OSM), `eod_destinations.csv`, `eod_trip_rates.csv` | No hay visitas por lugar | ENUT 2023 (INE, uso del tiempo, **verificada**): https://www.ine.gob.cl/enut. ENPCCL 2024 (participación cultural, con base de datos, **verificada**): https://www.cultura.gob.cl/participacioncultural/resultados-enpccl-2024/ |
| Asistencia a partidos | Cifras sueltas publicadas (p. ej. 39.222 espectadores el 31/08/2025, en #203) | No hay datos abiertos por partido | *sin verificar*: Estadio Seguro por Transparencia |
| Coeficientes logit de Santiago | No hay | Los de ESTRAUS / EOD 2012 no están publicados | *sin verificar*: pedir a SECTRA por Transparencia. Pista: https://www.dtpm.cl/descargas/licitacion21/Informe%20SECTRA_Licitacion.pdf |

## Qué conectar primero

1. **Teletrabajo:** P44=1 sale del mismo archivo censal que `work_od.csv`, pero es otro universo: la matriz OD lo excluye. No se puede restar de los viajes que ya existen. Para simular días adicionales en casa hay que fijar, como supuesto explícito, qué proporción de los viajes existentes deja de hacerse. El `stay_home` de Hernán-10 sigue esa línea; la tabla por comuna está en #221.
2. **Valor del tiempo**, de Precios Sociales 2025. Son valores para evaluar inversión pública. Para dar una cifra en pesos de un escenario hacen falta las diferencias de pasajeros-hora por propósito y por componente del tiempo (en vehículo, espera, caminata); un cambio en las subidas a estaciones no basta.


Límites: SAE 2022 da pobreza, no ingreso. ENUT y ENPCCL son regionales. La asistencia a partidos y los coeficientes de ESTRAUS no son datos abiertos.
