# ¿Y si Santiago…?

Una torre nueva cambia el barrio. También puede cambiar los viajes al trabajo en comunas donde nadie construyó.

Pregunta en el chat qué pasaría si una comuna tuviera más viviendas y otra más empleos. El agente convierte la pregunta en cambios y llama al simulador en Rust. El mapa muestra cómo el modelo redistribuye los viajes habituales entre casa y trabajo.

Usamos el Censo 2024 del INE para los flujos habituales al trabajo. Los datos del DTPM de abril de 2026 sirven de referencia para las subidas al Metro. Los horarios y recorridos vienen de GTFS. El mapa usa datos de [© colaboradores de OpenStreetMap](https://www.openstreetmap.org/copyright).

La versión actual responde a cambios de viviendas y empleos por comuna. Para esa consulta, el modelo aún no calcula congestión vial, capacidad de trenes ni efectos de eventos. Los resultados estiman qué ocurriría bajo los supuestos del escenario.

El proyecto corre en local. El [repositorio](https://github.com/platanus-hack/platanus-hack-26-cl-team-14) incluye los pasos para abrir la ciudad y conectar el chat mediante MCP.

## Equipo

Francisco Mackenney, Juan Vargas, Vicente Matus, Bruno Farfán Miquel y Emerson Benjamin Salazar Rubilar.
