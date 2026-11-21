# Para los humanos: qué hay, cómo verlo, qué deciden

Escrito por los agentes de la noche (keen-koala, 09:55 UTC). Es una propuesta: `context/pitch.md` sigue siendo de ustedes. El detalle y las fuentes de cada número están en `context/research/pitch-proposal.md`.

## Qué funciona hoy (en `main`)

- **Un motor en Rust** con datos reales: Censo 2024 (7.400.741 personas, 2.740.454 viajes al trabajo con comuna conocida), subidas al Metro de DTPM (abril 2026) y permisos de edificación del INE.
- **Tres vistas en el navegador**, para proyector:
  1. **Tobalaba**: edificios reales, la estación con sus 48 medias horas observadas (7.516 a las 18:00). Un control: sacar el MUT (4.679 empleos modelados) y poner una torre de N viviendas. Al correr, dos estimaciones con sus etiquetas: contabilidad del sitio (−513 subidas al día con 298 viviendas) y sensibilidad del modelo de estaciones (−2.781 con la capa L4b, base del modelo 41.702 frente a 81.613 observadas, regla de reparto visible). Debajo: "83 personas salen en auto al día · 51 en transporte público" (nuevos residentes, supuesto de transferencia).
  2. **Santiago**: las 52 comunas, flujos casa→trabajo. Un control de empleo y vivienda por comuna; el motor redistribuye y conserva los totales. Etiqueta: "reconstrucción del Censo 2024 + sensibilidad, sin validar".
  3. **Puntaje**: 31 estaciones de Metro que el modelo nunca vio, columnas observadas y predichas, y el error agregado (45% con L4b). Es un puntaje de desarrollo; no valida el cambio del MUT.
- **Lo que no funciona**: el estadio (caso 3 de la propuesta) no está construido. La Variante B (Estación Central 2017→2024) tampoco.

## Cómo verlo (3 comandos, primera compilación ~1 minuto)

```
git clone https://github.com/platanus-hack/platanus-hack-26-cl-team-14.git
cd platanus-hack-26-cl-team-14 && ./run.sh
open http://127.0.0.1:5173/web/
```

Vistas directas: `/web/?t=36&mut=1` (Tobalaba con el MUT reemplazado), `/web/?v=s` (Santiago), `/web/?v=p&t=36` (Puntaje). Ctrl-C en la terminal apaga los dos servidores; comprueben con `lsof -i :8787`.

## Los números congelados

A las 12:30 UTC (09:30 en Santiago) los agentes congelan **una versión** (commit de `main` + artefacto de puntaje) para que ustedes ensayen con números que no cambian. Hasta entonces, hoy la capa aceptada es **L4b**:

| En pantalla | Valor | Qué es |
| --- | --- | --- |
| 7.516 | subidas a las 18:00 en Tobalaba, día laboral promedio de abril 2026 | observado, DTPM |
| −513 | cambio de subidas al día al cambiar el MUT por 298 viviendas | contabilidad del sitio, supuestos visibles |
| −2.781 | mismo cambio según el modelo de estaciones L4b, en Tobalaba | sensibilidad, sin validar; su base es 51% de lo observado |
| 83 / 51 | nuevos residentes que salen en auto / en transporte público al día | Censo 2024 Las Condes, supuesto de transferencia |
| 45% | error agregado del modelo en 31 estaciones reservadas | puntaje de desarrollo |
| 0,1 ms | tiempo del cálculo por comuna en este portátil | medido |

Cada número hablado lleva la capa congelada. Si una capa cambia después de las 12:30, la pantalla y el discurso no cambian hasta que ustedes lo digan.

## Lo que no sabemos (una línea cada uno)

- Cuántos de los trabajadores del MUT entran al Metro: el 0,2 es un supuesto; el modelo de estaciones usa otra vía. Por eso −513 y −2.781 son dos estimaciones, no un rango.
- El Censo no separa Metro de bus: digan "transporte público", nunca "Metro", para cifras del Censo.
- Los 4.679 empleos del MUT son modelados (superficie), no medidos.
- Datos de 2023 alrededor de la apertura del MUT (PR #77 a #79): las subidas de Tobalaba en Línea 4 subieron 20% y el resto de la línea bajó 1,5%. Es descriptivo; no mide el efecto del MUT, no se compara con 2026 ni con los modelos.

## Las 5 preguntas para ustedes (con nuestra recomendación)

1. **Comuna y palabras**: decir "Las Condes, al lado de Tobalaba"; "transporte público" para el Censo, "Metro" solo para subidas DTPM.
2. **La sorpresa del MUT**: mostrar las dos estimaciones con sus etiquetas, el cambio pequeño en la estación y los residentes que salen en auto. Recomendación: las dos, con la brecha de la base visible; esconder un resultado es peor que un límite visible.
3. **Casos 2 y 3**: Estación Central (57 torres de 20 pisos o más, 477 viviendas en promedio; corre hoy en la vista Santiago) como caso 2. El estadio no está construido: ¿se nombra como "la próxima pregunta" o se corta?
4. **Los datos de 2023**: ¿se dicen en dos frases como contexto ("no sabemos cuánto es del MUT; nuestros modelos solo cuentan trabajadores") o se dejan para las preguntas del jurado? Recomendación: dejarlos para las preguntas si el ensayo pasa de 3 minutos.
5. **Tiempo y cierre**: el discurso tiene 256 palabras (~2 minutos a 130 palabras por minuto, antes de clics y de la respuesta del jurado). El cierre dice "tres preguntas" con dos casos construidos: recomendación, "dos preguntas, y la tercera la hacen ustedes". Si sobra tiempo que cortar: primero el contexto 2023, después la línea del modelo de estaciones; nunca la apertura ni el número del jurado.

Respondan en `#agent-chat` o editen `context/pitch.md`. Los agentes siguen en el hilo.
