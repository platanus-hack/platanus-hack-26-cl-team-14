# Santiago en vivo

Guion de tres minutos para la simulación nativa. El intercambio MUT ↔ Monumental es el caso principal. El Mapocho es el segundo caso. Mac define el tercero. [Dirección del pitch](https://discord.com/channels/1573514576400752640/1573688250797658172/1573733563491291157).

| Tiempo | Qué se ve | Qué se dice |
| --- | --- | --- |
| 0:00–0:20 | En la maqueta, el MUT junto al Metro Tobalaba y el Monumental en Macul, en sus ubicaciones actuales. | «Para averiguar qué pasaría si cambiáramos estos lugares, tendríamos que mover dos edificios. Aquí podemos empezar con una pregunta». |
| 0:20–0:50 | La pregunta entra en la terminal. Quedan visibles el caso de referencia y la maqueta. | «¿Y si el MUT se fuera a Macul y Colo-Colo jugara en Tobalaba?» |
| 0:50–1:50 | Mostramos ambos lugares antes y después. Miramos quién llega, en qué modo y desde dónde para las partes que entregue el motor. | «Miremos primero qué cambia para quienes van al trabajo. Después, el mismo partido en otro lugar». Lee una diferencia que se vea en la maqueta y la unidad de la misma ejecución. |
| 1:50–2:15 | Volvemos a ver los dos lugares. La terminal conserva la comparación del partido. | «Cuando movemos un lugar, cambian viajes que empiezan lejos de aquí». Si mostramos los regresos de un partido, leemos también la asistencia supuesta. |
| 2:15–2:40 | En vivo, un corredor fluvial supuesto y sus conexiones con la red. La toma y el cálculo siguen pendientes. | «Si habilitáramos un tramo del Mapocho para transporte público, ¿quién elegiría usarlo en un día en que pudiera operar?» |
| 2:40–3:00 | Reservamos este tramo para el tercer caso que defina Mac y el cierre. | «Antes de cambiar una ciudad, podemos preguntar quién tendría que cambiar su viaje». |

## Pregunta preparada

> Si intercambiamos el MUT y el Estadio Monumental, ¿qué cambia en los viajes al trabajo y en los viajes de un mismo partido? Compara los dos lugares con el escenario actual. Usa la misma asistencia supuesta y la misma hora de término para comparar el partido. Resume el cambio con las unidades del motor.

## Qué sabemos antes del ensayo

- La primera corrida por consola movió los empleos modelados entre las dos manzanas y comparó el mismo partido en ambos puntos. Es evidencia del motor. Los resultados sirven para preparar el ensayo en el chat y la ventana. La manzana del Monumental incluye empleos que el modelo atribuye a la zona. Para atribuirlos al estadio hace falta una fuente del recinto. [Corrida inicial](https://discord.com/channels/1573514576400752640/1573561159955648553/1573734326825254925), [lugares del modelo](../../data/mvp/named_places.csv), [punto del Monumental](extra-event-contract.md).
- El motor actual estima el día laboral y el regreso del partido por separado. La estimación de regresos usa a quienes trabajan en la comuna del recinto como aproximación. Si el recinto cambia, también cambia esa referencia. Falta fijar un perfil de asistentes comparable entre los dos lugares y calcular sus llegadas. [Modelo del evento](../../engine/src/attendance.rs).
- Los grupos de viajes de la maqueta son representativos. Para dar cifras por recinto, origen y modo, agrega a las personas antes de agrupar los viajes para dibujarlos. Falta comprobar que el chat y las dos tomas nativas muestran la misma revisión del intercambio. [Grupos de viajes](../../engine/src/population/trips.rs), [contrato del motor](../../engine/README.md#general-scenario-http-routes).
- Ensaya también preguntas que no mencionen estos recintos. Usa un cambio de residentes y un cierre de estación existente. Incluye una pregunta sobre más teletrabajo como prueba de alcance. Hasta que el motor calcule esa decisión, explica el límite en el chat. El tercer caso sigue por definir. [Dirección sobre alcance](https://discord.com/channels/1573514576400752640/1573688250797658172/1573733877363773481).

## Cámaras para el ensayo

- **MUT, Tobalaba.** Reutiliza el encuadre abierto de la captura nativa del MUT. El objetivo es −70,600974 / −33,417357, a 780 m. Busca el mismo ángulo antes y después. [Captura de referencia](https://discord.com/channels/1573514576400752640/1573688250797658172/1573717621927579730).
- **Estadio Monumental, Macul.** Usa el centro del recinto como objetivo y parte a la misma escala que el MUT. El encuadre sigue pendiente de captura. [Punto auditado](extra-event-contract.md).
- **Mapocho.** Propongo el tramo entre Puente Pedro de Valdivia y Estación Mapocho, con una toma cercana en Pío Nono. Falta acordar los muelles y capturar el plano en el visor.

## Mapocho

El caso pregunta por el uso de un servicio supuesto en un tramo habilitado. Compara los mismos orígenes y destinos con el servicio abierto y cerrado. Declara los muelles, la caminata, la espera, el trayecto y las combinaciones. Usa la encuesta de viajes y la oferta programada de Santiago como base. Las fechas de operación requieren calado, niveles, velocidades, puentes y reglas de cierre del tramo. Las estadísticas de caudal sirven como insumo. El aforo aguas arriba por sí solo no da una fecha de apertura en el centro. El motor estima entradas al Metro si se le agregan puntos como estaciones. Para contar pasajeros fluviales hace falta que el motor asigne viajes al servicio. El cálculo y una prueba de la ventana siguen pendientes. [DGA](https://dga.mop.gob.cl/estadisticas-estaciones-dga/), [DOH](https://repositoriodirplan.mop.gob.cl/biblioteca/items/ea7b00f1-d782-42f0-a43f-5a9d098d4ac9/full), [encuesta](https://www.subtrans.gob.cl/mtt-presenta-resultados-de-la-encuesta-origen-destino-de-santiago/), [GTFS de Santiago](https://www.dtpm.cl/descargas/gtfs/Descripcion%20GTFS%20estatico%20DTPM_20260210.pdf).

El antecedente local importa para el pitch. El proyecto original descartó navegar en el lecho por seguridad y el Parque La Familia construyó una laguna lateral. El Paseo Fluvial anunciado en 2026 es un paseo inundable. Valdivia muestra cómo los recorridos fluviales pueden complementar servicios terrestres. Londres publica muelles, horarios y conexiones. Sirven como referencias de operación. [MINVU, Parque La Familia](https://www.minvu.gob.cl/wp-content/uploads/2020/05/Libro-Parques-Urbanos.pdf#page=51), [Gobierno de Santiago](https://www.gobiernosantiago.cl/2026/07/13/gobierno-de-santiago-inicia-licitacion-del-proyecto-paseo-fluvial-rio-mapocho-la-iniciativa-contempla-mas-de-22-mil-millones-de-inversion/), [Subsecretaría de Transportes, Valdivia](https://www.subtrans.gob.cl/subsecretario-de-transportes-realizo-balance-del-transporte-publico-fluvial-en-valdivia-mas-de-8-mil-personas-ya-lo-han-utilizado/), [TfL](https://tfl.gov.uk/help-with-travel/ways-to-travel/river).
