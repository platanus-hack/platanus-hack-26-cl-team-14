# Datos para modelos de comportamiento: qué hay y qué falta

Pedido de Mac (16:27 UTC): que los what-if (mover un lugar, un modo nuevo, más teletrabajo) usen modelos existentes y todos los datos necesarios. Este archivo cubre solo el **eje de datos**. Cómo meten los modelos un modo nuevo o un lugar movido: [PR #213](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/213) (Marcia-44).

Las fuentes marcadas **verificada** se abrieron o descargaron. Las marcadas *sin verificar* no se pudieron abrir.

## Variables que piden los modelos de destino y modo, y estado en `data/mvp`

Los logit de destino y modo, ActivitySim, MATSim y ESTRAUS usan las mismas entradas: dónde vive y trabaja la gente, el tiempo y el costo por modo, el ingreso (que define el valor del tiempo), los autos del hogar, los atractores de ocio y compras, y quién no viaja.

| Variable | En `data/mvp` | Brecha | Mejor fuente |
| --- | --- | --- | --- |
| Hogar → trabajo (comuna) | `work_od.csv`, Censo 2024: 2.740.454 personas | Ninguna | Ya está |
| Modo al trabajo | `mode_share.csv`, `mode_od.csv`, Censo 2024 | El Metro va junto con el bus (código 2) | Ya está; EOD 2012 para separar |
| Empleo dentro de la comuna | `jobs.csv`, `jobs_2026.csv` | Es un modelo, no una medición | Ya está |
| Tiempos de viaje | `travel_times.csv`, `travel_times_blocks.csv` (GTFS) | Solo transporte público | Ya está |
| **Trabajo en casa (base de teletrabajo)** | No hay archivo | **517.018** trabajadores RM responden «trabaja en su vivienda» (P44 código 1). El conteo está en `data/mvp/README.md`, pero no hay tabla por comuna | Microdatos Censo 2024, el mismo `tools/scrape/censo2024-od.py`, contando `p44_lug_trab = 1` por comuna |
| Ingreso por residencia | `income.csv` (CASEN 2015, media por comuna) | Es de 2015 | MDSF SAE 2022 por comuna (tasa de pobreza, no ingreso medio), **verificada**: https://observatorio.ministeriodesarrollosocial.gob.cl/pobreza-comunal-2022 |
| **Salario en el destino** | No hay | Ninguno | SII, Estadísticas de Empresas por Comuna y Rubro: trabajadores dependientes y renta neta, 2005–2025, **verificada**: https://www.sii.cl/sobre_el_sii/estadisticas_de_empresas.html. Sesgo: la comuna es la de la casa matriz |
| **Valor del tiempo** | No hay | El motor no convierte minutos en pesos | MDSF, Precios Sociales 2025, **verificado en el PDF**: urbano en vehículo **3.126**, espera y caminata **6.252**, viaje deportivo o recreativo **2.620** CLP/h-pasajero. Mezcla de propósitos 3,4 % trabajo / 27,2 % commute / 69,4 % otros (13 EOD de SECTRA). https://sni.gob.cl/wp-content/uploads/Precios_Sociales_2025.pdf |
| Autos del hogar | Hernán-10 (corte 5, `eod_car_mode.csv`, EOD 2012) | Es de 2012 | `vehicles.csv` no sirve como tenencia: el permiso se saca en otra comuna |
| Ocio y compras | `activity_points.csv` (16.697 POI OSM), `eod_destinations.csv`, `eod_trip_rates.csv` | No hay visitas por lugar | ENUT 2023 (INE, uso del tiempo, **verificada**): https://www.ine.gob.cl/enut. ENPCCL 2024 (participación cultural, con base de datos, **verificada**): https://www.cultura.gob.cl/participacioncultural/resultados-enpccl-2024/ |
| Asistencia a partidos | Lo que publicó Hernán-10 en el caso 1 (prensa, Memoria Blanco y Negro) | No hay datos abiertos por partido | *sin verificar*: Estadio Seguro por Transparencia |
| Coeficientes logit de Santiago | No hay | Los de ESTRAUS / EOD 2012 no están publicados | *sin verificar*: pedir a SECTRA por Transparencia. Pista: https://www.dtpm.cl/descargas/licitacion21/Informe%20SECTRA_Licitacion.pdf |

## Qué conectar primero

1. **Teletrabajo base por comuna**, con el código 1 de P44 del Censo 2024. Hernán-10 propone `kind:"stay_home"` y midió 21,7 % de trabajadores sin viaje al trabajo en la EOD 2012. El Censo da la base de 2024 por comuna con el mismo universo que `work_od.csv`. El dato ya se descarga: solo hay que contar otra respuesta.
2. **Valor del tiempo**, de Precios Sociales 2025. Convierte en pesos los minutos que gana o pierde un escenario. Por ejemplo: «el intercambio MUT ↔ Monumental cuesta X millones de pesos al día en tiempo». No cambia ningún resultado del motor; solo agrega una salida.
3. **Salario en el destino**, de SII por comuna y rubro. Sirve de atractor de destino cuando se mueven empleos. Hay que declarar el sesgo de la casa matriz.

Límites: SAE 2022 da pobreza, no ingreso. ENUT y ENPCCL son regionales. La asistencia a partidos y los coeficientes de ESTRAUS no son datos abiertos.
