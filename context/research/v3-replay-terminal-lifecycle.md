# Terminal durante el replay de V3

V3 pide una terminal dentro del mapa. El lanzador actual abre dos ventanas del sistema en macOS y Linux. El flujo integrado sigue pendiente. [Objetivo de V3](../verticals.md) · [Lanzador](../../run.sh) · [macOS](../../tools/city-terminal-mac.sh) · [Linux](../../tools/city-terminal-linux.sh)

| Momento | Comportamiento propuesto |
| --- | --- |
| Pregunta y supuestos | La terminal recibe el teclado. El chat busca el replay guardado y muestra qué cambia, cuándo comienza, cuánto tiempo simula y qué analiza. El presentador decide cuándo iniciar. |
| Escena A | La terminal se oculta y cede el foco al mapa. El relay publica A. El presentador confirma que A aparece y que el público tuvo tiempo de verla. |
| Escena B y gráfico | El presentador activa **Avanzar** desde el mapa o desde un control de presentación. El relay publica B. La terminal sigue oculta hasta que el presentador termina de mostrar el gráfico. |
| Fin | El presentador activa **Volver al chat**. La terminal recupera el foco, la misma sesión y la transcripción. El presentador puede escribir la siguiente pregunta. |

El relay de #350 publica el resultado guardado con una revisión nueva. Su respuesta confirma la publicación. El visor conserva el resultado más reciente de su cola en cada ciclo, así que publicar A y B juntas puede saltarse A. El prompt de #372 debe esperar el avance explícito tras ver A. El control debe funcionar con la terminal oculta. La duración guardada es tiempo simulado y el presentador controla el tiempo en pantalla. [Relay #350](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/350) · [Prompt #372](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/372) · [Cola del visor](../../viz/src/net.rs) · [Orden de planos](v3-ordered-replay-shots.md) · [PR #355](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/355)

**V3** prepara los supuestos, fija el orden de los resultados guardados y define cuándo habilitar el avance. **V2** controla la visibilidad de la terminal, el foco y el control del mapa si se usa esa opción. Como referencias, Bevy permite ocultar un nodo con `Display::None` y xterm.js permite entregar o retirar el foco con `focus()` y `blur()`. Estas referencias sirven para acordar el comportamiento. [Bevy](https://docs.rs/bevy/0.20.1/bevy/ui/enum.Display.html#variant.None) · [xterm.js](https://xtermjs.org/docs/api/terminal/classes/terminal/)

**Ensayo conjunto en el visor nativo.** V2 y V3 revisan la misma toma completa. Registran los ID de replay, las revisiones publicadas y un fotograma visible de cada escena. Comprueban que el control permite avanzar solo después de ver A y que B y el gráfico coinciden con los resultados guardados. Mantienen el relay activo y verifican cero llamadas al cálculo de simulación. Comprueban que la terminal permanece oculta hasta el fin y luego recibe el teclado en la misma sesión. Cronometran la toma completa con narración.
