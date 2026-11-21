# Para los humanos: qué hay, cómo verlo, qué deciden

Propuesta de los agentes; `context/pitch.md` sigue siendo de ustedes. Fuentes: `context/research/pitch-proposal.md`.

## Qué funciona hoy (en `main`)

- **Un motor en Rust** con datos reales: Censo 2024 (7.400.741 personas; 2.740.454 con lugar de trabajo habitual conocido, sin las 517.018 que trabajan en casa ni las 313.358 sin lugar fijo), subidas al Metro de DTPM (abril 2026), permisos del INE y la encuesta EOD 2012 para ubicar los empleos por comuna.
- **Cuatro vistas en el navegador**, para proyector:
  1. **Tobalaba**: edificios reales y la estación con sus 48 medias horas (37 con dato, 11 sin dato; 7.516 a las 18:00). Un control: sacar el MUT (4.679 empleos modelados) y poner una torre de N viviendas. Al correr, dos estimaciones etiquetadas (−513 y −2.781 subidas al día con 298 viviendas, ver tabla) y "84 personas salen en auto al día · 52 en transporte público".
  2. **Santiago**: 52 comunas y flujos casa→trabajo; control de empleo y vivienda por comuna, totales conservados. Etiqueta: "reconstrucción del Censo 2024 + sensibilidad, sin validar".
  3. **Puntaje**: 31 estaciones reservadas, columnas observadas y predichas, error agregado 45% (L4b). Puntaje de desarrollo; no valida el cambio del MUT.
  4. **Evento**: dos botones (Estadio Nacional, Monumental) y un control de 0 a 3. Añade, como supuesto, el patrón de subidas de la noche del sábado 11 de abril de 2026 en Parque O'Higgins (día de un evento agotado en el Movistar Arena) a las estaciones a menos de 1 km del sitio; con 0 no cambia nada. Límites: solo 19:00–23:30, la llegada no se modela; dos supuestos en pantalla; sin validar.
- **No construido**: la Variante B (Estación Central 2017→2024).

## Cómo verlo (requiere Rust y Python 3; una primera compilación medida tomó 75 s)

```
git clone https://github.com/platanus-hack/platanus-hack-26-cl-team-14.git
cd platanus-hack-26-cl-team-14 && ./run.sh      # deja esta terminal ocupada
```

En el navegador: `http://127.0.0.1:5173/web/`. En otra terminal de macOS: `open http://127.0.0.1:5173/web/`.

Vistas: `/web/?t=36&mut=1`, `/web/?v=s`, `/web/?v=p&t=36`, `/web/?v=e`. Ctrl-C en la primera terminal apaga ambos servidores (`lsof -i :8787` debe quedar vacío).

## Los números de referencia de hoy (la versión congelada se elige a las 12:30 UTC)

A las 12:30 UTC (09:30 en Santiago) los agentes eligen **una versión revisada** (commit + artefacto de puntaje; regla en `context/decisions/20261121-0845-presentation-freeze.md`) y el demo corre esa versión, no `main`. Una **capa** es una versión del modelo de estaciones; se queda solo si baja el error en las 31 estaciones reservadas. Hoy la capa aceptada es **L4b**; L5 y L8 empeoraron el puntaje y L6 se retiró sin correr, así que L4b es la candidata:

| En pantalla | Valor | Qué es |
| --- | --- | --- |
| 7.516 | subidas a las 18:00 en Tobalaba, día laboral promedio de abril 2026 | observado, DTPM |
| −513 | cambio de subidas al día al cambiar el MUT por 298 viviendas | contabilidad del sitio, supuestos visibles |
| −2.781 | mismo cambio según el modelo de estaciones L4b, en Tobalaba | sensibilidad, sin validar; su base es 51% de lo observado |
| 84 / 52 | nuevos residentes que salen en auto / en transporte público al día (flujo diario, después del supuesto de asistencia 0,8; stock 105 / 65; tasa de ocupados 0,3577 de `site_inputs.csv`, viviendas particulares) | Censo 2024 Las Condes, supuesto de transferencia |
| 45% | error agregado del modelo en 31 estaciones reservadas | puntaje de desarrollo |
| 0,1 ms | tiempo del cálculo por comuna, medido en el portátil de Emerson (no es el tiempo del navegador ni del modelo de estaciones) | medido |

Nada cambia después de las 12:30 hasta que ustedes lo digan.

## Lo que no sabemos (una línea cada uno)

- Cuántos trabajadores del MUT entran al Metro: el 0,2 es un supuesto; el modelo de estaciones usa otra vía. −513 y −2.781 son dos estimaciones, no un rango.
- El Censo no separa Metro de bus: digan "transporte público", nunca "Metro".
- Los 4.679 empleos del MUT son modelados (superficie publicada de oficinas y locales, 88% y 75% arrendada, 10 y 46 m² por puesto); el operador declara 4.000 en oficinas.
- Datos de 2023 (PR #77 a #79): entre abril y agosto, Tobalaba en Línea 4 subió 20,4% y la mediana de las otras 18 estaciones bajó 1,5%. Dos semanas, por línea, conteo no documentado. Descriptivo: no mide el efecto del MUT ni se compara con 2026 ni con los modelos.
- Los dos modelos mueven residentes y trabajadores; no incorporamos una fuente verificada de visitantes al modelo.
- Tobalaba no está entre las 31 estaciones reservadas: el 45% nunca la midió.

## Las 5 preguntas para ustedes (con nuestra recomendación)

1. **Comuna y palabras**: "Las Condes, al lado de Tobalaba"; "transporte público" para el Censo, "Metro" solo para DTPM.
2. **La sorpresa del MUT**: ¿las dos estimaciones etiquetadas, el cambio pequeño en la estación y los residentes que salen en auto? Recomendación: todo visible, incluida la brecha de la base.
3. **Casos 2 y 3**: Estación Central (57 torres de 20 pisos o más, 477 viviendas en promedio; corre hoy en la vista Santiago) como caso 2. El caso evento está construido y revisado (vista Evento). Recomendación: tercera pregunta, con sus dos supuestos dichos en voz alta: "qué pasaría si ese patrón se repite".
4. **Los datos de 2023**: ¿dos frases de contexto ("no sabemos cuánto es del MUT; nuestros modelos no cuentan visitantes") o se dejan para las preguntas del jurado? Recomendación: para las preguntas.
5. **Tiempo y cierre**: 256 palabras (~2 minutos a 130 por minuto, antes de clics y del jurado). Con el evento construido, el cierre puede decir "tres preguntas"; si el ensayo pasa de 3 minutos, el evento se corta primero y el cierre vuelve a "dos preguntas, y la tercera la hacen ustedes". Si hay que cortar: primero el contexto 2023, después la línea del modelo de estaciones; nunca la apertura ni el número del jurado.

Respondan en `#agent-chat` o editen `context/pitch.md`.
