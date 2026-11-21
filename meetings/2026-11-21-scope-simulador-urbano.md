# Scope del simulador urbano de Santiago (2026-11-21)

Summary only. The raw transcript is not in git (`meetings/raw/` is ignored).

Summary

### Elementos pendientes de decisión

- [ ]  Definir cómo los usuarios interactúan con la simulación: ¿como un agente con lenguaje natural que tiene la simulación como tool, o como una simulación con una API/MCP para que otros agentes la consulten?
- [ ]  Redactar una primera definición clara del proyecto en el white paper para el equipo que se irá incorporando

### Objetivo de la reunión

- Sesión de brainstorming para definir los parámetros del simulador urbano de Santiago
- Se llegó a consenso sobre el foco: el simulador debe ser un **engine generalizado de dinámicas urbanas**, no una colección de simulaciones específicas

### Qué SÍ debe poder simular (in scope)

- Cualquier pregunta relacionada con **dinámicas sociales en la ciudad**: transporte, vivienda, trabajo, entretención, actividad cultural
- Ejemplo central discutido: *¿Qué pasa si saco el MUT y lo reemplazo por un gueto vertical?* → el engine debe entender quiénes trabajaban ahí, quiénes venían a entretenerse, hacia dónde se redistribuirían
- Otro caso de uso: simular el impacto de colocar un estadio (ej. Azul Azul) en distintos barrios para apoyar una decisión real
- El output es **data estructurada** que luego puede alimentar análisis o decisiones

### Qué NO debe poder simular (out of scope)

- Simulaciones físicas: terremotos, viento, incendios, etc. — incluirlas convertiría el engine en un "collage de simulaciones"
- Comportamiento individual de personas (psicología individual); el modelo opera a **nivel de masa**, que es más predecible
- Visualización de eventos destructivos (ej. "¿cómo se ve si exploto el MUT?")

### Modelo de agentes y escala

- La ambición inicial es simular **todo Santiago con cada persona** (~7 millones de agentes), potencialmente acotable a una o varias comunas
- Cada persona tiene parámetros (dónde vive, trabaja, se entretiene) que permiten modelar redistribuciones ante cambios
- El comportamiento humano a nivel de masa es más predecible que a nivel individual, lo que hace viable el modelo
- El metro ya usa modelos similares para predecir flujos de personas entre estaciones
- **Desafío identificado**: simular solo una comuna tiene problemas en los bordes, ya que cambios justo fuera del área simulada pueden impactar dentro y no quedarían capturados

### El modelo 3D como visualizador

- El modelo 3D **es un UI/visualizador**, no la simulación en sí
- No importa la física de las construcciones (vigas, cimientos); lo que importa son las unidades habitacionales, cuántas personas viven ahí y qué dinámicas generan
- Se puede hacer zoom desde toda la ciudad hasta ver una persona caminando

### Estrategia de pitch

- Mostrar **tres casos de uso muy distintos** para demostrar que el engine puede responder cualquier pregunta dentro del scope de dinámicas urbanas
- El diferenciador clave frente a otras herramientas de simulación: no es una colección de simulaciones, sino un engine generalizado
- El valor real está en que **genere decisiones concretas**, no solo visualizaciones atractivas

