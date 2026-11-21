# ¿Y si Santiago…?

Una torre nueva cambia el barrio. También puede cambiar los viajes al trabajo en comunas donde nadie construyó.

Pregunta en el chat qué pasaría si una comuna tuviera más residentes y otra más empleos. El agente traduce la pregunta a un escenario y llama al simulador en Rust. El motor estima cómo cambian los viajes habituales al trabajo. La maqueta nativa de Santiago, en preparación, muestra los cambios del mismo escenario.

El motor aplica cambios de residentes, empleos y tiempos de viaje. También puede cerrar o reabrir estaciones de Metro existentes. Para eventos, estima la demanda de los viajes de retorno por separado. Los resultados son estimaciones bajo los supuestos de cada escenario.

Usamos el Censo 2024 del INE para los flujos habituales al trabajo. Comparamos las estimaciones de subidas al Metro con datos del DTPM de abril de 2026. Los horarios y recorridos vienen de GTFS. La maqueta usa datos de [© colaboradores de OpenStreetMap](https://www.openstreetmap.org/copyright).

El motor y el chat corren en local. El [repositorio](https://github.com/platanus-hack/platanus-hack-26-cl-team-14#run-a-scenario-from-your-local-agent) documenta cómo conectarlos. La descarga pública de la aplicación queda pendiente.

## Equipo

Francisco Mackenney, Juan Vargas, Vicente Matus, Bruno Farfán Miquel y Emerson Benjamin Salazar Rubilar.
