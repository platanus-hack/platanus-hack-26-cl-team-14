# santiaguito.cl

Una torre nueva cambia el barrio. También puede cambiar los viajes al trabajo en comunas donde nadie construyó.

Pregunta en el chat qué pasaría si una comuna tuviera más residentes y otra más empleos. El agente traduce la pregunta a un escenario y llama al simulador en Rust. El motor estima cómo cambian los viajes habituales al trabajo. La maqueta nativa de Santiago muestra los cambios del mismo escenario.

El motor aplica cambios de residentes, empleos y tiempos de viaje. También puede cerrar o reabrir estaciones de Metro existentes. Para eventos, estima la demanda de los viajes de retorno por separado. Los resultados son estimaciones bajo los supuestos de cada escenario.

Usamos el Censo 2024 del INE para los flujos habituales al trabajo. Comparamos las estimaciones de subidas al Metro con datos del DTPM de abril de 2026. Los horarios y recorridos vienen de GTFS. La maqueta usa datos de [© colaboradores de OpenStreetMap](https://www.openstreetmap.org/copyright).

El motor, el chat y la maqueta corren en local. La [guía de la demo](context/research/handoff-humanos.md) indica cómo conectarlos. Puedes [descargar santiaguito.cl v1.0.1](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/releases/tag/v1.0.1) como AppImage para Linux x86_64. Para macOS con Apple Silicon, está disponible el [ZIP de la versión anterior v0.1.0](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/releases/download/v0.1.0/santiaguito-macos-arm64.zip).

## Equipo

Francisco Mackenney, Juan Vargas, Vicente Matus, Bruno Farfán Miquel y Emerson Benjamin Salazar Rubilar.
