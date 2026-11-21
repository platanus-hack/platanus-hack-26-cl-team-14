# Para los humanos: qué hay, cómo verlo, qué deciden

Propuesta de los agentes (keen-koala, 10:05 UTC); `context/pitch.md` sigue siendo de ustedes. Fuentes de cada número: `context/research/pitch-proposal.md`.

## Qué funciona hoy (en `main`)

- **Un motor en Rust** con datos reales: Censo 2024 (7.400.741 personas; 2.740.454 personas con lugar de trabajo habitual conocido, sin las 517.018 que trabajan en casa ni las 313.358 sin lugar fijo), subidas al Metro de DTPM (abril 2026), permisos de edificación del INE y la encuesta EOD 2012 para ubicar los empleos dentro de cada comuna.
- **Tres vistas en el navegador**, para proyector:
  1. **Tobalaba**: edificios reales y la estación con sus 48 medias horas (37 con dato, 11 sin dato; 7.516 a las 18:00). Un control: sacar el MUT (4.679 empleos modelados) y poner una torre de N viviendas. Al correr, dos estimaciones etiquetadas (−513 y −2.781 subidas al día con 298 viviendas, ver tabla) y "83 personas salen en auto al día · 51 en transporte público".
  2. **Santiago**: las 52 comunas y los flujos casa→trabajo; un control de empleo y vivienda por comuna, totales conservados. Etiqueta: "reconstrucción del Censo 2024 + sensibilidad, sin validar".
  3. **Puntaje**: 31 estaciones de Metro reservadas, columnas observadas y predichas, error agregado 45% (L4b). Puntaje de desarrollo; no valida el cambio del MUT.
- **Lo que no funciona**: el estadio (caso 3 de la propuesta) no está construido. La Variante B (Estación Central 2017→2024) tampoco.

## Cómo verlo (requiere Rust y Python 3; una primera compilación medida tomó 75 s)

```
git clone https://github.com/platanus-hack/platanus-hack-26-cl-team-14.git
cd platanus-hack-26-cl-team-14 && ./run.sh      # deja esta terminal ocupada
```

En otra terminal, o en el navegador: `open http://127.0.0.1:5173/web/` (macOS).

Vistas directas: `/web/?t=36&mut=1` (Tobalaba), `/web/?v=s` (Santiago), `/web/?v=p&t=36` (Puntaje). Ctrl-C en la primera terminal apaga los dos servidores; comprueben con `lsof -i :8787`.

## Los números de referencia de hoy (la versión congelada se elige a las 12:30 UTC)

A las 12:30 UTC (09:30 en Santiago) los agentes eligen **una versión revisada** (commit de `main` + artefacto de puntaje; regla en `context/decisions/20261121-0845-presentation-freeze.md`; una capa aún en revisión a las 12:15 queda fuera) y el demo corre esa versión, no `main`. Una **capa** es una versión del modelo de estaciones: cada una agrega un mecanismo y se queda solo si baja el error en las 31 estaciones reservadas. Hoy la capa aceptada es **L4b**; L5 y L8 empeoraron el puntaje y L6 se retiró sin correr, así que L4b es la candidata:

| En pantalla | Valor | Qué es |
| --- | --- | --- |
| 7.516 | subidas a las 18:00 en Tobalaba, día laboral promedio de abril 2026 | observado, DTPM |
| −513 | cambio de subidas al día al cambiar el MUT por 298 viviendas | contabilidad del sitio, supuestos visibles |
| −2.781 | mismo cambio según el modelo de estaciones L4b, en Tobalaba | sensibilidad, sin validar; su base es 51% de lo observado |
| 83 / 51 | nuevos residentes que salen en auto / en transporte público al día (flujo diario, después del supuesto de asistencia 0,8; stock 104 / 64) | Censo 2024 Las Condes, supuesto de transferencia |
| 45% | error agregado del modelo en 31 estaciones reservadas | puntaje de desarrollo |
| 0,1 ms | tiempo del cálculo por comuna, medido en el portátil de Emerson (no es el tiempo del navegador ni del modelo de estaciones) | medido |

Cada número hablado lleva la capa de la versión elegida; nada cambia después de las 12:30 hasta que ustedes lo digan.

## Lo que no sabemos (una línea cada uno)

- Cuántos trabajadores del MUT entran al Metro: el 0,2 es un supuesto; el modelo de estaciones usa otra vía. −513 y −2.781 son dos estimaciones, no un rango.
- El Censo no separa Metro de bus: digan "transporte público", nunca "Metro", para cifras del Censo.
- Los 4.679 empleos del MUT son modelados (superficie publicada de oficinas y locales, 88% y 75% arrendada en 2024, 10 y 46 m² por puesto); el operador declara 4.000 personas en oficinas.
- Datos de 2023 alrededor de la apertura del MUT (PR #77 a #79): entre abril y agosto, Tobalaba en Línea 4 subió 20,4% y la mediana de las otras 18 estaciones bajó 1,5%. Dos semanas, por línea, conteo de combinaciones no documentado. Descriptivo: no mide el efecto del MUT ni se compara con 2026 ni con los modelos.
- Los dos modelos mueven residentes y trabajadores; no incorporamos una fuente verificada de visitantes al modelo.
- Tobalaba es una estación de entrenamiento, no una de las 31 reservadas: el 45% nunca midió Tobalaba.

## Las 5 preguntas para ustedes (con nuestra recomendación)

1. **Comuna y palabras**: decir "Las Condes, al lado de Tobalaba"; "transporte público" para el Censo, "Metro" solo para subidas DTPM.
2. **La sorpresa del MUT**: ¿las dos estimaciones con sus etiquetas, el cambio pequeño en la estación y los residentes que salen en auto? Recomendación: todo visible, incluida la brecha de la base; un límite visible vale más que un resultado escondido.
3. **Casos 2 y 3**: Estación Central (57 torres de 20 pisos o más, 477 viviendas en promedio; corre hoy en la vista Santiago) como caso 2. El estadio no está construido. Recomendación: cortarlo del demo y nombrarlo solo como "la próxima pregunta" en el cierre; si los agentes encuentran una firma observada de evento antes de las 12:15 UTC, ustedes deciden si entra.
4. **Los datos de 2023**: ¿dos frases de contexto ("no sabemos cuánto es del MUT; nuestros modelos no cuentan visitantes") o se dejan para las preguntas del jurado? Recomendación: para las preguntas, si el ensayo pasa de 3 minutos.
5. **Tiempo y cierre**: 256 palabras (~2 minutos a 130 por minuto, antes de clics y de la respuesta del jurado). El cierre dice "tres preguntas" con dos casos: recomendación, "dos preguntas, y la tercera la hacen ustedes". Si hay que cortar: primero el contexto 2023, después la línea del modelo de estaciones; nunca la apertura ni el número del jurado.

Respondan en `#agent-chat` o editen `context/pitch.md`.
