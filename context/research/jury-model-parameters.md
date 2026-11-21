# Qué usa el simulador de Santiago

Texto para una lámina extra del jurado. [Pedido de Mac](https://discord.com/channels/1573514576400752640/1573815827617288294/1573847113094275093)

| Datos de Santiago | Cómo estima los viajes | Qué cambia en la demo |
| --- | --- | --- |
| Censo 2024: hogares, empleo y viaje al trabajo. | Distribuye trabajadores según empleos y tiempos de viaje. | **Monumental:** cambia el recinto y los empleos de dos manzanas. Mantiene 39.222 asistentes. |
| MINEDUC 2026: matrícula y destinos de estudio. | Estima el modo según tiempo, distancia y acceso a auto. | **Mapocho:** agrega 10 muelles. Supone 17 km/h, un ferry cada 10 min y 1 min por parada. |
| DTPM: horarios, rutas y validaciones. OpenStreetMap: calles y edificios. | Calcula una base y una variante. Guarda los resultados para reproducir el antes y el después. | **Metro:** suprime el servicio de 07:00 a 10:00. Mantiene los lugares de trabajo y estima cambios de modo. |

## Notas para responder al jurado

- La base de la Región Metropolitana contiene 2.688.814 registros sintéticos de trabajo y 1.277.336 de estudio. Una persona puede tener ambos roles. La encuesta EOD 2012 aporta la relación entre acceso a auto y modo de viaje. [Datos y cobertura](../../data/mvp/README.md) · [Escala y medición](v1-engine-scale.md) · [Acceso a auto](../../data/mvp/eod_car_mode.md)
- Más empleos aumentan el peso de un destino. Un viaje más largo reduce ese peso. El motor ajusta las probabilidades de modo cuando cambia la red y agrupa resultados en medias horas. [Reglas de destino y modo](../../engine/src/choice.rs) · [Datos del Metro](v1-fixed-score.md)
- En el MUT, la variante retira 28 edificios, cierra seis vías y fija 53 y 4.679 empleos en las manzanas de Tobalaba y Macul. Las 17:00 son la hora supuesta de término del partido. El modelo usa el viaje laboral para estimar el modo de llegada de la hinchada. [Comparación del MUT](v3-mut-replay.md)
- El ferry supone 162 plazas y guarda una ruta que sigue el río. El gráfico presenta viajes al trabajo estimados por día hábil. Las personas de la animación ilustran ese flujo. [Parámetros del ferry](mapocho/boat_line.json) · [Unidad y recorrido](mapocho/chart-capture.md)
- El cierre del Metro es una variante de un día hábil. La vista empieza a las 08:00. El gráfico cuenta salidas al trabajo de 07:00 a 10:00. La caché final depende de la comprobación al reabrir. [Cierre y alcance](metro-closure-replay.md) · [Autoría del replay](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/451)
